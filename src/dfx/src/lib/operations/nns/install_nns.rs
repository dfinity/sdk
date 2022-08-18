use crate::config::dfinity::{Config, ConfigNetwork, ReplicaSubnetType};
use crate::DfxResult;

use anyhow::{anyhow, bail, Context};
use fn_error_context::context;
use ic_agent::Agent;
use libflate::gzip::Decoder;
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;

#[context("Failed to install nns components.")]
pub async fn install_nns(
    _agent: &Agent,
    provider_url: &str,
    ic_nns_init_path: &Path,
    _replicated_state_dir: &Path,
) -> DfxResult {
    /*
    // Notes:
    //   - Set DFX_IC_NNS_INIT_PATH=<path to binary> to use a different binary for local development
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
    /*
    let ic_nns_init_opts = IcNnsInitOpts {
        wasm_dir: NNS_WASM_DIR.to_string(),
        nns_url: provider_url.to_string(),
        test_accounts: Some("5b315d2f6702cb3a27d826161797d7b2c2e131cd312aece51d4d5574d1247087".to_string()),
    };
    ic_nns_init(ic_nns_init_path, &ic_nns_init_opts).await.unwrap();
    */
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

const NNS_WASM_DIR: &'static str = "wasm/nns";
/// Downloads wasm file
pub async fn download_wasm(wasm_name: &str, ic_commit: &str, wasm_dir: &str) -> anyhow::Result<()> {
    fs::create_dir_all(wasm_dir)?;
    let final_path = Path::new(wasm_dir).join(format!("{wasm_name}.wasm"));
    if final_path.exists() {
        return Ok(());
    }

    let url_str =
        format!("https://download.dfinity.systems/ic/{ic_commit}/canisters/{wasm_name}.wasm.gz");
    let url = reqwest::Url::parse(&url_str)?;
    let mut response = reqwest::get(url.clone()).await?.bytes().await?;
    let mut decoder = Decoder::new(&response[..])?;
    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer).unwrap();

    let tmp_dir = tempfile::Builder::new().prefix(wasm_name).tempdir()?;
    let downloaded_filename = {
        let filename = tmp_dir.path().join(wasm_name);
        let mut file = fs::File::create(&filename)?;
        file.write_all(&buffer);
        filename
    };
    fs::rename(downloaded_filename, final_path)?;
    Ok(())
}
pub async fn download_nns_wasms() -> anyhow::Result<()> {
    let ic_commit = "3982db093a87e90cbe0595877a4110e4f37ac740"; // TODO: Where should this commit come from?
    for wasm_name in ["registry-canister", "governance-canister_test", "governance-canister_test", "ledger-canister_notify-method", "root-canister", "cycles-minting-canister", "lifeline", "sns-wasm-canister", "genesis-token-canister", "identity-canister", "nns-ui-canister"] {
      download_wasm(wasm_name, ic_commit, NNS_WASM_DIR).await.unwrap();
    }
    Ok(())
}

/// Arguments for the ic-nns-init command line function.
pub struct IcNnsInitOpts {
    nns_url: String,
    wasm_dir: String,
    test_accounts: Option<String>, // TODO, does the CLI actually support several?
}

#[context("Failed to install nns components.")]
pub async fn ic_nns_init(ic_nns_init_path: &Path, opts: &IcNnsInitOpts) -> DfxResult {
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
    println!("{:?}", &cmd);
    cmd.stdout(std::process::Stdio::inherit());
    cmd.stderr(std::process::Stdio::inherit());
    let output = cmd
        .output()
        .with_context(|| format!("Error executing {:#?}", cmd))?;

    if !output.status.success() {
        bail!("ic-nns-init call failed");
    }
    Ok(())
}
