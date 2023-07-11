use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{
    BlockHeight, Memo, NotifyCreateCanisterArg, NotifyCreateCanisterResult, NotifyTopUpArg,
    NotifyTopUpResult, MAINNET_CYCLE_MINTER_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::operations::ledger::transfer;
use anyhow::Context;
use candid::{Decode, Encode, Principal};
use ic_agent::Agent;

const NOTIFY_CREATE_METHOD: &str = "notify_create_canister";
const NOTIFY_TOP_UP_METHOD: &str = "notify_top_up";

pub async fn transfer_cmc(
    agent: &Agent,
    memo: Memo,
    amount: ICPTs,
    fee: ICPTs,
    from_subaccount: Option<Subaccount>,
    to_principal: Principal,
    created_at_time: Option<u64>,
) -> DfxResult<BlockHeight> {
    let to_subaccount = Subaccount::from(&to_principal);
    let to =
        AccountIdentifier::new(MAINNET_CYCLE_MINTER_CANISTER_ID, Some(to_subaccount)).to_address();
    transfer(
        agent,
        &MAINNET_LEDGER_CANISTER_ID,
        memo,
        amount,
        fee,
        from_subaccount,
        to,
        created_at_time,
    )
    .await
}

pub async fn notify_create(
    agent: &Agent,
    controller: Principal,
    block_height: BlockHeight,
    subnet_type: Option<String>,
) -> DfxResult<NotifyCreateCanisterResult> {
    let result = agent
        .update(&MAINNET_CYCLE_MINTER_CANISTER_ID, NOTIFY_CREATE_METHOD)
        .with_arg(
            Encode!(&NotifyCreateCanisterArg {
                block_index: block_height,
                controller,
                subnet_type,
            })
            .context("Failed to encode notify arguments.")?,
        )
        .call_and_wait()
        .await
        .context("Notify call failed.")?;
    let result =
        Decode!(&result, NotifyCreateCanisterResult).context("Failed to decode notify response")?;
    Ok(result)
}

pub async fn notify_top_up(
    agent: &Agent,
    canister: Principal,
    block_height: BlockHeight,
) -> DfxResult<NotifyTopUpResult> {
    let result = agent
        .update(&MAINNET_CYCLE_MINTER_CANISTER_ID, NOTIFY_TOP_UP_METHOD)
        .with_arg(
            Encode!(&NotifyTopUpArg {
                block_index: block_height,
                canister_id: canister,
            })
            .context("Failed to encode notify arguments.")?,
        )
        .call_and_wait()
        .await
        .context("Notify call failed.")?;
    let result = Decode!(&result, NotifyTopUpResult).context("Failed to decode notify response")?;
    Ok(result)
}
