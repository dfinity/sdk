use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::{AccountIdentifier, Subaccount};
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::nns_types::{
    BlockHeight, CyclesResponse, Memo, NotifyCanisterArgs, SendArgs, CYCLE_MINTER_CANISTER_ID,
    LEDGER_CANISTER_ID,
};
use crate::lib::provider::create_agent_environment;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use anyhow::anyhow;
use candid::{Decode, Encode};
use clap::Clap;
use ic_types::principal::Principal;
use std::str::FromStr;
use tokio::runtime::Runtime;

const SEND_METHOD: &str = "send_dfx";
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

async fn send_and_notify(
    env: &dyn Environment,
    memo: Memo,
    amount: ICPTs,
    fee: ICPTs,
    to_subaccount: Option<Subaccount>,
    max_fee: ICPTs,
) -> DfxResult<CyclesResponse> {
    let ledger_canister_id = Principal::from_text(LEDGER_CANISTER_ID)?;

    let cycle_minter_id = Principal::from_text(CYCLE_MINTER_CANISTER_ID)?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let to = AccountIdentifier::new(cycle_minter_id, to_subaccount);

    let result = agent
        .update(&ledger_canister_id, SEND_METHOD)
        .with_arg(Encode!(&SendArgs {
            memo,
            amount,
            fee,
            from_subaccount: None,
            to,
            created_at_time: None,
        })?)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await?;

    let block_height = Decode!(&result, BlockHeight)?;
    println!("Transfer sent at BlockHeight: {}", block_height);

    let result = agent
        .update(&ledger_canister_id, NOTIFY_METHOD)
        .with_arg(Encode!(&NotifyCanisterArgs {
            block_height,
            max_fee,
            from_subaccount: None,
            to_canister: cycle_minter_id,
            to_subaccount,
        })?)
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await?;

    let result = Decode!(&result, CyclesResponse)?;
    Ok(result)
}
