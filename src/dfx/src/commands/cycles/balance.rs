use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::operations::cycles_ledger;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::{format_as_trillions, pretty_thousand_separators};
use candid::Principal;
use clap::Parser;

/// Get the cycle balance of the selected Identity's cycles wallet.
#[derive(Parser)]
pub struct CyclesBalanceOpts {
    /// Specifies a Principal to get the balance of
    #[arg(long)]
    owner: Option<Principal>,

    /// Subaccount of the selected identity to get the balance of
    #[arg(long)]
    subaccount: Option<Subaccount>,

    /// Get balance raw value (without upscaling to trillions of cycles).
    #[arg(long)]
    precise: bool,
}

pub async fn exec(env: &dyn Environment, opts: CyclesBalanceOpts) -> DfxResult {
    fetch_root_key_if_needed(env).await?;

    let agent = env.get_agent();

    let owner = opts.owner.unwrap_or_else(|| {
        env.get_selected_identity_principal()
            .expect("Selected identity not instantiated.")
    });

    let subaccount = opts.subaccount.map(|x| x.0);

    todo!();

    // TODO: Support this functionality again.
    // let balance = cycles_ledger::balance(agent, owner, subaccount).await?;

    // if opts.precise {
    //     println!("{} cycles.", balance);
    // } else {
    //     println!(
    //         "{} TC (trillion cycles).",
    //         pretty_thousand_separators(format_as_trillions(balance))
    //     );
    // }

    Ok(())
}
