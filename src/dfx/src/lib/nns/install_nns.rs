//! Implements the `dfx nns install` command, which installs the core NNS canisters, including II and NNS-dapp.
//!
//! Note: `dfx nns` will be a `dfx` plugin, so this code SHOULD NOT depend on NNS code except where extremely inconvenient or absolutely necessary:
//! * Example: Minimise crate dependencies outside the nns modules.
//! * Example: Use `anyhow::Result` not `DfxResult`
#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use crate::config::cache::get_bin_cache;
use crate::config::dfinity::ReplicaSubnetType;
use crate::lib::environment::Environment;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::info::replica_rev;
use crate::lib::operations::canister::install_canister_wasm;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::blob_from_arguments;
use crate::util::expiry_duration;
use crate::util::network::get_replica_urls;

use anyhow::{anyhow, bail, Context};
use flate2::bufread::GzDecoder;
use fn_error_context::context;
use futures_util::future::try_join_all;
use garcon::{Delay, Waiter};
use ic_agent::export::Principal;
use ic_agent::Agent;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use ic_utils::interfaces::ManagementCanister;
use reqwest::Url;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::Duration;

use self::canisters::{
    IcNnsInitCanister, SnsCanisterInstallation, StandardCanister, NNS_CORE, NNS_FRONTEND,
    SNS_CANISTERS,
};

pub mod canisters;

/// Installs NNS canisters on a local dfx server.
/// # Notes:
///   - Set DFX_IC_NNS_INIT_PATH=<path to binary> to use a different &binary for local development
///   - This won't work with an HSM, because the agent holds a session open
///   - The provider_url is what the agent connects to, and forwards to the replica.
/// # Prerequisites
///   - There must be no canisters already present in the dfx server.
///   - The dfx server must be running as subnet type system; this is set in the local network setting in dfx.json and
///     will normally be different from the production network type, which will most
///     likely be "application".
/// # Errors
/// This will return an error if:
/// - Any of the steps failed to complete.
///
/// # Panics
/// Ideally this should never panic and always return an error on error; while this code is in development reality may differ.
#[context("Failed to install NNS components.")]
pub async fn install_nns(
    env: &dyn Environment,
    agent: &Agent,
    ic_nns_init_path: &Path,
) -> anyhow::Result<()> {
    eprintln!("Checking out the environment...");
    verify_local_replica_type_is_system(env)?;
    verify_nns_canister_ids_are_available(agent).await?;
    let provider_url = get_and_check_provider(env)?;
    let nns_url = get_and_check_replica_url(env)?;
    let subnet_id = get_subnet_id(agent).await?.to_text();
    let ic_admin_cli = bundled_binary(env, "ic-admin")?;

    eprintln!("Installing the core backend wasm canisters...");
    download_nns_wasms(env).await?;
    let ic_nns_init_opts = IcNnsInitOpts {
        wasm_dir: nns_wasm_dir(env)?,
        nns_url: nns_url.to_string(),
        test_accounts: vec![
            canisters::ED25519_TEST_ACCOUNT.to_string(),
            canisters::SECP256K1_TEST_ACCOUNT.to_string(),
        ],
        sns_subnets: Some(subnet_id.to_string()),
    };
    ic_nns_init(ic_nns_init_path, &ic_nns_init_opts).await?;

    eprintln!("Uploading NNS configuration data...");
    upload_nns_sns_wasms_canister_wasms(env)?;

    // Install the GUI canisters:
    for StandardCanister {
        wasm_url,
        wasm_name,
        canister_name,
        canister_id,
    } in NNS_FRONTEND
    {
        let local_wasm_path = nns_wasm_dir(env)?.join(wasm_name);
        let parsed_wasm_url = Url::parse(wasm_url)
            .with_context(|| format!("Could not parse url for {canister_name} wasm: {wasm_url}"))?;
        download(&parsed_wasm_url, &local_wasm_path).await?;
        let installed_canister_id = install_canister(env, agent, canister_name, &local_wasm_path)
            .await?
            .to_text();
        if canister_id != &installed_canister_id {
            bail!("Canister '{canister_name}' was installed at an incorrect canister ID.  Expected '{canister_id}' but got '{installed_canister_id}'.");
        }
    }
    // ... and configure the backend NNS canisters:
    eprintln!("Configuring the NNS...");
    set_xdr_rate(1234567, &nns_url, &ic_admin_cli)?;
    set_cmc_authorized_subnets(&nns_url, &subnet_id, &ic_admin_cli)?;

    print_nns_details(provider_url)?;
    Ok(())
}

/// Gets and checks the provider URL
///
/// # Errors
/// - The provider may be malformed.
/// - Only provider localhost:8080 is supported; this is compiled into the canister IDs.
///   - The port constraint may be eased, perhaps, at some stage.
///   - The requirement that the domain root is 'localhost' is less likely to change; 127.0.0.1 doesn't support subdomains.
#[context("Failed to get a valid provider for network '{}'.  Please check networks.json and dfx.json.", env.get_network_descriptor().name)]
fn get_and_check_provider(env: &dyn Environment) -> anyhow::Result<Url> {
    let provider_url = env
        .get_network_descriptor()
        .first_provider()
        .with_context(|| "Environment has no providers")?;
    let provider_url: Url = Url::parse(provider_url)
        .with_context(|| "Malformed provider URL in this environment: {url_str}")?;

    if provider_url.port() != Some(8080) {
        return Err(anyhow!(
            "dfx nns install supports only port 8080, not {provider_url}. Please set the 'local' network's provider to '127.0.0.1:8080'."
        ));
    }

    Ok(provider_url)
}

/// Gets the local replica URL.  Note: This is not the same as the provider URL.
///
/// The replica URL hosts the canister dashboard and is used for installing NNS wasms.
///
/// Note: The port typically changes every time `dfx start --clean` is run.
///
/// # Errors
/// - Returns an error if the replica URL could not be found.  Typically this indicates that the local replica
///   is not running or is running in a different location.
/// - Returns an error if the network name is not "local"; that is the only network that `ic nns install` can deploy to.
///
/// # Panics
/// This code is not expected to panic.
#[context("Failed to determine the replica URL for network '{}'.  This may be caused by `dfx start` failing.", env.get_network_descriptor().name)]
pub fn get_and_check_replica_url(env: &dyn Environment) -> anyhow::Result<Url> {
    let network_descriptor = env.get_network_descriptor();
    if network_descriptor.name != "local" {
        return Err(anyhow!(
            "dfx nns install can only deploy to the 'local' network."
        ));
    }
    get_replica_urls(env, env.get_network_descriptor())?
        .pop()
        .ok_or_else(|| {
            anyhow!("The list of replica URLs is empty; `dfx start` appears to be unhealthy.")
        })
}

/// Gets the subnet ID
#[context("Failed to determine subnet ID.")]
async fn get_subnet_id(agent: &Agent) -> anyhow::Result<Principal> {
    let root_key = agent
        .status()
        .await
        .with_context(|| "Could not get agent status")?
        .root_key
        .with_context(|| "Agent should have fetched the root key.")?;
    Ok(Principal::self_authenticating(root_key))
}

/// The NNS canisters use the very first few canister IDs; they must be available.
#[context("Failed to verify that the network is empty; dfx nns install must be run just after dfx start --clean")]
async fn verify_nns_canister_ids_are_available(agent: &Agent) -> anyhow::Result<()> {
    /// Checks that the canister is unused on the given network.
    ///
    /// The network is queried directly; local state such as canister_ids.json has no effect on this function.
    async fn verify_canister_id_is_available(
        agent: &Agent,
        canister_id: &str,
        canister_name: &str,
    ) -> anyhow::Result<()> {
        let canister_principal = Principal::from_text(canister_id).with_context(|| {
            format!("Internal error: {canister_name} has an invalid canister ID: {canister_id}")
        })?;
        if agent
            .read_state_canister_info(canister_principal, "module_hash", false)
            .await
            .is_ok()
        {
            return Err(anyhow!(
                "The ID for the {canister_name} canister has already been taken."
            ));
        }
        Ok(())
    }

    try_join_all(NNS_CORE.iter().cloned().map(
        |IcNnsInitCanister {
             canister_id,
             canister_name,
             ..
         }| verify_canister_id_is_available(agent, canister_id, canister_name),
    ))
    .await?;
    Ok(())
}

/// Provides the user with a printout detailing what has been installed for them.
///
/// # Errors
/// - May fail if the provider URL is invalid.
#[context("Failed to print NNS details.")]
fn print_nns_details(provider_url: Url) -> anyhow::Result<()> {
    let canister_url = |canister_id: &str| -> anyhow::Result<String> {
        let mut url = provider_url.clone();
        let host = format!("{}.localhost", canister_id);
        url.set_host(Some(&host))
            .with_context(|| "Could not add canister ID as a subdomain to localhost")?;
        Ok(url.to_string())
    };

    println!(
        r#"

######################################
# NNS CANISTER INSTALLATION COMPLETE #
######################################

Backend canisters:
{}

Frontend canisters:
{}

"#,
        NNS_CORE
            .iter()
            .map(|canister| format!("{:20}  {}\n", canister.canister_name, canister.canister_id))
            .collect::<Vec<String>>()
            .join(""),
        NNS_FRONTEND
            .iter()
            .map(|canister| format!(
                "{:20}  {}\n",
                canister.canister_name,
                canister_url(canister.canister_id).unwrap_or_default()
            ))
            .collect::<Vec<String>>()
            .join("")
    );
    Ok(())
}

/// Gets a URL, trying repeatedly until it is available.
#[context("Failed to download after multiple tries: {}", url)]
pub async fn get_with_retries(url: &Url) -> anyhow::Result<reqwest::Response> {
    /// The time between the first try and the second.
    const RETRY_PAUSE: Duration = Duration::from_millis(200);
    /// Intervals will increase exponentially until they reach this.
    const MAX_RETRY_PAUSE: Duration = Duration::from_secs(5);

    let mut waiter = Delay::builder()
        .exponential_backoff_capped(RETRY_PAUSE, 1.4, MAX_RETRY_PAUSE)
        .build();

    loop {
        match reqwest::get(url.clone()).await {
            Ok(response) => {
                return Ok(response);
            }
            Err(err) => waiter.wait().map_err(|_| err)?,
        }
    }
}

/// Gets the local replica type from dfx.json
///
/// # Errors
/// Returns an error if the replica type could not be determined.  Possible reasons include:
/// - There is no `dfx.json`
/// - `dfx.json` could not be read.
/// - `dfx.json` is not valid JSON.
/// - The replica type is not defined for the `local` network.
///
/// # Panics
/// This code is not expected to panic.
#[context("Failed to determine the local replica type.")]
fn local_replica_type(env: &dyn Environment) -> anyhow::Result<ReplicaSubnetType> {
    Ok(env
        .get_network_descriptor()
        .local_server_descriptor()?
        .replica
        .subnet_type
        .unwrap_or_default())
}

/// Checks that the local replica type is 'system'.
///
/// Note: At present dfx runs a single local replica and the replica type is taken from dfx.json.  It is unfortunate that the subnet type is forced
/// on the other canisters, however in practice this is unlikely to be a huge problem in the short term.
///
/// # Errors
/// - Returns an error if the local replica type in `dfx.json` is not "system".
/// # Panics
/// This code is not expected to panic.
#[context("Failed to verify that the local replica type is 'system'.")]
pub fn verify_local_replica_type_is_system(env: &dyn Environment) -> anyhow::Result<()> {
    match local_replica_type(env) {
        Ok(ReplicaSubnetType::System) => Ok(()),
        other => Err(anyhow!("The replica subnet_type needs to be \"system\" to run NNS canisters. Current value: {other:?}. You can configure it by setting defaults.replica.subnet_type in your project's dfx.json or by setting local.replica.subnet_type in your global networks.json to \"system\".")),
    }
}

/// Downloads a file
#[context("Failed to download '{:?}' to '{:?}'.", source, target)]
pub async fn download(source: &Url, target: &Path) -> anyhow::Result<()> {
    if target.exists() {
        println!("Already downloaded: {}", target.to_string_lossy());
        return Ok(());
    }
    println!(
        "Downloading {}\n  from: {}",
        target.to_string_lossy(),
        source.as_str()
    );
    let buffer = reqwest::get(source.clone())
        .await
        .with_context(|| "Failed to connect")?
        .bytes()
        .await
        .with_context(|| "Download was interrupted")?;
    let target_parent = target
        .parent()
        .unwrap_or_else(|| Path::new(Component::CurDir.as_os_str()));
    let tmp_dir = tempfile::TempDir::new_in(target_parent)
        .with_context(|| "Failed to create temporary directory for download")?;
    let downloaded_filename = {
        let filename = tmp_dir.path().join("wasm");
        let mut file = fs::File::create(&filename)
            .with_context(|| format!("Failed to create temp file at '{}'", filename.display()))?;
        file.write_all(&buffer)
            .with_context(|| format!("Failed to write temp file at '{}'.", filename.display()))?;
        filename
    };
    fs::rename(&downloaded_filename, target).with_context(|| {
        format!(
            "Failed to rename '{}' to '{}'",
            downloaded_filename.display(),
            target.display()
        )
    })?;
    Ok(())
}

/// Downloads and unzips a file
#[context("Failed to download and unzip '{:?}' from '{:?}'.", target, source.as_str())]
pub async fn download_gz(source: &Url, target: &Path) -> anyhow::Result<()> {
    if target.exists() {
        println!("Already downloaded: {}", target.to_string_lossy());
        return Ok(());
    }
    println!(
        "Downloading {}\n  from .gz: {}",
        target.to_string_lossy(),
        source.as_str()
    );
    let response = reqwest::get(source.clone())
        .await
        .with_context(|| "Failed to connect")?
        .bytes()
        .await
        .with_context(|| "Download was interrupted")?;
    let mut decoder = GzDecoder::new(&response[..]);

    let target_parent = target
        .parent()
        .unwrap_or_else(|| Path::new(Component::CurDir.as_os_str()));
    let tmp_dir = tempfile::TempDir::new_in(target_parent)
        .with_context(|| "Failed to create temporary directory for download")?;
    let downloaded_filename = {
        let filename = tmp_dir.path().join("wasm");
        let mut file = fs::File::create(&filename).with_context(|| {
            format!(
                "Failed to write temp file when downloading '{}'.",
                filename.display()
            )
        })?;
        std::io::copy(&mut decoder, &mut file)
            .with_context(|| format!("Failed to unzip WASM to '{}'", filename.display()))?;
        filename
    };
    fs::rename(&downloaded_filename, target).with_context(|| {
        format!(
            "Failed to move downloaded tempfile '{}' to '{}'.",
            downloaded_filename.display(),
            target.display()
        )
    })?;
    Ok(())
}

/// Downloads wasm file from the main IC repo CI.
#[context("Failed to download {} from the IC CI.", wasm_name)]
pub async fn download_ic_repo_wasm(
    wasm_name: &str,
    ic_commit: &str,
    wasm_dir: &Path,
) -> anyhow::Result<()> {
    fs::create_dir_all(wasm_dir)
        .with_context(|| format!("Failed to create wasm directory: '{}'", wasm_dir.display()))?;
    let final_path = wasm_dir.join(&wasm_name);
    let url_str =
        format!("https://download.dfinity.systems/ic/{ic_commit}/canisters/{wasm_name}.gz");
    let url = Url::parse(&url_str)
      .with_context(|| format!("Could not determine download URL. Are ic_commit '{ic_commit}' and wasm_name '{wasm_name}' valid?"))?;
    download_gz(&url, &final_path).await
}

/// Downloads all the core NNS wasms, excluding only the front-end wasms II and NNS-dapp.
#[context("Failed to download NNS wasm files.")]
pub async fn download_nns_wasms(env: &dyn Environment) -> anyhow::Result<()> {
    let ic_commit = std::env::var("DFX_IC_COMMIT").unwrap_or_else(|_| replica_rev().to_string());
    let wasm_dir = &nns_wasm_dir(env)?;
    for IcNnsInitCanister {
        wasm_name,
        test_wasm_name,
        ..
    } in NNS_CORE
    {
        download_ic_repo_wasm(wasm_name, &ic_commit, wasm_dir).await?;
        if let Some(test_wasm_name) = test_wasm_name {
            download_ic_repo_wasm(test_wasm_name, &ic_commit, wasm_dir).await?;
        }
    }
    try_join_all(
        SNS_CANISTERS
            .iter()
            .map(|SnsCanisterInstallation { wasm_name, .. }| {
                download_ic_repo_wasm(wasm_name, &ic_commit, wasm_dir)
            }),
    )
    .await?;
    Ok(())
}

/// Arguments for the ic-nns-init command line function.
pub struct IcNnsInitOpts {
    /// An URL to accees one or more NNS subnet replicas.
    nns_url: String,
    /// A directory that needs to be populated will all required wasms before calling ic-nns-init.
    wasm_dir: PathBuf,
    /// The ID of a test account that ic-nns-init will create and to initialise with tokens.
    /// Note: At present only one test account is supported.
    test_accounts: Vec<String>,
    /// A subnet for SNS canisters.
    /// Note: In this context we support at most one subnet.
    sns_subnets: Option<String>,
}

/// Calls the `ic-nns-init` executable.
///
/// Notes:
///   - Set DFX_IC_NNS_INIT_PATH=<path to binary> to use a different binary for local development
///   - This won't work with an HSM, because the agent holds a session open
///   - The provider_url is what the agent connects to, and forwards to the replica.
#[context("Failed to install NNS components.")]
pub async fn ic_nns_init(ic_nns_init_path: &Path, opts: &IcNnsInitOpts) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new(ic_nns_init_path);
    cmd.arg("--url");
    cmd.arg(&opts.nns_url);
    cmd.arg("--wasm-dir");
    cmd.arg(&opts.wasm_dir);
    opts.test_accounts.iter().for_each(|account| {
        cmd.arg("--initialize-ledger-with-test-accounts");
        cmd.arg(account);
    });
    opts.sns_subnets.iter().for_each(|subnet| {
        cmd.arg("--sns-subnet");
        cmd.arg(subnet);
    });
    let args: Vec<_> = cmd
        .get_args()
        .into_iter()
        .map(OsStr::to_string_lossy)
        .collect();
    println!("ic-nns-init {}", args.join(" "));
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    let output = cmd
        .output()
        .with_context(|| format!("Error executing {:#?}", cmd))?;

    if !output.status.success() {
        return Err(anyhow!("ic-nns-init call failed"));
    }
    Ok(())
}

/// Sets the exchange rate between ICP and cycles.
///
/// # Implementation
/// This is done by proposal.  Just after startung a test server, ic-admin
/// proposals with a test user pass immediately, as the small test neuron is
/// the only neuron and has absolute majority.
#[context("Failed to set an initial exchange rate between ICP and cycles.  It may not be possible to create canisters or purchase cycles.")]
pub fn set_xdr_rate(rate: u64, nns_url: &Url, ic_admin: &PathBuf) -> anyhow::Result<()> {
    std::process::Command::new(ic_admin)
        .arg("--nns-url")
        .arg(nns_url.as_str())
        .arg("propose-xdr-icp-conversion-rate")
        .arg("--test-neuron-proposer")
        .arg("--summary")
        .arg(format!("Set the cycle exchange rate to {rate}."))
        .arg("--xdr-permyriad-per-icp")
        .arg(format!("{}", rate))
        .stdin(process::Stdio::null())
        .output()
        .map_err(anyhow::Error::from)
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(anyhow!("Call to propose to set xdr rate failed"))
            }
        })
}

/// Sets the subnets the CMC is authorized to create canisters in.
#[context("Failed to authorize a subnet for use by the cycles management canister.  The CMC may not be able to create canisters.")]
pub fn set_cmc_authorized_subnets(
    nns_url: &Url,
    subnet: &str,
    ic_admin: &PathBuf,
) -> anyhow::Result<()> {
    std::process::Command::new(ic_admin)
        .arg("--nns-url")
        .arg(nns_url.as_str())
        .arg("propose-to-set-authorized-subnetworks")
        .arg("--test-neuron-proposer")
        .arg("--proposal-title")
        .arg("Set Cycles Minting Canister Authorized Subnets")
        .arg("--summary")
        .arg(format!(
            "Authorize the Cycles Minting Canister to create canisters in the subnet '{subnet}'."
        ))
        .arg("--subnets")
        .arg(subnet)
        .stdin(process::Stdio::null())
        .output()
        .map_err(anyhow::Error::from)
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(anyhow!("Call to propose to set xdr rate failed"))
            }
        })
}

/// Uploads wasms to the nns-sns-wasm canister.
#[context("Failed to upload wasm fils to the nns-sns-wasm canister; it may not be possible to create an SNS.")]
pub fn upload_nns_sns_wasms_canister_wasms(env: &dyn Environment) -> anyhow::Result<()> {
    for SnsCanisterInstallation {
        upload_name,
        wasm_name,
        ..
    } in SNS_CANISTERS
    {
        let sns_cli = bundled_binary(env, "sns")?;
        let wasm_path = nns_wasm_dir(env)?.join(wasm_name);
        let mut command = Command::new(sns_cli);
        command
            .arg("add-sns-wasm-for-tests")
            .arg("--network")
            .arg("local")
            .arg("--override-sns-wasm-canister-id-for-tests")
            .arg(canisters::NNS_SNS_WASM.canister_id)
            .arg("--wasm-file")
            .arg(&wasm_path)
            .arg(upload_name);
        command
        .stdin(process::Stdio::null())
        .output()
            .map_err(anyhow::Error::from)
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Failed to upload {} from {} to the nns-sns-wasm canister:\n{:?} {:?}\nStdout:\n{:?}\n\nStderr:\n{:?}",
                        upload_name,
                        wasm_path.to_string_lossy(),
                        command.get_program(),
                        command.get_args(),
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            })?;
    }
    Ok(())
}

/// Installs a canister without adding it to `dfx.json` or `canister_ids.json`.
///
/// # Errors
/// - Returns an error if the canister could not be created.
/// # Panics
/// None
//
// Notes:
// - This does not pass any initialisation argument.  If needed, one can be added to the code.
// - This function may be needed by other plugins as well.
#[context("Failed to install canister '{canister_name}' on network '{}' using wasm at '{}'.", env.get_network_descriptor().name, wasm_path.display())]
pub async fn install_canister(
    env: &dyn Environment,
    agent: &Agent,
    canister_name: &str,
    wasm_path: &Path,
) -> anyhow::Result<Principal> {
    let mgr = ManagementCanister::create(agent);
    let builder = mgr
        .create_canister()
        .as_provisional_create_with_amount(None);

    let res = builder
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await;
    let canister_id: Principal = res.context("Canister creation call failed.")?.0;
    let canister_id_str = canister_id.to_text();

    let install_args = blob_from_arguments(None, None, None, &None)?;
    let install_mode = InstallMode::Install;
    let timeout = expiry_duration();
    let call_sender = CallSender::SelectedId;

    install_canister_wasm(
        env,
        agent,
        canister_id,
        Some(canister_name),
        &install_args,
        install_mode,
        timeout,
        &call_sender,
        fs::read(&wasm_path).with_context(|| format!("Unable to read {:?}", wasm_path))?,
    )
    .await?;

    println!("Installed {canister_name} at {canister_id_str}");

    Ok(canister_id)
}

/// The local directory where NNS wasm files are cached.  The directory is typically created on demand.
fn nns_wasm_dir(env: &dyn Environment) -> anyhow::Result<PathBuf> {
    Ok(get_bin_cache(&env.get_version().to_string())?.join("wasms"))
}

/// Get the path to a bundled command line binary
fn bundled_binary(env: &dyn Environment, cli_name: &str) -> anyhow::Result<PathBuf> {
    env.get_cache()
        .get_binary_command_path(cli_name)
        .with_context(|| format!("Could not find bundled binary '{cli_name}'."))
}
