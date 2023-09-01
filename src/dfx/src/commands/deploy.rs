use crate::lib::agent::create_agent_environment;
use crate::lib::canister_info::CanisterInfo;
use crate::lib::error::DfxResult;
use crate::lib::network::network_opt::NetworkOpt;
use crate::lib::operations::canister::deploy_canisters::deploy_canisters;
use crate::lib::operations::canister::deploy_canisters::DeployMode::{
    ComputeEvidence, ForceReinstallSingleCanister, NormalDeploy, PrepareForProposal,
};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::{environment::Environment, named_canister};
use crate::util::clap::parsers::cycle_amount_parser;
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use console::Style;
use dfx_core::canister::url::{
    format_frontend_url, format_ui_canister_url_custom, format_ui_canister_url_ic,
};
use dfx_core::config::model::network_descriptor::NetworkDescriptor;
use dfx_core::identity::CallSender;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use slog::info;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use tokio::runtime::Runtime;
use url::Url;

/// Deploys all or a specific canister from the code in your project. By default, all canisters are deployed.
#[derive(Parser)]
pub struct DeployOpts {
    /// Specifies the name of the canister you want to deploy.
    /// If you don’t specify a canister name, all canisters defined in the dfx.json file are deployed.
    canister_name: Option<String>,

    /// Specifies the argument to pass to the method.
    #[arg(long, requires("canister_name"))]
    argument: Option<String>,

    /// Specifies the data type for the argument when making the call using an argument.
    #[arg(long, requires("argument"), value_parser = ["idl", "raw"])]
    argument_type: Option<String>,

    /// Force the type of deployment to be reinstall, which overwrites the module.
    /// In other words, this erases all data in the canister.
    /// By default, upgrade will be chosen automatically if the module already exists,
    /// or install if it does not.
    #[arg(long, short, value_parser = ["reinstall"])]
    mode: Option<String>,

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
}

pub fn exec(env: &dyn Environment, opts: DeployOpts) -> DfxResult {
    let env = create_agent_environment(env, opts.network.to_network_name())?;

    let canister_name = opts.canister_name.as_deref();
    let argument = opts.argument.as_deref();
    let argument_type = opts.argument_type.as_deref();
    let mode = opts
        .mode
        .as_deref()
        .map(InstallMode::from_str)
        .transpose()
        .map_err(|err| anyhow!(err))
        .context("Failed to parse InstallMode.")?;
    let config = env.get_config_or_anyhow()?;
    let env_file = opts
        .output_env_file
        .or_else(|| config.get_config().output_env_file.clone());

    let with_cycles = opts.with_cycles;

    let deploy_mode = match (mode, canister_name) {
        (Some(InstallMode::Reinstall), Some(canister_name)) => {
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
        (Some(InstallMode::Reinstall), None) => {
            bail!("The --mode=reinstall is only valid when deploying a single canister, because reinstallation destroys all data in the canister.");
        }
        (Some(_), _) => {
            unreachable!("The only valid option for --mode is --mode=reinstall");
        }
        (None, None) if opts.by_proposal => {
            bail!("The --by-proposal flag is only valid when deploying a single canister.");
        }
        (None, Some(canister_name)) if opts.by_proposal => {
            PrepareForProposal(canister_name.to_string())
        }
        (None, None) if opts.compute_evidence => {
            bail!("The --compute-evidence flag is only valid when deploying a single canister.");
        }
        (None, Some(canister_name)) if opts.compute_evidence => {
            ComputeEvidence(canister_name.to_string())
        }
        (None, _) => NormalDeploy,
    };

    let runtime = Runtime::new().expect("Unable to create a runtime");

    let call_sender = CallSender::from(&opts.wallet)
        .map_err(|e| anyhow!("Failed to determine call sender: {}", e))?;

    runtime.block_on(fetch_root_key_if_needed(&env))?;

    runtime.block_on(deploy_canisters(
        &env,
        canister_name,
        argument,
        argument_type,
        &deploy_mode,
        opts.upgrade_unchanged,
        with_cycles,
        opts.specified_id,
        &call_sender,
        opts.no_wallet,
        opts.yes,
        env_file,
        opts.no_asset_upgrade,
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

    let ui_canister_id = named_canister::get_ui_canister_id(&canister_id_store);

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

                let provider = Url::parse(&network.providers[0]).with_context(|| {
                    format!(
                        "Failed to parse url for network provider {}.",
                        &network.providers[0]
                    )
                })?;
                if is_assets {
                    let url = format_frontend_url(&provider, &canister_id.to_string());
                    frontend_urls.insert(canister_name, url);
                }

                if !canister_info.is_assets() {
                    let is_local = env.get_network_descriptor().name == "local";
                    if is_local {
                        let ui_canister_id = ui_canister_id.ok_or_else(|| {
                            anyhow!(
                                "The ui canister id is not set in the canister_id_store.json file."
                            )
                        })?;
                        let url = format_ui_canister_url_custom(
                            &&canister_id.to_string(),
                            &provider,
                            &ui_canister_id.to_string(),
                        );
                        candid_urls.insert(canister_name, url);
                    } else {
                        let url = format_ui_canister_url_ic(&canister_id.to_string())?;
                        candid_urls.insert(canister_name, url);
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
            for (name, url) in frontend_urls {
                info!(log, "    {}: {}", name, green.apply_to(url));
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
