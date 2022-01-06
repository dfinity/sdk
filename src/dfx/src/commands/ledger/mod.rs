use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{
    AccountIdBlob, BlockHeight, CyclesResponse, Memo, NotifyCanisterArgs, TimeStamp, TransferArgs,
    TransferError, TransferResult, MAINNET_CYCLE_MINTER_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::provider::create_agent_environment;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use anyhow::{anyhow, bail};
use candid::{Decode, Encode};
use clap::Clap;
use garcon::{Delay, Waiter};
use ic_agent::agent_error::HttpErrorPayload;
use ic_agent::{Agent, AgentError};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;

const TRANSFER_METHOD: &str = "transfer";
const NOTIFY_METHOD: &str = "notify_dfx";

mod account_id;
mod balance;
mod create_canister;
mod notify;
mod top_up;
mod transfer;

/// Ledger commands.
#[derive(Clap)]
#[clap(name("ledger"))]
pub struct LedgerOpts {
    /// Override the compute network to connect to. By default, the local network is used.
    #[clap(long)]
    network: Option<String>,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    AccountId(account_id::AccountIdOpts),
    Balance(balance::BalanceOpts),
    CreateCanister(create_canister::CreateCanisterOpts),
    Notify(notify::NotifyOpts),
    TopUp(top_up::TopUpOpts),
    Transfer(transfer::TransferOpts),
}

pub fn exec(env: &dyn Environment, opts: LedgerOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.clone())?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::AccountId(v) => account_id::exec(&agent_env, v).await,
            SubCommand::Balance(v) => balance::exec(&agent_env, v).await,
            SubCommand::CreateCanister(v) => create_canister::exec(&agent_env, v).await,
            SubCommand::Notify(v) => notify::exec(&agent_env, v).await,
            SubCommand::TopUp(v) => top_up::exec(&agent_env, v).await,
            SubCommand::Transfer(v) => transfer::exec(&agent_env, v).await,
        }
    })
}

fn get_icpts_from_args(
    amount: Option<String>,
    icp: Option<String>,
    e8s: Option<String>,
) -> DfxResult<ICPTs> {
    if amount.is_none() {
        let icp = match icp {
            Some(s) => {
                // validated by e8s_validator
                let icps = s.parse::<u64>().unwrap();
                ICPTs::from_icpts(icps).map_err(|err| anyhow!(err))?
            }
            None => ICPTs::from_e8s(0),
        };
        let icp_from_e8s = match e8s {
            Some(s) => {
                // validated by e8s_validator
                let e8s = s.parse::<u64>().unwrap();
                ICPTs::from_e8s(e8s)
            }
            None => ICPTs::from_e8s(0),
        };
        let amount = icp + icp_from_e8s;
        Ok(amount.map_err(|err| anyhow!(err))?)
    } else {
        Ok(ICPTs::from_str(&amount.unwrap())
            .map_err(|err| anyhow!("Could not add ICPs and e8s: {}", err))?)
    }
}

pub async fn transfer(
    agent: &Agent,
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

    // let mut n = 0;
    //
    let block_height: BlockHeight = loop {
        match agent
            .update(&MAINNET_LEDGER_CANISTER_ID, TRANSFER_METHOD)
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
                // eprintln!("transfer result: {:?}", &result);
                // n += 1;
                // if n < 2 && waiter.async_wait().await.is_ok() {
                //     eprintln!("force retry (no error)");
                //     continue;
                // }
                match result {
                    Ok(block_height) => break block_height,
                    Err(TransferError::TxDuplicate { duplicate_of }) => break duplicate_of,
                    Err(transfer_err) => bail!(transfer_err),
                }
            }
            Err(agent_err) if !retryable(&agent_err) => {
                // eprintln!("non-retryable error");
                bail!(agent_err);
            }
            Err(agent_err) => {
                // eprintln!("retryable error {:?}", &agent_err);
                if let Err(_waiter_err) = waiter.async_wait().await {
                    bail!(agent_err);
                }
            }
        }
    };

    Ok(block_height)
}

async fn transfer_and_notify(
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

    let block_height = transfer(agent, memo, amount, fee, to).await?;

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
