use crate::commands::wallet::get_wallet;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::util::clap::parsers::cycle_amount_parser;

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
    #[arg(value_parser = cycle_amount_parser)]
    amount: u128,
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
    let amount = opts.amount;
    let res = get_wallet(env).await?.wallet_send(canister, amount).await;
    res.map_err(|err| {
        anyhow!(
            "Sending cycles to {} failed with: {}",
            opts.destination,
            err
        )
    })
}
