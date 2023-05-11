use crate::lib::agent::create_agent_environment;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{
    AccountIdBlob, BlockHeight, Memo, NotifyCreateCanisterArg, NotifyCreateCanisterResult,
    NotifyTopUpArg, NotifyTopUpResult, TimeStamp, TransferArgs, TransferError, TransferResult,
    MAINNET_CYCLE_MINTER_CANISTER_ID, MAINNET_LEDGER_CANISTER_ID,
};
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::ICPTs;
use crate::NetworkOpt;

use anyhow::{anyhow, bail, Context};
use backoff::backoff::Backoff;
use backoff::ExponentialBackoff;
use candid::Principal;
use candid::{Decode, Encode};
use clap::Parser;
use fn_error_context::context;
use ic_agent::agent_error::HttpErrorPayload;
use ic_agent::{Agent, AgentError};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;

const TRANSFER_METHOD: &str = "transfer";
const NOTIFY_TOP_UP_METHOD: &str = "notify_top_up";
const NOTIFY_CREATE_METHOD: &str = "notify_create_canister";

mod account_id;
mod balance;
pub mod create_canister;
mod fabricate_cycles;
mod notify;
pub mod show_subnet_types;
mod top_up;
mod transfer;

/// Ledger commands.
#[derive(Parser)]
#[command(name = "ledger")]
pub struct LedgerOpts {
    #[command(flatten)]
    network: NetworkOpt,

    #[command(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    AccountId(account_id::AccountIdOpts),
    Balance(balance::BalanceOpts),
    CreateCanister(create_canister::CreateCanisterOpts),
    FabricateCycles(fabricate_cycles::FabricateCyclesOpts),
    Notify(notify::NotifyOpts),
    ShowSubnetTypes(show_subnet_types::ShowSubnetTypesOpts),
    TopUp(top_up::TopUpOpts),
    Transfer(transfer::TransferOpts),
}

pub fn exec(env: &dyn Environment, opts: LedgerOpts) -> DfxResult {
    let agent_env = create_agent_environment(env, opts.network.network)?;
    let runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(async {
        match opts.subcmd {
            SubCommand::AccountId(v) => account_id::exec(&agent_env, v).await,
            SubCommand::Balance(v) => balance::exec(&agent_env, v).await,
            SubCommand::CreateCanister(v) => create_canister::exec(&agent_env, v).await,
            SubCommand::FabricateCycles(v) => fabricate_cycles::exec(&agent_env, v).await,
            SubCommand::Notify(v) => notify::exec(&agent_env, v).await,
            SubCommand::ShowSubnetTypes(v) => show_subnet_types::exec(&agent_env, v).await,
            SubCommand::TopUp(v) => top_up::exec(&agent_env, v).await,
            SubCommand::Transfer(v) => transfer::exec(&agent_env, v).await,
        }
    })
}

#[context("Failed to determine icp amount from supplied arguments.")]
fn get_icpts_from_args(
    amount: Option<ICPTs>,
    icp: Option<u64>,
    e8s: Option<u64>,
) -> DfxResult<ICPTs> {
    match amount {
        None => {
            let icp = match icp {
                Some(icps) => ICPTs::from_icpts(icps).map_err(|err| anyhow!(err))?,
                None => ICPTs::from_e8s(0),
            };
            let icp_from_e8s = match e8s {
                Some(e8s) => ICPTs::from_e8s(e8s),
                None => ICPTs::from_e8s(0),
            };
            let amount = icp + icp_from_e8s;
            Ok(amount.map_err(|err| anyhow!(err))?)
        }
        Some(amount) => Ok(amount),
    }
}

#[context("Failed to transfer funds.")]
pub async fn transfer(
    agent: &Agent,
    canister_id: &Principal,
    memo: Memo,
    amount: ICPTs,
    fee: ICPTs,
    from_subaccount: Option<Subaccount>,
    to: AccountIdBlob,
    created_at_time: Option<u64>,
) -> DfxResult<BlockHeight> {
    let timestamp_nanos = created_at_time.unwrap_or(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64,
    );

    let mut retry_policy = ExponentialBackoff::default();

    let block_height: BlockHeight = loop {
        match agent
            .update(canister_id, TRANSFER_METHOD)
            .with_arg(
                Encode!(&TransferArgs {
                    memo,
                    amount,
                    fee,
                    from_subaccount,
                    to,
                    created_at_time: Some(TimeStamp { timestamp_nanos }),
                })
                .context("Failed to encode arguments.")?,
            )
            .call_and_wait()
            .await
        {
            Ok(data) => {
                let result = Decode!(&data, TransferResult)
                    .context("Failed to decode transfer response.")?;
                match result {
                    Ok(block_height) => break block_height,
                    Err(TransferError::TxDuplicate { duplicate_of }) => {
                        println!("{}", TransferError::TxDuplicate { duplicate_of });
                        break duplicate_of;
                    }
                    Err(transfer_err) => bail!(transfer_err),
                }
            }
            Err(agent_err) if !retryable(&agent_err) => {
                bail!(agent_err);
            }
            Err(agent_err) => match retry_policy.next_backoff() {
                Some(duration) => {
                    eprintln!("Waiting to retry after error: {:?}", &agent_err);
                    tokio::time::sleep(duration).await;
                    println!("Sending duplicate transaction");
                }
                None => bail!(agent_err),
            },
        }
    };

    println!("Transfer sent at block height {block_height}");

    Ok(block_height)
}

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
