use candid::Principal;
use ic_agent::Agent;
use ic_utils::{call::SyncCall, Canister};

use crate::lib::{
    error::DfxResult,
    ledger_types::{
        AccountBalanceArgs, IcpXdrConversionRateCertifiedResponse,
        MAINNET_CYCLE_MINTER_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
    },
    nns_types::{account_identifier::AccountIdentifier, icpts::ICPTs},
};

const ACCOUNT_BALANCE_METHOD: &str = "account_balance_dfx";

pub async fn balance(
    agent: &Agent,
    acct: &AccountIdentifier,
    ledger_canister_id: Option<Principal>,
) -> DfxResult<ICPTs> {
    let canister_id = ledger_canister_id.unwrap_or(MAINNET_LEDGER_CANISTER_ID);
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(canister_id)
        .build()?;
    let (result,) = canister
        .query_(ACCOUNT_BALANCE_METHOD)
        .with_arg(AccountBalanceArgs {
            account: acct.to_string(),
        })
        .build()
        .call()
        .await?;
    Ok(result)
}

pub async fn icp_xdr_rate(agent: &Agent) -> DfxResult<u64> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(MAINNET_CYCLE_MINTER_CANISTER_ID)
        .build()?;
    let (certified_rate,): (IcpXdrConversionRateCertifiedResponse,) = canister
        .query_("get_icp_xdr_conversion_rate")
        .build()
        .call()
        .await?;
    //todo check certificate
    Ok(certified_rate.data.xdr_permyriad_per_icp)
}
