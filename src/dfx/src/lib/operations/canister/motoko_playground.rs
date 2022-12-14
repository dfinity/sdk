use std::time::SystemTime;

use anyhow::{bail, Context};
use candid::{encode_args, CandidType, Decode, Deserialize, Encode, Principal};
use fn_error_context::context;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use rand::Rng;
use slog::info;

use crate::{
    lib::{
        environment::Environment, error::DfxResult, identity::identity_utils::CallSender,
        models::canister_id_store::CanisterIdStore,
        network::network_descriptor::NetworkTypeDescriptor, waiter::waiter_with_timeout,
    },
    util::expiry_duration,
};

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
    _call_sender: &CallSender,
) -> DfxResult {
    //todo!(call sender)
    let agent = env.get_agent().context("Failed to get HTTP agent")?;
    let playground_cid = if let NetworkTypeDescriptor::Playground { playground_cid, .. } =
        env.get_network_descriptor().r#type
    {
        playground_cid
    } else {
        bail!("This shouldn't happen")
    };
    let mut canister_id_store = CanisterIdStore::for_env(env)?;
    let (timestamp, nonce) = create_nonce();
    let get_can_arg = Encode!(&GetCanisterIdArgs {
        timestamp: timestamp,
        nonce: nonce,
    })?;
    let result = agent
        .update(&playground_cid, "getCanisterId")
        .with_arg(get_can_arg)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await
        .context("Failed to reserve canister at the playground.")?;
    let reserved_canister = Decode!(&result, CanisterInfo)?;
    println!("Return value: {:?}", reserved_canister);
    canister_id_store.add(
        canister_name,
        &reserved_canister.id.to_string(),
        Some(reserved_canister.timestamp.into()),
    )?;

    info!(
        env.get_logger(),
        "Reserved canister '{}' with id {} with the playground.",
        canister_name,
        reserved_canister.id.to_string()
    );

    Ok(())
}

pub async fn _playground_install_code(
    env: &dyn Environment,
    canister_info: &CanisterInfo,
    arg: &[u8],
    wasm_module: &[u8],
    mode: InstallMode,
) -> DfxResult {
    //todo!(properly implement)

    println!("Trying to install a wasm:");
    let agent = env.get_agent().context("Failed to get HTTP agent")?;
    let playground_cid = if let NetworkTypeDescriptor::Playground { playground_cid, .. } =
        env.get_network_descriptor().r#type
    {
        playground_cid
    } else {
        bail!("This shouldn't happen")
    };
    let install_arg = InstallArgs {
        arg,
        wasm_module,
        mode,
        canister_id: canister_info.id.clone(),
    };
    let encoded_arg = encode_args((canister_info, install_arg, false))?;
    let result = agent
        .update(&playground_cid, "installCode")
        .with_arg(encoded_arg.as_slice())
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await
        .context("install failed")?;
    let out = Decode!(&result, CanisterInfo)?;
    println!("Install result: {:?}, principal: {}", &out, &out.id);
    Ok(())
}

fn create_nonce() -> (candid::Int, candid::Nat) {
    println!("generating nonce");
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let timestamp = candid::Int::from(now * 1_000_000);
    let out = proof_of_work(timestamp);
    println!("nonce generated");
    out
}

const DOMAIN: &str = "motoko-playground";

fn proof_of_work(timestamp: candid::Int) -> (candid::Int, candid::Nat) {
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
