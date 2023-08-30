use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::root_key::fetch_root_key_if_needed;
use clap::Parser;

/// Convert some of the user's ICP balance into cycles.
#[derive(Parser)]
pub struct ConvertOpts {
    // /// Subaccount to transfer from.
    // #[arg(long)]
    // from_subaccount: Option<Subaccount>,
    /// ICPs to convert
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[arg(long)]
    icp_amount: ICPTs,

    // /// Specify a numeric memo for this transaction.
    // #[arg(long, value_parser = memo_parser)]
    // memo: u64,
    /// Transaction fee, default is 10000 e8s.
    #[arg(long)]
    fee: Option<ICPTs>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction-deduplication, default is system-time. // https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,
}

pub async fn exec(env: &dyn Environment, opts: ConvertOpts) -> DfxResult {
    let amount = opts.icp_amount;

    let _fee = opts.fee.unwrap_or(TRANSACTION_FEE);

    // let _memo = Memo(opts.memo);

    let _agent = env.get_agent();

    fetch_root_key_if_needed(env).await?;

    // TODO https://dfinity.atlassian.net/browse/SDK-1161

    println!("TODO: Convert {} ICP into cycles", amount);

    Ok(())
}
