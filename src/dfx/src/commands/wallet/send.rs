use crate::commands::wallet::get_wallet;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::waiter::waiter_with_timeout;
use crate::util::clap::validators::cycle_amount_validator;
use crate::util::expiry_duration;

use anyhow::{anyhow, Context};
use candid::CandidType;
use candid::Principal;
use clap::Parser;

/// Send cycles to another cycles wallet.
#[derive(Parser)]
pub struct SendOpts {
    /// Canister ID of the destination cycles wallet.
    destination: String,

    /// Specifies the amount of cycles to send.
    /// Deducted from the wallet.
    #[clap(validator(cycle_amount_validator))]
    amount: String,
}

pub async fn exec(env: &dyn Environment, opts: SendOpts) -> DfxResult {
    #[derive(CandidType)]
    struct In {
        canister: Principal,
        amount: u128,
    }
    let canister = Principal::from_text(&opts.destination).with_context(|| {
        format!(
            "Failed to parse {:?} as destination principal.",
            &opts.destination
        )
    })?;
    // amount has been validated by cycle_amount_validator
    let amount = opts.amount.parse::<u128>().unwrap();
    let res = get_wallet(env)
        .await?
        .wallet_send(canister, amount, waiter_with_timeout(expiry_duration()))
        .await;
    res.map_err(|err| {
        anyhow!(
            "Sending cycles to {} failed with: {}",
            opts.destination,
            err
        )
    })
}
