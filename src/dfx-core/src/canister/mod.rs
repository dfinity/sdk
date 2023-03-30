use candid::Principal;
use ic_agent::Agent;
use ic_utils::interfaces::WalletCanister;

use crate::error::canister::CanisterBuilderError;

pub async fn build_wallet_canister(
    id: Principal,
    agent: &Agent,
) -> Result<WalletCanister<'_>, CanisterBuilderError> {
    Ok(WalletCanister::from_canister(
        ic_utils::Canister::builder()
            .with_agent(agent)
            .with_canister_id(id)
            .build()
            .unwrap(),
    )
    .await
    .map_err(|e| CanisterBuilderError::WalletCanisterCaller(e))?)
}
