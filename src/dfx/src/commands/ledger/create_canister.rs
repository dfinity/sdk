use crate::commands::ledger::get_icpts_from_args;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::error::NotifyCreateCanisterError::Notify;
use crate::lib::ledger_types::Memo;
use crate::lib::ledger_types::NotifyError::Refunded;
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};
use crate::lib::operations::cmc::{notify_create, transfer_cmc, MEMO_CREATE_CANISTER};
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::e8s_parser;
use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;

/// Create a canister from ICP
#[derive(Parser)]
pub struct CreateCanisterOpts {
    /// Specify the controller of the new canister
    controller: String,

    /// Subaccount to withdraw from
    #[arg(long)]
    from_subaccount: Option<Subaccount>,

    /// ICP to mint into cycles and deposit into destination canister
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[arg(long)]
    amount: Option<ICPTs>,

    /// Specify ICP as a whole number, helpful for use in conjunction with `--e8s`
    #[arg(long, value_parser = e8s_parser, conflicts_with("amount"))]
    icp: Option<u64>,

    /// Specify e8s as a whole number, helpful for use in conjunction with `--icp`
    #[arg(long, value_parser = e8s_parser, conflicts_with("amount"))]
    e8s: Option<u64>,

    /// Transaction fee, default is 10000 e8s.
    #[arg(long)]
    fee: Option<ICPTs>,

    /// Max fee, default is 10000 e8s.
    #[arg(long)]
    max_fee: Option<ICPTs>,

    /// Transaction timestamp, in nanoseconds, for use in controlling transaction-deduplication, default is system-time. // https://internetcomputer.org/docs/current/developer-docs/integrations/icrc-1/#transaction-deduplication-
    #[arg(long)]
    created_at_time: Option<u64>,

    /// Specify the optional subnet type to create the canister on. If no
    /// subnet type is provided, the canister will be created on a random
    /// default application subnet.
    #[arg(long)]
    subnet_type: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: CreateCanisterOpts) -> DfxResult {
    let amount = get_icpts_from_args(opts.amount, opts.icp, opts.e8s)?;

    let fee = opts.fee.unwrap_or(TRANSACTION_FEE);
    let memo = Memo(MEMO_CREATE_CANISTER);

    let controller = Principal::from_text(&opts.controller).with_context(|| {
        format!(
            "Failed to parse {:?} as controller principal.",
            &opts.controller
        )
    })?;

    let agent = env
        .get_agent()
        .ok_or_else(|| anyhow!("Cannot get HTTP client from environment."))?;

    fetch_root_key_if_needed(env).await?;

    let height = transfer_cmc(
        agent,
        memo,
        amount,
        fee,
        opts.from_subaccount,
        controller,
        opts.created_at_time,
    )
    .await?;
    println!("Using transfer at block height {height}");

    let result = notify_create(agent, controller, height, opts.subnet_type).await;

    match result {
        Ok(principal) => {
            println!("Canister created with id: {:?}", principal.to_text());
        }
        Err(Notify(Refunded {
            reason,
            block_index,
        })) => {
            match block_index {
                Some(height) => {
                    println!("Refunded at block height {height} with message: {reason}")
                }
                None => println!("Refunded with message: {reason}"),
            };
        }
        Err(other) => bail!("{other:?}"),
    };
    Ok(())
}
