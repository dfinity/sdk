use crate::lib::error::DfxResult;
use candid::{CandidType, Principal};
use ic_agent::Agent;
use ic_utils::Canister;

pub const NNS_MIGRATION_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x11, 0x01, 0x01]);
const MIGRATE_CANISTER_METHOD: &str = "migrate_canister";
// const MIGRATE_STATUS_METHOD: &str = "migrate_status";

#[derive(CandidType)]
pub struct MigrateCanisterArg {
    pub from_canister: Principal,
    pub to_canister: Principal,
}

pub async fn migrate_canister(
    agent: &Agent,
    from_canister: Principal,
    to_canister: Principal,
) -> DfxResult {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(NNS_MIGRATION_CANISTER_ID)
        .build()?;

    let arg = MigrateCanisterArg {
        from_canister,
        to_canister,
    };

    let _: () = canister
        .update(MIGRATE_CANISTER_METHOD)
        .with_arg(arg)
        .build()
        .await?;

    Ok(())
}
