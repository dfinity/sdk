//! Implements the `dfx nns install` command, which installs the core NNS canisters, including II and NNS-dapp.
//!
//! Note: `dfx nns` will be a `dfx` plugin, so this code SHOULD NOT depend on NNS code except where extremely inconvenient or absolutely necessary:
//! * Example: Minimise crate dependencies outside the nns modules.
//! * Example: Use `anyhow::Result` not `DfxResult`
use crate::config::dfinity::{Config, ConfigNetwork, ReplicaSubnetType};
use crate::lib::environment::Environment;
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::{create_canister, install_canister_wasm};
use crate::util::{blob_from_arguments, expiry_duration};

use anyhow::{anyhow, Context};
use fn_error_context::context;
use garcon::{Delay, Waiter};
use ic_agent::Agent;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use libflate::gzip::Decoder;
use reqwest::Url;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;
use std::time::Duration;

/// The local directory where NNS wasm files are cached.  This is typically created on demand.
const NNS_WASM_DIR: &'static str = "wasm/nns";
/// The name typically used in dfx.json to refer to the Internet& Identity canister, which provides a login service.
const II_NAME: &'static str = "internet_identity";
/// The name of the Internet Identity wasm file in the local wasm cache.
const II_WASM: &'static str = "internet_identity_dev.wasm";
/// The URL from which the Internet Identity wasm file is downloaded, if not already present in the local cache.
const II_URL: &'static str = "https://github.com/dfinity/internet-identity/releases/download/release-2022-07-11/internet_identity_dev.wasm";
/// The name of the NNS frontend dapp, used primarily for voting but also as a wallet.
const ND_NAME: &'static str = "nns-dapp";
/// The name of the NNS frontend dapp in the local cache.
const ND_WASM: &'static str = "nns-dapp_local.wasm";
/// The URL from which the NNS dapp wasm file is downloaded, if not already present in the local cache
const ND_URL: &'static str =
    "https://github.com/dfinity/nns-dapp/releases/download/tip/nns-dapp_local.wasm";

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
#[context("Failed to install nns components.")]
pub async fn install_nns(
    env: &dyn Environment,
    agent: &Agent,
    _provider_url: &str,
    ic_nns_init_path: &Path,
    _replicated_state_dir: &Path,
) -> anyhow::Result<()> {
    println!("Checking out the environment...");
    // Check out the environment.
    verify_local_replica_type_is_system()?;
    let subnet_id = get_local_subnet_id()?;
    let nns_url = get_replica_url()?;

    // Wait for the server to be ready...
    println!("Waiting for the server to be ready...");
    get_with_retries(&Url::parse(&nns_url)?).await?;

    // Install the core backend wasm canisters
    download_nns_wasms().await?;
    let ic_nns_init_opts = IcNnsInitOpts {
        wasm_dir: NNS_WASM_DIR.to_string(),
        nns_url: nns_url.clone(),
        test_accounts: Some(
            "5b315d2f6702cb3a27d826161797d7b2c2e131cd312aece51d4d5574d1247087".to_string(),
        ),
        sns_subnets: Some(subnet_id.clone()),
    };
    ic_nns_init(ic_nns_init_path, &ic_nns_init_opts).await?;
    // ... and configure the backend NNS canisters:
    set_xdr_rate(1234567, &nns_url)?;
    set_cmc_authorized_subnets(&nns_url, &subnet_id)?;

    // Install the GUI canisters:
    download(
        &Path::new(&NNS_WASM_DIR).join(&II_WASM),
        &Url::parse(&II_URL)?,
    )
    .await?;
    install_canister(env, agent, II_NAME, &format!("{NNS_WASM_DIR}/{II_WASM}")).await?;
    install_canister(env, agent, ND_NAME, &format!("{NNS_WASM_DIR}/{ND_WASM}")).await?;
    Ok(())
}

/// Gets a URL, trying repeatedly until it is available.
pub async fn get_with_retries(url: &Url) -> anyhow::Result<reqwest::Response> {
    const RETRY_PAUSE: Duration = Duration::from_millis(200);
    const MAX_RETRY_PAUSE: Duration = Duration::from_secs(5);

    let mut waiter = Delay::builder()
        .exponential_backoff_capped(RETRY_PAUSE, 1.4, MAX_RETRY_PAUSE)
        .build();

    loop {
        println!("Trying the NNS URL...");
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
fn local_replica_type() -> Result<ReplicaSubnetType, &'static str> {
    let dfx_config = Config::from_current_dir()
        .map_err(|_| "Could not get config from dfx.json.")?
        .ok_or("No config in dfx.json")?;
    let network = dfx_config
        .get_config()
        .get_network("local")
        .ok_or("'local' network is not defined in dfx.json.")?;
    let local_network = if let ConfigNetwork::ConfigLocalProvider(local_network) = network {
        local_network
    } else {
        return Err("In dfx.json, 'local' is not a local provider.");
    };
    let local_replica_config = local_network
        .replica
        .as_ref()
        .expect("In dfx.json, 'local' network has no replica setting.");
    local_replica_config
        .subnet_type
        .ok_or("Replica type is not defined for 'local' network.")
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
pub fn verify_local_replica_type_is_system() -> anyhow::Result<()> {
    match local_replica_type() {
        Ok(ReplicaSubnetType::System) => Ok(()),
        other => Err(anyhow!("In dfx.json networks.local.replica.subnet_type needs to be \"system\" to run NNS canisters.  Current value: {other:?}")),
    }
}

/// Downloads a file.
pub async fn download(target: &Path, source: &Url) -> anyhow::Result<()> {
    let response = reqwest::get(source.clone()).await?.bytes().await?;
    let mut decoder = Decoder::new(&response[..])?;
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;

    let tmp_dir = tempfile::Builder::new().prefix(target).tempdir()?;
    let downloaded_filename = {
        let filename = tmp_dir.path().join(target);
        let mut file = fs::File::create(&filename)?;
        file.write_all(&buffer)?;
        filename
    };
    fs::rename(downloaded_filename, target)?;
    Ok(())
}

/// Downloads wasm file from the main IC repo CI.
pub async fn download_ic_repo_wasm(
    target_name: &str,
    src_name: &str,
    ic_commit: &str,
    wasm_dir: &str,
) -> anyhow::Result<()> {
    fs::create_dir_all(wasm_dir)?;
    let final_path = Path::new(wasm_dir).join(format!("{target_name}.wasm"));
    println!("{final_path:?}");
    if final_path.exists() {
        return Ok(());
    }

    let url_str =
        format!("https://download.dfinity.systems/ic/{ic_commit}/canisters/{src_name}.wasm.gz");
    let url = Url::parse(&url_str)?;
    download(&final_path, &url).await
}

/// Downloads all the core NNS wasms, excluding only the front-end wasms II and NNS-dapp.
pub async fn download_nns_wasms() -> anyhow::Result<()> {
    // TODO: Include the canister ID in the path.  .dfx/local/wasms/nns/${COMMIT}/....
    let ic_commit = "3982db093a87e90cbe0595877a4110e4f37ac740"; // TODO: Where should this commit come from?
    for (src_name, target_name) in [
        ("registry-canister", "registry-canister"),
        ("governance-canister", "governance-canister_test"),
        ("ledger-canister", "ledger-canister_notify-method"),
        ("ic-icrc1-ledger", "ic-icrc1-ledger"),
        ("root-canister", "root-canister"),
        ("cycles-minting-canister", "cycles-minting-canister"),
        ("lifeline", "lifeline"),
        ("sns-wasm-canister", "sns-wasm-canister"),
        ("genesis-token-canister", "genesis-token-canister"),
        ("identity-canister", "identity-canister"),
        ("nns-ui-canister", "nns-ui-canister"),
    ] {
        download_ic_repo_wasm(src_name, target_name, ic_commit, NNS_WASM_DIR).await?;
    }
    for (wasm, url) in [(II_WASM, II_URL), (ND_WASM, ND_URL)] {
        download(&Path::new(&NNS_WASM_DIR).join(wasm), &Url::parse(url)?).await?;
    }
    Ok(())
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
///
/// # Panics
/// This code is not expected to panic.
fn get_replica_url() -> Result<String, io::Error> {
    let port = fs::read_to_string(".dfx/replica-configuration/replica-1.port")
        .map(|string| string.trim().to_string())?;
    Ok(format!("http://localhost:{port}"))
}

/// Arguments for the ic-nns-init command line function.
pub struct IcNnsInitOpts {
    nns_url: String,
    wasm_dir: String,
    test_accounts: Option<String>, // TODO, does the CLI actually support several?
    sns_subnets: Option<String>,   // TODO: Can there be several?
}

/// Calls the `ic-nns-init` executable.
///
/// Notes:
///   - Set DFX_IC_NNS_INIT_PATH=<path to binary> to use a different binary for local development
///   - This won't work with an HSM, because the agent holds a session open
///   - The provider_url is what the agent connects to, and forwards to the replica.
#[context("Failed to install nns components.")]
pub async fn ic_nns_init(ic_nns_init_path: &Path, opts: &IcNnsInitOpts) -> anyhow::Result<()> {
    println!("Before ic-nns-init");

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
    println!("{:?}", &cmd);
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

/// Gets the local subnet ID.
///
// TODO: This is a hack.  Need a proper protobuf parser.  dalves mentioned that he might do this, else I'll dive in.
pub fn get_local_subnet_id() -> anyhow::Result<String> {
    // protoc --decode_raw <.dfx/state/replicated_state/ic_registry_local_store/0000000000/00/00/01.pb | sed -nE 's/.*"subnet_record_(.*)".*/\1/g;ta;b;:a;p'
    let file = fs::File::open(
        ".dfx/network/local/state/replicated_state/ic_registry_local_store/0000000000/00/00/01.pb",
    )?;
    let parsed = std::process::Command::new("protoc")
        .arg("--decode_raw")
        .stdin(file)
        .output()
        .expect("Failed to start protobuf file parser");
    let parsed_str = std::str::from_utf8(&parsed.stdout)?;
    parsed_str
        .split("\n")
        .into_iter()
        .find_map(|line| line.split("subnet_record_").into_iter().nth(1))
        .and_then(|line| line.split("\"").next())
        .map(|subnet| subnet.to_string())
        .ok_or(anyhow!("Protobuf has no subnet"))
}

/// Sets the exchange rate between ICP and cycles.
///
/// # Implementation
/// This is done by proposal.  Just after startung a test server, ic-admin
/// proposals with a test user pass immediately, as the small test neuron is
/// the only neuron and has absolute majority.
pub fn set_xdr_rate(rate: u64, nns_url: &str) -> anyhow::Result<()> {
    std::process::Command::new("ic-admin")
        .arg("--nns-url")
        .arg(nns_url)
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
pub fn set_cmc_authorized_subnets(nns_url: &str, subnet: &str) -> anyhow::Result<()> {
    std::process::Command::new("ic-admin")
        .arg("--nns-url")
        .arg(nns_url)
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

/// Installs a canister without adding it to dfx.json.
///
/// # Errors
/// - Returns an error if the canister could not be created.
/// # Panics
/// None
//
// Notes:
// - This does not pass any initialisation argument.  If needed, one can be added to the code.
// - This function may be needed by other plugins as well.
pub async fn install_canister(
    env: &dyn Environment,
    agent: &Agent,
    canister_name: &str,
    wasm_path: &str,
) -> anyhow::Result<()> {
    env.get_logger();
    let timeout = expiry_duration();
    let with_cycles = None;
    let call_sender = CallSender::SelectedId;
    let canister_settings = CanisterSettings {
        controllers: None,
        compute_allocation: None,
        memory_allocation: None,
        freezing_threshold: None,
    };

    create_canister(
        env,
        canister_name,
        timeout,
        with_cycles,
        &call_sender,
        canister_settings,
    )
    .await?;

    let canister_id_store = CanisterIdStore::for_env(env)?;
    let canister_id = canister_id_store.get(canister_name)?;

    println!("Canister ID: {:?}", canister_id.to_string());
    let install_args = blob_from_arguments(None, None, None, &None)?;
    let install_mode = InstallMode::Install;

    install_canister_wasm(
        env,
        agent,
        canister_id,
        Some(canister_name),
        &install_args,
        install_mode,
        timeout,
        &call_sender,
        fs::read(&wasm_path).with_context(|| format!("Unable to read {}", wasm_path))?,
    )
    .await?;

    println!("Installed internet identity");
    Ok(())
}
