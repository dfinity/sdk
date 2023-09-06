use crate::lib::cycles_ledger_types;
use crate::lib::cycles_ledger_types::send::SendError;
use crate::lib::error::DfxResult;
use crate::lib::retryable::retryable;
use anyhow::anyhow;
use backoff::future::retry;
use backoff::ExponentialBackoff;
use candid::{Nat, Principal};
use ic_agent::Agent;
use ic_utils::call::SyncCall;
use ic_utils::Canister;
use icrc_ledger_types::icrc1;
use icrc_ledger_types::icrc1::transfer::{BlockIndex, TransferError};

const ICRC1_BALANCE_OF_METHOD: &str = "icrc1_balance_of";
const ICRC1_TRANSFER_METHOD: &str = "icrc1_transfer";
const SEND_METHOD: &str = "send";

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

pub async fn transfer(
    agent: &Agent,
    amount: u128,
    from_subaccount: Option<icrc1::account::Subaccount>,
    owner: Principal,
    to_subaccount: Option<icrc1::account::Subaccount>,
    created_at_time: u64,
    fee: Option<u128>,
    memo: Option<u64>,
    cycles_ledger_canister_id: Principal,
) -> DfxResult<BlockIndex> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(cycles_ledger_canister_id)
        .build()?;

    let retry_policy = ExponentialBackoff::default();

    let block_index = retry(retry_policy, || async {
        let arg = icrc1::transfer::TransferArg {
            from_subaccount,
            to: icrc1::account::Account {
                owner,
                subaccount: to_subaccount,
            },
            fee: fee.map(Nat::from),
            created_at_time: Some(created_at_time),
            memo: memo.map(|v| v.into()),
            amount: Nat::from(amount),
        };
        match canister
            .update(ICRC1_TRANSFER_METHOD)
            .with_arg(arg)
            .build()
            .map(|result: (Result<BlockIndex, TransferError>,)| (result.0,))
            .call_and_wait()
            .await
            .map(|(result,)| result)
        {
            Ok(Ok(block_index)) => Ok(block_index),
            Ok(Err(TransferError::Duplicate { duplicate_of })) => {
                println!(
                    "{}",
                    TransferError::Duplicate {
                        duplicate_of: duplicate_of.clone()
                    }
                );
                Ok(duplicate_of)
            }
            Ok(Err(transfer_err)) => Err(backoff::Error::permanent(anyhow!(transfer_err))),
            Err(agent_err) if retryable(&agent_err) => {
                Err(backoff::Error::transient(anyhow!(agent_err)))
            }
            Err(agent_err) => Err(backoff::Error::permanent(anyhow!(agent_err))),
        }
    })
    .await?;

    Ok(block_index)
}

pub async fn send(
    agent: &Agent,
    to: Principal,
    amount: u128,
    created_at_time: u64,
    memo: Option<u64>,
    from_subaccount: Option<icrc1::account::Subaccount>,
    cycles_ledger_canister_id: Principal,
) -> DfxResult<BlockIndex> {
    let canister = Canister::builder()
        .with_agent(agent)
        .with_canister_id(cycles_ledger_canister_id)
        .build()?;

    let retry_policy = ExponentialBackoff::default();
    let block_index: BlockIndex = retry(retry_policy, || async {
        let arg = cycles_ledger_types::send::SendArgs {
            from_subaccount,
            to,
            created_at_time: Some(created_at_time),
            memo: memo.map(|v| v.into()),
            amount: Nat::from(amount),
        };
        match canister
            .update(SEND_METHOD)
            .with_arg(arg)
            .build()
            .map(|result: (Result<BlockIndex, SendError>,)| (result.0,))
            .call_and_wait()
            .await
            .map(|(result,)| result)
        {
            Ok(Ok(block_index)) => Ok(block_index),
            Ok(Err(SendError::Duplicate { duplicate_of })) => {
                println!(
                    "transaction is a duplicate of another transaction in block {}",
                    duplicate_of
                );
                Ok(duplicate_of)
            }
            Ok(Err(SendError::InvalidReceiver { receiver })) => {
                Err(backoff::Error::permanent(anyhow!(
                    "Invalid receiver: {}.  Make sure the receiver is a canister.",
                    receiver
                )))
            }
            Ok(Err(send_err)) => Err(backoff::Error::permanent(anyhow!(
                "send error: {:?}",
                send_err
            ))),
            Err(agent_err) if retryable(&agent_err) => {
                Err(backoff::Error::transient(anyhow!(agent_err)))
            }
            Err(agent_err) => Err(backoff::Error::permanent(anyhow!(agent_err))),
        }
    })
    .await?;

    Ok(block_index)
}
