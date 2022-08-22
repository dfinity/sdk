//! Implements the `dfx nns install` command, which installs the core NNS canisters, including II and NNS-dapp.
//! Note: `dfx nns` will be a dfx plugin, so this code SHOULD NOT depend on NNS code except where extremely inconvenient or absolutely necessary:
//! * Example: Minimise crate dependencies outside the nns modules.
//! * Example: Use `anyhow::Result` not `DfxResult`
use crate::config::dfinity::{Config, ConfigNetwork, ReplicaSubnetType};
use crate::lib::environment::{Environment};
use crate::lib::ic_attributes::CanisterSettings;
use crate::lib::identity::identity_utils::CallSender;
use crate::lib::models::canister_id_store::CanisterIdStore;
use crate::lib::operations::canister::{create_canister, install_canister_wasm};
use crate::util::{blob_from_arguments, expiry_duration};

use anyhow::{anyhow, bail, Context};
use fn_error_context::context;
use ic_agent::Agent;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use libflate::gzip::Decoder;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process;

const NNS_WASM_DIR: &'static str = "wasm/nns";
const II_NAME: &'static str = "internet_identity";
const II_WASM: &'static str = "internet_identity.wasm";
const ND_NAME: &'static str = "nns-dapp";
const ND_WASM: &'static str = "nns-dapp_local.wasm";

#[context("Failed to install nns components.")]
pub async fn install_nns(
    env: &dyn Environment,
    agent: &Agent,
    _provider_url: &str,
    ic_nns_init_path: &Path,
    _replicated_state_dir: &Path,
) -> anyhow::Result<()> {
    /*
    // Notes:
    //   - Set DFX_IC_NNS_INIT_PATH=<path to binary> to use a different &binary for local development
    //   - This won't work with an HSM, because the agent holds a session open
    //   - The provider_url is what the agent connects to, and forwards to the replica.

    let mut cmd = std::process::Command::new(ic_nns_init_path);
    cmd.arg("--help");
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    let output = cmd
        .output()
        .with_context(|| format!("Error executing {:#?}", cmd))?;

    if !output.status.success() {
        bail!("ic-nns-init call failed");
    }
    */
    assert_local_replica_type_is_system();

    download_nns_wasms().await.unwrap();
    let subnet_id = get_local_subnet_id().unwrap();
    let nns_url = get_replica_url().unwrap();

    let ic_nns_init_opts = IcNnsInitOpts {
        wasm_dir: NNS_WASM_DIR.to_string(),
        nns_url: nns_url.clone(),
        test_accounts: Some(
            "5b315d2f6702cb3a27d826161797d7b2c2e131cd312aece51d4d5574d1247087".to_string(),
        ),
        sns_subnets: Some(subnet_id.clone()),
    };

    ic_nns_init(ic_nns_init_path, &ic_nns_init_opts)
        .await
        .unwrap();
    set_xdr_rate(1234567, &nns_url)?;
    set_cmc_authorized_subnets(&nns_url, &subnet_id)?;
    install_canister(env, agent, II_NAME, &format!("{NNS_WASM_DIR}/{II_WASM}")).await?;
    install_canister(env, agent, ND_NAME, &format!("{NNS_WASM_DIR}/{ND_WASM}")).await?;
    Ok(())
}

/// Gets the local replica type.
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
/// Asserts that the local replica type is 'system'.
/// Note: At present dfx runs a single local replica and the replica type is taken from dfx.json.  It is unfortunate that the subnet type is forced
/// on the other canisters, however in practice this is unlikely to be a huge problem in the short term.
pub fn assert_local_replica_type_is_system() {
    match local_replica_type() {
        Ok(ReplicaSubnetType::System) => (),
        other => panic!("In dfx.json networks.local.replica.subnet_type needs to be \"system\" to run NNS canisters.  Current value: {:?}", other),
    }
}

/// Downloads wasm file
pub async fn download_wasm(
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
    let url = reqwest::Url::parse(&url_str)?;
    let response = reqwest::get(url.clone()).await?.bytes().await?;
    let mut decoder = Decoder::new(&response[..])?;
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer).unwrap();

    let tmp_dir = tempfile::Builder::new().prefix(target_name).tempdir()?;
    let downloaded_filename = {
        let filename = tmp_dir.path().join(target_name);
        let mut file = fs::File::create(&filename)?;
        file.write_all(&buffer)?;
        filename
    };
    fs::rename(downloaded_filename, final_path)?;
    Ok(())
}
pub async fn download_nns_wasms() -> anyhow::Result<()> {
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
        download_wasm(src_name, target_name, ic_commit, NNS_WASM_DIR)
            .await
            .unwrap();
    }
    Ok(())
}

/// get the replica URL.  Note: This is not the same as the provider URL.
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

#[context("Failed to install nns components.")]
pub async fn ic_nns_init(ic_nns_init_path: &Path, opts: &IcNnsInitOpts) -> anyhow::Result<()> {
    println!("Before ic-nns-init");
    // Notes:
    //   - Set DFX_IC_NNS_INIT_PATH=<path to binary> to use a different binary for local development
    //   - This won't work with an HSM, because the agent holds a session open
    //   - The provider_url is what the agent connects to, and forwards to the replica.

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
        bail!("ic-nns-init call failed");
    }
    println!("After ic-nns-init");
    Ok(())
}

/// Gets the local subnet ID
/// TODO: This is a hack.  Need a proper protobuf parser.  dalves mentioned that he might do this, else I'll dive in.
pub fn get_local_subnet_id() -> anyhow::Result<String> {
    // protoc --decode_raw <.dfx/state/replicated_state/ic_registry_local_store/0000000000/00/00/01.pb | sed -nE 's/.*"subnet_record_(.*)".*/\1/g;ta;b;:a;p'
    let file = fs::File::open(
        ".dfx/state/replicated_state/ic_registry_local_store/0000000000/00/00/01.pb",
    )
    .unwrap();
    let parsed = std::process::Command::new("protoc")
        .arg("--decode_raw")
        .stdin(file)
        .output()
        .expect("Failed to start protobuf file parser");
    let parsed_str = std::str::from_utf8(&parsed.stdout).unwrap();
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

/// Set the subnets the CMC is authorized to create canisters in.
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


pub async fn download_ii() {
    // TODO
}

pub async fn install_canister(env: &dyn Environment, agent: &Agent, canister_name: &str, wasm_path: &str) -> anyhow::Result<()> {
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
    .await
    .unwrap();

    let canister_id_store = CanisterIdStore::for_env(env).unwrap();
    let canister_id = canister_id_store.get(canister_name).unwrap();

    println!("Canister ID: {:?}", canister_id.to_string());
    let install_args = blob_from_arguments(None, None, None, &None).unwrap();
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
        fs::read(&wasm_path)
            .with_context(|| format!("Unable to read {}", wasm_path))
            .unwrap(),
    )
    .await?;

    println!("Installed internet identity");
    Ok(())
}
