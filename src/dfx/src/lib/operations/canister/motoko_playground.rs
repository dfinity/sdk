use crate::lib::{environment::Environment, error::DfxResult};
use anyhow::{bail, Context};
use candid::{encode_args, CandidType, Decode, Deserialize, Encode, Principal};
use dfx_core::config::model::canister_id_store::AcquisitionDateTime;
use dfx_core::config::model::network_descriptor::{
    NetworkTypeDescriptor, MAINNET_MOTOKO_PLAYGROUND_CANISTER_ID,
};
use fn_error_context::context;
use ic_utils::interfaces::management_canister::builders::InstallMode;
use num_traits::ToPrimitive;
use rand::Rng;
use slog::{debug, info};
use std::time::SystemTime;

/// Arguments for the `getCanisterId` call.
#[derive(CandidType)]
pub struct GetCanisterIdArgs {
    pub timestamp: candid::Int,
    pub nonce: candid::Nat,
}

/// Used to uniquely identify a canister with the playground
#[derive(CandidType, Deserialize, Debug)]
pub struct CanisterInfo {
    pub id: Principal,
    pub timestamp: candid::Int,
}

impl CanisterInfo {
    pub fn from(id: Principal, timestamp: &AcquisitionDateTime) -> Self {
        let timestamp = candid::Int::from(timestamp.unix_timestamp_nanos());
        Self { id, timestamp }
    }

    #[context("Failed to get timestamp from CanisterInfo")]
    pub fn get_timestamp(&self) -> DfxResult<AcquisitionDateTime> {
        AcquisitionDateTime::from_unix_timestamp_nanos(
            self.timestamp.0.to_i128().context("i128 overflow")?,
        )
        .context("Failed to make unix timestamp from nanos")
    }
}

#[derive(CandidType, Deserialize, Debug)]
pub struct InstallArgs<'a> {
    pub arg: &'a [u8],
    pub wasm_module: &'a [u8],
    pub mode: PlaygroundInstallMode,
    pub canister_id: Principal,
}

#[derive(CandidType, Deserialize, Debug)]
pub enum PlaygroundInstallMode {
    #[serde(rename = "install")]
    Install,
    #[serde(rename = "upgrade")]
    Upgrade,
    #[serde(rename = "reinstall")]
    Reinstall,
}

impl TryFrom<InstallMode> for PlaygroundInstallMode {
    type Error = anyhow::Error;
    fn try_from(m: InstallMode) -> DfxResult<Self> {
        match m {
            InstallMode::Install => Ok(Self::Install),
            InstallMode::Reinstall => Ok(Self::Reinstall),
            InstallMode::Upgrade {
                skip_pre_upgrade: false,
            } => Ok(Self::Upgrade),
            InstallMode::Upgrade {
                skip_pre_upgrade: true,
            } => bail!("Cannot skip pre-upgrade on the playground"),
        }
    }
}

#[derive(CandidType)]
struct InstallConfig<'a> {
    profiling: bool,
    is_whitelisted: bool,
    origin: Origin<'a>,
}
#[derive(CandidType)]
struct Origin<'a> {
    origin: &'a str,
    tags: &'a [&'a str],
}
impl<'a> Origin<'a> {
    fn new() -> Self {
        Self {
            origin: "dfx",
            tags: &[],
        }
    }
}

#[context("Failed to reserve canister '{}'.", canister_name)]
pub async fn reserve_canister_with_playground(
    env: &dyn Environment,
    canister_name: &str,
) -> DfxResult {
    let agent = env.get_agent();
    let log = env.get_logger();
    let playground_canister = if let NetworkTypeDescriptor::Playground {
        playground_canister,
        ..
    } = env.get_network_descriptor().r#type
    {
        debug!(log, "playground canister is {}", playground_canister);
        playground_canister
    } else {
        bail!("Trying to reserve canister with playground on non-playground network.")
    };
    if ci_info::is_ci() && playground_canister == MAINNET_MOTOKO_PLAYGROUND_CANISTER_ID {
        bail!("Cannot reserve playground canister in CI, please run `dfx start` to use the local replica.")
    }

    let mut canister_id_store = env.get_canister_id_store()?;
    let (timestamp, nonce) = create_nonce();
    let get_can_arg = Encode!(&GetCanisterIdArgs { timestamp, nonce }, &Origin::new())?;
    let result = agent
        .update(&playground_canister, "getCanisterId")
        .with_arg(get_can_arg)
        .call_and_wait()
        .await
        .context("Failed to reserve canister at the playground.")?;
    let reserved_canister = Decode!(&result, CanisterInfo)?;
    canister_id_store.add(
        canister_name,
        &reserved_canister.id.to_string(),
        Some(reserved_canister.get_timestamp()?),
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
    canister_timestamp: &AcquisitionDateTime,
    principal_to_authorize: &Principal,
) -> DfxResult {
    let agent = env.get_agent();
    let playground_canister = if let NetworkTypeDescriptor::Playground {
        playground_canister,
        ..
    } = env.get_network_descriptor().r#type
    {
        playground_canister
    } else {
        bail!("Trying to authorize asset uploader on non-playground network.")
    };
    let canister_info = CanisterInfo::from(canister_id, canister_timestamp);

    let nested_arg = Encode!(&principal_to_authorize)?;
    let call_arg = Encode!(&canister_info, &"authorize", &nested_arg)?;

    let _ = agent
        .update(&playground_canister, "callForward")
        .with_arg(call_arg)
        .call_and_wait()
        .await
        .context("Failed to call playground.")?;
    Ok(())
}

pub async fn playground_install_code(
    env: &dyn Environment,
    canister_id: Principal,
    canister_timestamp: &AcquisitionDateTime,
    arg: &[u8],
    wasm_module: &[u8],
    mode: InstallMode,
    is_asset_canister: bool,
) -> DfxResult<AcquisitionDateTime> {
    let canister_info = CanisterInfo::from(canister_id, canister_timestamp);
    let agent = env.get_agent();
    let playground_canister = match env.get_network_descriptor().r#type {
        NetworkTypeDescriptor::Playground {
            playground_canister,
            ..
        } => playground_canister,
        _ => bail!("Trying to install wasm through playground on non-playground network."),
    };
    let install_arg = InstallArgs {
        arg,
        wasm_module,
        mode: mode.try_into()?,
        canister_id: canister_info.id,
    };
    let install_config = InstallConfig {
        profiling: false,
        is_whitelisted: is_asset_canister,
        origin: Origin::new(),
    };
    let encoded_arg = encode_args((canister_info, install_arg, install_config))?;
    let result = agent
        .update(&playground_canister, "installCode")
        .with_arg(encoded_arg.as_slice())
        .call_and_wait()
        .await
        .context("install failed")?;
    let out = Decode!(&result, CanisterInfo)?;
    out.get_timestamp()
}

fn create_nonce() -> (candid::Int, candid::Nat) {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let timestamp = candid::Int::from(now);
    let mut rng = rand::thread_rng();
    let mut nonce = candid::Nat::from(rng.gen::<u32>());
    let prefix = format!("{}{}", POW_DOMAIN, timestamp);
    loop {
        let to_hash = format!("{}{}", prefix, nonce).replace('_', "");
        let hash = motoko_hash(&to_hash);
        if (hash & 0xc0000000) == 0 {
            return (timestamp, nonce);
        }
        nonce += 1_u8;
    }
}

const POW_DOMAIN: &str = "motoko-playground";

// djb2 hash function, from http://www.cse.yorku.ca/~oz/hash.html
fn motoko_hash(s: &str) -> i64 {
    fn to_utf16_code_point(c: char) -> u16 {
        let mut b = [0; 2];
        let result = c.encode_utf16(&mut b);

        result[0]
    }

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
