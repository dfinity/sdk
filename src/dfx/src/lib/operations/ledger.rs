use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, bail};
use candid::{Decode, Encode, Principal};
use garcon::{Delay, Waiter};
use ic_agent::{agent_error::HttpErrorPayload, Agent, AgentError};
use ic_utils::{call::SyncCall, Canister};

use crate::{
    lib::{
        environment::Environment,
        error::DfxResult,
        ledger_types::{
            AccountBalanceArgs, AccountIdBlob, BlockHeight, CyclesResponse,
            IcpXdrConversionRateCertifiedResponse, Memo, NotifyCanisterArgs, TimeStamp,
            TransferArgs, TransferError, TransferResult, MAINNET_CYCLE_MINTER_CANISTER_ID,
            MAINNET_LEDGER_CANISTER_ID,
        },
        nns_types::{
            account_identifier::{AccountIdentifier, Subaccount},
            icpts::ICPTs,
        },
        root_key::fetch_root_key_if_needed,
        waiter::waiter_with_timeout,
    },
    util::expiry_duration,
};

const TRANSFER_METHOD: &str = "transfer";
const NOTIFY_METHOD: &str = "notify_dfx";
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

pub async fn transfer(
    agent: &Agent,
    canister_id: &Principal,
    memo: Memo,
    amount: ICPTs,
    fee: ICPTs,
    to: AccountIdBlob,
) -> DfxResult<BlockHeight> {
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    let mut waiter = Delay::builder()
        .with(Delay::count_timeout(30))
        .exponential_backoff_capped(
            std::time::Duration::from_secs(1),
            2.0,
            std::time::Duration::from_secs(16),
        )
        .build();
    waiter.start();

    let block_height: BlockHeight = loop {
        match agent
            .update(canister_id, TRANSFER_METHOD)
            .with_arg(Encode!(&TransferArgs {
                memo,
                amount,
                fee,
                from_subaccount: None,
                to,
                created_at_time: Some(TimeStamp { timestamp_nanos }),
            })?)
            .call_and_wait(waiter_with_timeout(expiry_duration()))
            .await
        {
            Ok(data) => {
                let result = Decode!(&data, TransferResult)?;
                match result {
                    Ok(block_height) => break block_height,
                    Err(TransferError::TxDuplicate { duplicate_of }) => break duplicate_of,
                    Err(transfer_err) => bail!(transfer_err),
                }
            }
            Err(agent_err) if !retryable(&agent_err) => {
                bail!(agent_err);
            }
            Err(agent_err) => {
                eprintln!("Waiting to retry after error: {:?}", &agent_err);
                if let Err(_waiter_err) = waiter.async_wait().await {
                    bail!(agent_err);
                }
            }
        }
    };

    Ok(block_height)
}

pub async fn transfer_and_notify(
    env: &dyn Environment,
    memo: Memo,
    amount: ICPTs,
    fee: ICPTs,
    to_subaccount: Option<Subaccount>,
    max_fee: ICPTs,
) -> DfxResult<CyclesResponse> {
    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let to = AccountIdentifier::new(MAINNET_CYCLE_MINTER_CANISTER_ID, to_subaccount).to_address();

    let block_height = transfer(agent, &MAINNET_LEDGER_CANISTER_ID, memo, amount, fee, to).await?;

    println!("Transfer sent at BlockHeight: {}", block_height);

    let result = agent
        .update(&MAINNET_LEDGER_CANISTER_ID, NOTIFY_METHOD)
        .with_arg(Encode!(&NotifyCanisterArgs {
            block_height,
            max_fee,
            from_subaccount: None,
            to_canister: MAINNET_CYCLE_MINTER_CANISTER_ID,
            to_subaccount,
        })?)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await?;

    let result = Decode!(&result, CyclesResponse)?;
    Ok(result)
}

fn retryable(agent_error: &AgentError) -> bool {
    match agent_error {
        AgentError::ReplicaError {
            reject_code,
            reject_message,
        } if *reject_code == 5 && reject_message.contains("is out of cycles") => false,
        AgentError::HttpError(HttpErrorPayload {
            status,
            content_type: _,
            content: _,
        }) if *status == 403 => {
            // sometimes out of cycles looks like this
            // assume any 403(unauthorized) is not retryable
            false
        }
        _ => true,
    }
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
