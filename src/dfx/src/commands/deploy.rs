use crate::lib::agent::create_agent_environment;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use crate::lib::operations::canister::deploy_canisters::deploy_canisters;
use crate::lib::operations::canister::deploy_canisters::DeployMode::{
    ComputeEvidence, ForceReinstallSingleCanister, NormalDeploy, PrepareForProposal,
};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::argument_from_cli::ArgumentFromCliLongOpt;
use crate::util::clap::install_mode::{InstallModeHint, InstallModeOpt};
use crate::util::clap::parsers::{cycle_amount_parser, icrc_subaccount_parser};
use crate::util::clap::subnet_selection_opt::SubnetSelectionOpt;
use crate::util::url::{construct_frontend_url, construct_ui_canister_url};
use anyhow::{anyhow, bail};
use candid::Principal;
use clap::Parser;
use console::Style;
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::identity::CallSender;
use icrc_ledger_types::icrc1::account::Subaccount;
use slog::info;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::runtime::Runtime;
use url::Url;

/// Deploys all or a specific canister from the code in your project. By default, all canisters are deployed.
#[derive(Parser)]
pub struct DeployOpts {
    /// Specifies the name of the canister you want to deploy.
    /// If you don’t specify a canister name, all canisters defined in the dfx.json file are deployed.
    canister_name: Option<String>,

    #[command(flatten)]
    argument_from_cli: ArgumentFromCliLongOpt,

    #[command(flatten)]
    install_mode: InstallModeOpt,

    /// Upgrade the canister even if the .wasm did not change.
    #[arg(long)]
    upgrade_unchanged: bool,

    #[command(flatten)]
    network: NetworkOpt,

    /// Specifies the initial cycle balance to deposit into the newly created canister.
    /// The specified amount needs to take the canister create fee into account.
    /// This amount is deducted from the wallet's cycle balance.
    #[arg(long, value_parser = cycle_amount_parser)]
    with_cycles: Option<u128>,

    /// Attempts to create the canister with this Canister ID.
    ///
    /// This option only works with non-mainnet replica.
    /// This option implies the --no-wallet flag.
    /// This option takes precedence over the specified_id field in dfx.json.
    #[arg(long, value_name = "PRINCIPAL", requires = "canister_name")]
    specified_id: Option<Principal>,

    /// Specify a wallet canister id to perform the call.
    /// If none specified, defaults to use the selected Identity's wallet canister.
    #[arg(long)]
    wallet: Option<String>,

    /// Performs the create call with the user Identity as the Sender of messages.
    /// Bypasses the Wallet canister.
    #[arg(long, conflicts_with("wallet"))]
    no_wallet: bool,

    /// Output environment variables to a file in dotenv format (without overwriting any user-defined variables, if the file already exists).
    #[arg(long)]
    output_env_file: Option<PathBuf>,

    /// Skips yes/no checks by answering 'yes'. Such checks usually result in data loss,
    /// so this is not recommended outside of CI.
    #[arg(long, short)]
    yes: bool,

    /// Skips upgrading the asset canister, to only install the assets themselves.
    #[arg(long)]
    no_asset_upgrade: bool,

    /// Prepare (upload) assets for later commit by proposal.
    #[arg(long, conflicts_with("compute_evidence"))]
    by_proposal: bool,

    /// Compute evidence and compare it against expected evidence
    #[arg(long, conflicts_with("by_proposal"))]
    compute_evidence: bool,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction deduplication, default is system time.
    /// https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long, requires = "canister_name")]
    created_at_time: Option<u64>,

    /// Subaccount of the selected identity to spend cycles from.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    from_subaccount: Option<Subaccount>,

    #[command(flatten)]
    subnet_selection: SubnetSelectionOpt,

    /// Skip compilation before deploying.
    #[arg(long)]
    no_compile: bool,

    /// Always use Candid assist when the argument types are all optional.
    #[arg(
        long,
        conflicts_with("argument"),
        conflicts_with("argument_file"),
        conflicts_with("yes")
    )]
    always_assist: bool,
}

pub fn exec(env: &dyn Environment, opts: DeployOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network.to_network_name())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");

    let canister_name = opts.canister_name.as_deref();
    let (argument_from_cli, argument_type) = opts.argument_from_cli.get_argument_and_type()?;
    if argument_from_cli.is_some() && canister_name.is_none() {
        bail!("The init argument can only be set when deploying a single canister.");
    }
    let mode_hint = opts.install_mode.mode_for_deploy()?;
    let config = env.get_config_or_anyhow()?;
    let env_file = config.get_output_env_file(opts.output_env_file)?;
    let mut subnet_selection =
        runtime.block_on(opts.subnet_selection.into_subnet_selection_type(&env))?;
    let with_cycles = opts.with_cycles;

    let deploy_mode = match (&mode_hint, canister_name) {
        (InstallModeHint::Reinstall, Some(canister_name)) => {
            let network = env.get_network_descriptor();
            if config
                .get_config()
                .is_remote_canister(canister_name, &network.name)?
            {
                bail!("The '{}' canister is remote for network '{}' and cannot be force-reinstalled from here",
                    canister_name, &network.name);
            }
            ForceReinstallSingleCanister(canister_name.to_string())
        }
        (InstallModeHint::Reinstall, None) => {
            bail!("The --mode=reinstall is only valid when deploying a single canister, because reinstallation destroys all data in the canister.");
        }
        (_, None) if opts.by_proposal => {
            bail!("The --by-proposal flag is only valid when deploying a single canister.");
        }
        (_, Some(canister_name)) if opts.by_proposal => {
            PrepareForProposal(canister_name.to_string())
        }
        (_, None) if opts.compute_evidence => {
            bail!("The --compute-evidence flag is only valid when deploying a single canister.");
        }
        (_, Some(canister_name)) if opts.compute_evidence => {
            ComputeEvidence(canister_name.to_string())
        }
        (_, _) => NormalDeploy,
    };

    let call_sender = CallSender::from(&opts.wallet, env.get_network_descriptor())
        .map_err(|e| anyhow!("Failed to determine call sender: {}", e))?;

    runtime.block_on(fetch_root_key_if_needed(&env))?;

    runtime.block_on(deploy_canisters(
        &env,
        canister_name,
        argument_from_cli.as_deref(),
        argument_type.as_deref(),
        &deploy_mode,
        &mode_hint,
        opts.upgrade_unchanged,
        with_cycles,
        opts.created_at_time,
        opts.specified_id,
        &call_sender,
        opts.from_subaccount,
        opts.no_wallet,
        opts.yes,
        env_file,
        opts.no_asset_upgrade,
        &mut subnet_selection,
        opts.always_assist,
        opts.no_compile,
    ))?;

    if matches!(deploy_mode, NormalDeploy | ForceReinstallSingleCanister(_)) {
        display_urls(&env)?;
    }
    Ok(())
}

fn display_urls(env: &dyn Environment) -> DfxResult {
    let config = env.get_config_or_anyhow()?;
    let network: &NetworkDescriptor = env.get_network_descriptor();
    let log = env.get_logger();
    let canister_id_store = env.get_canister_id_store()?;

    let mut frontend_urls = BTreeMap::new();
    let mut candid_urls: BTreeMap<&String, Url> = BTreeMap::new();

    if let Some(canisters) = &config.get_config().canisters {
        for (canister_name, canister_config) in canisters {
            let canister_is_remote = config
                .get_config()
                .is_remote_canister(canister_name, &network.name)?;
            if canister_is_remote {
                continue;
            }
            let canister_id = match Principal::from_text(canister_name) {
                Ok(principal) => Some(principal),
                Err(_) => canister_id_store.find(canister_name),
            };
            if let Some(canister_id) = canister_id {
                let canister_info = CanisterInfo::load(&config, canister_name, Some(canister_id))?;

                // If the canister is an assets canister or has a frontend section, we can display a frontend url.
                let is_assets = canister_info.is_assets() || canister_config.frontend.is_some();

                if is_assets {
                    let url = construct_frontend_url(network, &canister_id)?;
                    frontend_urls.insert(canister_name, url);
                }

                if !canister_info.is_assets() {
                    let url = construct_ui_canister_url(env, &canister_id)?;
                    if let Some(ui_canister_url) = url {
                        candid_urls.insert(canister_name, ui_canister_url);
                    }
                }
            }
        }
    }

    if !frontend_urls.is_empty() || !candid_urls.is_empty() {
        info!(log, "URLs:");
        let green = Style::new().green();
        if !frontend_urls.is_empty() {
            info!(log, "  Frontend canister via browser");
            for (name, (url1, url2)) in frontend_urls {
                if let Some(url2) = url2 {
                    info!(log, "    {}:", name);
                    info!(log, "      - {}", green.apply_to(url1));
                    info!(log, "      - {}", green.apply_to(url2));
                } else {
                    info!(log, "    {}: {}", name, green.apply_to(url1));
                }
            }
        }
        if !candid_urls.is_empty() {
            info!(log, "  Backend canister via Candid interface:");
            for (name, url) in candid_urls {
                info!(log, "    {}: {}", name, green.apply_to(url));
            }
        }
    }

    Ok(())
}
