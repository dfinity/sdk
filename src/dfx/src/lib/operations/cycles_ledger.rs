use crate::lib::error::DfxResult;
use crate::lib::retryable::retryable;
use anyhow::anyhow;
use backoff::future::retry;
use backoff::ExponentialBackoff;
use candid::Principal;
use ic_agent::Agent;
use ic_utils::call::SyncCall;
use ic_utils::Canister;
use icrc_ledger_types::icrc1;

const ICRC1_BALANCE_OF_METHOD: &str = "icrc1_balance_of";

pub async fn balance(
    agent: &Agent,
    owner: Principal,
    subaccount: Option<icrc1::account::Subaccount>,
    cycles_ledger_canister_id: Principal,
) -> DfxResult<u128> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(cycles_ledger_canister_id)
        .build()?;
    let arg = icrc1::account::Account { owner, subaccount };

    let retry_policy = ExponentialBackoff::default();

    retry(retry_policy, || async {
        let result = canister
            .query(ICRC1_BALANCE_OF_METHOD)
            .with_arg(arg)
            .build()
            .call()
            .await;
        match result {
            Ok((balance,)) => Ok(balance),
            Err(agent_err) if retryable(&agent_err) => {
                Err(backoff::Error::transient(anyhow!(agent_err)))
            }
            Err(agent_err) => Err(backoff::Error::permanent(anyhow!(agent_err))),
        }
    })
    .await
}
