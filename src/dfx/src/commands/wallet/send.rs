use crate::commands::wallet::do_wallet_call;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::util::clap::validators::cycle_amount_validator;

use candid::CandidType;
use clap::Clap;
use ic_types::Principal;

/// Send cycles to another cycles wallet.
#[derive(Clap)]
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
        amount: u64,
    }
    let canister = Principal::from_text(opts.destination.clone())?;
    // amount has been validated by cycle_amount_validator
    let amount = opts.amount.parse::<u64>().unwrap();
    let (res,): (Result<(), String>,) =
        do_wallet_call(env, "wallet_send", In { canister, amount }, false).await?;
    match res {
        Ok(()) => (),
        Err(err) => println!(
            "Sending cycles to {} failed with: {}",
            opts.destination, err
        ),
    };
    Ok(())
}
