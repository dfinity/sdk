use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{
    CyclesResponse, NotifyCanisterArgs, MAINNET_CYCLE_MINTER_CANISTER_ID,
    MAINNET_LEDGER_CANISTER_ID,
};
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::util::clap::validators::{e8s_validator, icpts_amount_validator};

use crate::lib::root_key::fetch_root_key_if_needed;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::expiry_duration;

use anyhow::{anyhow, Context};
use candid::{Decode, Encode};
use clap::Parser;
use ic_types::principal::Principal;
use std::str::FromStr;

const NOTIFY_METHOD: &str = "notify_dfx";

/// Notify the ledger about a send transaction to the cycles minting canister.
/// This command should only be used if `dfx ledger create-canister` or `dfx ledger top-up`
/// successfully sent a message to the ledger, and a transaction was recorded at some block height, but
/// for some reason the subsequent notify failed.
#[derive(Parser)]
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

    let to_subaccount = Some(Subaccount::from(
        &Principal::from_text(opts.destination_principal)
            .context("Failed to parse destination principal.")?,
    ));

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env)
        .await
        .context("Failed to fetch root key.")?;

    let result = agent
        .update(&MAINNET_LEDGER_CANISTER_ID, NOTIFY_METHOD)
        .with_arg(
            Encode!(&NotifyCanisterArgs {
                block_height,
                max_fee,
                from_subaccount: None,
                to_canister: MAINNET_CYCLE_MINTER_CANISTER_ID,
                to_subaccount,
            })
            .context("Failed to encode notify arguments.")?,
        )
        .call_and_wait(waiter_with_timeout(expiry_duration()))
        .await
        .context("Failed notify call.")?;

    let result = Decode!(&result, CyclesResponse).context("Failed to decode notify response.")?;

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
