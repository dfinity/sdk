mod replica_port;
mod webserver_port;

use std::path::PathBuf;
use std::time::SystemTime;

use crate::commands::info::webserver_port::get_webserver_port;
use crate::config::dfinity::NetworksConfig;
use crate::lib::error::DfxResult;
use crate::lib::info;
use crate::lib::provider::create_agent_environment;
use crate::util::expiry_duration;
use crate::{commands::info::replica_port::get_replica_port, lib::waiter::waiter_with_timeout};
use crate::{Environment, NetworkOpt};

use anyhow::Context;
use candid::{encode_args, CandidType, Decode, Deserialize, Encode, Principal};
use clap::Parser;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use rand::Rng;

#[derive(clap::ValueEnum, Clone, Debug)]
enum InfoType {
    ReplicaPort,
    ReplicaRev,
    WebserverPort,
    NetworksJsonPath,
}

#[derive(Parser)]
#[clap(name("info"))]
pub struct InfoOpts {
    #[clap(value_enum)]
    info_type: InfoType,
}
/// Arguments for the `getCanisterId` call.
#[derive(CandidType)]
pub struct GetCanisterIdArgs {
    pub timestamp: candid::Int,
    pub nonce: candid::Nat,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct CanisterInfo {
    pub id: Principal,
    pub timestamp: candid::Int,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct InstallArgs<'a> {
    pub arg: &'a [u8],
    pub wasm_module: &'a [u8],
    pub mode: InstallMode,
    pub canister_id: Principal,
}

pub async fn exec(env: &dyn Environment, opts: InfoOpts) -> DfxResult {
    let env = create_agent_environment(env, NetworkOpt::default())?;
    let agent = env.get_agent().unwrap();
    agent.fetch_root_key().await?;
    let backend_can = Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap();
    let (timestamp, nonce) = create_nonce();
    let get_can_arg = Encode!(&GetCanisterIdArgs {
        timestamp: timestamp,
        nonce: nonce,
    })?;
    let result = agent
        .update(&backend_can, "getCanisterId")
        .with_arg(get_can_arg)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await
        .context("Notify call failed.")?;
    let out = Decode!(&result, CanisterInfo)?;
    println!("Return value: {:?}", out);

    println!("Trying to install a wasm:");
    let empty_vu8: Vec<u8> = vec![];
    let can_arg = Encode!(&empty_vu8).unwrap();
    let wasm = std::fs::read(PathBuf::from(".dfx/local/canisters/backend/backend.wasm")).unwrap();
    let install_arg = InstallArgs {
        arg: &can_arg,
        wasm_module: &wasm,
        mode: InstallMode::Install,
        canister_id: out.id.clone(),
    };

    let result = agent
        .update(&backend_can, "installCode")
        .with_arg(encode_args((out, install_arg, false))?)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await
        .context("install failed")?;
    let out = Decode!(&result, CanisterInfo)?;
    println!("Install result: {:?}, principal: {}", &out, &out.id);

    let value = match opts.info_type {
        InfoType::ReplicaPort => get_replica_port(&env)?,
        InfoType::ReplicaRev => info::replica_rev().to_string(),
        InfoType::WebserverPort => get_webserver_port(&env)?,
        InfoType::NetworksJsonPath => NetworksConfig::new()?
            .get_path()
            .to_str()
            .context("Failed to convert networks.json path to a string.")?
            .to_string(),
    };
    println!("{}", value);
    Ok(())
}

fn create_nonce() -> (candid::Int, candid::Nat) {
    println!("generating nonce");
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let timestamp = candid::Int::from(now * 1_000_000);
    let out = pow(timestamp);
    println!("nonce generated");
    out
}

const DOMAIN: &str = "motoko-playground";

fn pow(timestamp: candid::Int) -> (candid::Int, candid::Nat) {
    let mut rng = rand::thread_rng();
    let mut nonce = candid::Nat::from(rng.gen::<i32>());
    let prefix = format!("{}{}", DOMAIN, timestamp);
    loop {
        let hash = motoko_hash(&format!("{}{}", prefix, nonce));
        if check_hash(hash) {
            return (timestamp, nonce);
        }
        nonce = nonce + 1;
    }
}

// djb2 hash function, from http://www.cse.yorku.ca/~oz/hash.html
fn motoko_hash(s: &str) -> i64 {
    let mut hash = 5381_u32;
    for c in s.chars() {
        let c_val = to_utf16_code_point(c);
        hash = hash
            .overflowing_mul(33)
            .0
            .overflowing_add(u32::from(c_val))
            .0;
    }
    return hash.into();
}

fn to_utf16_code_point(c: char) -> u16 {
    let mut b = [0; 2];
    let result = c.encode_utf16(&mut b);

    return result[0];
}

fn check_hash(hash: i64) -> bool {
    (hash & 0xc0000000) == 0
}
