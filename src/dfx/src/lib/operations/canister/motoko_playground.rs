use dfx_core::config::model::network_descriptor::NetworkTypeDescriptor;
use std::time::SystemTime;

use anyhow::{bail, Context};
use candid::{encode_args, CandidType, Decode, Deserialize, Encode, Principal};
use fn_error_context::context;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use rand::Rng;
use slog::{debug, info};

use crate::lib::{environment::Environment, error::DfxResult, wasm::file::is_wasm_module};

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

#[context("Failed to reserve canister '{}'.", canister_name)]
pub async fn reserve_canister_with_playground(
    env: &dyn Environment,
    canister_name: &str,
) -> DfxResult {
    let agent = env.get_agent().context("Failed to get HTTP agent")?;
    let log = env.get_logger();
    let playground_cid = if let NetworkTypeDescriptor::Playground { playground_cid, .. } =
        env.get_network_descriptor().r#type
    {
        debug!(log, "playground canister is {}", playground_cid);
        playground_cid
    } else {
        bail!("Trying to reserve canister with playground on non-playground network.")
    };
    let mut canister_id_store = env.get_canister_id_store()?;
    let (timestamp, nonce) = create_nonce();
    let get_can_arg = Encode!(&GetCanisterIdArgs { timestamp, nonce })?;
    let result = agent
        .update(&playground_cid, "getCanisterId")
        .with_arg(get_can_arg)
        .call_and_wait()
        .await
        .context("Failed to reserve canister at the playground.")?;
    let reserved_canister = Decode!(&result, CanisterInfo)?;
    canister_id_store.add(
        canister_name,
        &reserved_canister.id.to_string(),
        Some(reserved_canister.timestamp.into()),
    )?;

    info!(
        env.get_logger(),
        "Reserved canister '{}' with id {} with the playground.",
        canister_name,
        reserved_canister.id
    );

    Ok(())
}

#[context("Failed to authorize asset uploader through playground.")]
pub async fn authorize_asset_uploader(
    env: &dyn Environment,
    canister_id: Principal,
    canister_timestamp: candid::Int,
    principal_to_authorize: &Principal,
) -> DfxResult {
    let agent = env.get_agent().context("Failed to get HTTP agent")?;
    let playground_cid = if let NetworkTypeDescriptor::Playground { playground_cid, .. } =
        env.get_network_descriptor().r#type
    {
        playground_cid
    } else {
        bail!("Trying to authorize asset uploader on non-playground network.")
    };
    let canister_info = CanisterInfo {
        id: canister_id,
        timestamp: canister_timestamp,
    };

    let nested_arg = Encode!(&principal_to_authorize)?;
    let call_arg = Encode!(&canister_info, &"authorize", &nested_arg)?;

    let _ = agent
        .update(&playground_cid, "callForward")
        .with_arg(call_arg)
        .call_and_wait()
        .await
        .context("Failed to call playground.")?;
    Ok(())
}

pub async fn playground_install_code(
    env: &dyn Environment,
    canister_id: Principal,
    canister_timestamp: candid::Int,
    arg: &[u8],
    wasm_module: &[u8],
    mode: InstallMode,
) -> DfxResult<num_bigint::BigInt> {
    if !is_wasm_module(wasm_module) {
        bail!("Invalid WASM detected. Is your file maybe zipped? Playground can only deploy .wasm files.");
    }
    let canister_info = CanisterInfo {
        id: canister_id,
        timestamp: canister_timestamp,
    };
    let agent = env.get_agent().context("Failed to get HTTP agent")?;
    let playground_cid = if let NetworkTypeDescriptor::Playground { playground_cid, .. } =
        env.get_network_descriptor().r#type
    {
        playground_cid
    } else {
        bail!("Trying to install wasm through playground on non-playground network.")
    };
    let install_arg = InstallArgs {
        arg,
        wasm_module,
        mode,
        canister_id: canister_info.id,
    };
    let encoded_arg = encode_args((canister_info, install_arg, false))?;
    let result = agent
        .update(&playground_cid, "installCode")
        .with_arg(encoded_arg.as_slice())
        .call_and_wait()
        .await
        .context("install failed")?;
    let out = Decode!(&result, CanisterInfo)?;
    let refreshed_timestamp = out.timestamp;
    Ok(refreshed_timestamp.into())
}

fn create_nonce() -> (candid::Int, candid::Nat) {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let timestamp = candid::Int::from(now * 1_000_000);
    proof_of_work(timestamp)
}

const POW_DOMAIN: &str = "motoko-playground";

fn proof_of_work(timestamp: candid::Int) -> (candid::Int, candid::Nat) {
    let mut rng = rand::thread_rng();
    let mut nonce = candid::Nat::from(rng.gen::<i32>());
    let prefix = format!("{}{}", POW_DOMAIN, timestamp);
    loop {
        let to_hash = format!("{}{}", prefix, nonce).replace('_', "");
        let hash = motoko_hash(&to_hash);
        if check_hash(hash) {
            return (timestamp, nonce);
        }
        nonce += 1;
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
    hash.into()
}

fn to_utf16_code_point(c: char) -> u16 {
    let mut b = [0; 2];
    let result = c.encode_utf16(&mut b);

    result[0]
}

fn check_hash(hash: i64) -> bool {
    (hash & 0xc0000000) == 0
}
