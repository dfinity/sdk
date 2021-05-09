use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::nns_types::{
    CyclesResponse, NotifyCanisterArgs, CYCLE_MINTER_CANISTER_ID, LEDGER_CANISTER_ID,
};
use crate::util::clap::validators::{e8s_validator, icpts_amount_validator};

use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use anyhow::anyhow;
use candid::{Decode, Encode};
use clap::Clap;
use ic_types::principal::Principal;
use std::str::FromStr;

const NOTIFY_METHOD: &str = "notify_dfx";

/// Notify the ledger about a send transaction to the cycles minting canister.
/// This command should only be used if `dfx ledger create-canister` or `dfx ledger top-up`
/// successfully sent a message to the ledger, and a transaction was recorded at some block height, but
/// for some reason the subsequent notify failed.
#[derive(Clap)]
pub struct NotifyOpts {
    /// BlockHeight at which the send transation was recorded.
    #[clap(validator(e8s_validator))]
    block_height: String,

    /// Specify the principal of the destination, either a canister id or a user principal.
    /// If the send transaction was for `create-canister`, specify the `controller` here.
    /// If the send transacction was for `top-up`, specify the `canister` here.
    destination_principal: String,

    /// Max fee, default is 10000 e8s.
    #[clap(long, validator(icpts_amount_validator))]
    max_fee: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: NotifyOpts) -> DfxResult {
    // validated by block_height validator
    let block_height = opts.block_height.parse::<u64>().unwrap();

    let max_fee = opts
        .max_fee
        .map_or(Ok(TRANSACTION_FEE), |v| ICPTs::from_str(&v))
        .map_err(|err| anyhow!(err))?;

    let ledger_canister_id = Principal::from_text(LEDGER_CANISTER_ID)?;
    let cycle_minter_id = Principal::from_text(CYCLE_MINTER_CANISTER_ID)?;

    let to_subaccount = Some(Subaccount::from(&Principal::from_text(
        opts.destination_principal,
    )?));

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

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

    match result {
        CyclesResponse::ToppedUp(()) => {
            println!("Canister was topped up!");
        }
        CyclesResponse::Refunded(msg, maybe_block_height) => {
            match maybe_block_height {
                Some(height) => {
                    println!("Refunded at block height {} with message :{}", height, msg)
                }
                None => println!("Refunded with message: {}", msg),
            };
        }
        CyclesResponse::CanisterCreated(v) => {
            println!("Canister created with id: {:?}", v.to_text());
        }
    };

    Ok(())
}
