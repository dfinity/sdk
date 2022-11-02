use crate::commands::ledger::{get_icpts_from_args, notify_create, transfer_cmc};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::{Memo, NotifyError};
use crate::lib::nns_types::account_identifier::Subaccount;
use crate::lib::nns_types::icpts::{ICPTs, TRANSACTION_FEE};

use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::validators::{e8s_validator, icpts_amount_validator};

use anyhow::{anyhow, bail, Context};
use candid::Principal;
use clap::Parser;
use std::str::FromStr;

pub const MEMO_CREATE_CANISTER: u64 = 1095062083_u64;

/// Create a canister from ICP
#[derive(Parser)]
pub struct CreateCanisterOpts {
    /// Specify the controller of the new canister
    controller: String,

    /// Subaccount to withdraw from
    #[clap(long)]
    from_subaccount: Option<Subaccount>,

    /// ICP to mint into cycles and deposit into destination canister
    /// Can be specified as a Decimal with the fractional portion up to 8 decimal places
    /// i.e. 100.012
    #[clap(long, validator(icpts_amount_validator))]
    amount: Option<String>,

    /// Specify ICP as a whole number, helpful for use in conjunction with `--e8s`
    #[clap(long, validator(e8s_validator), conflicts_with("amount"))]
    icp: Option<String>,

    /// Specify e8s as a whole number, helpful for use in conjunction with `--icp`
    #[clap(long, validator(e8s_validator), conflicts_with("amount"))]
    e8s: Option<String>,

    /// Transaction fee, default is 10000 e8s.
    #[clap(long, validator(icpts_amount_validator))]
    fee: Option<String>,

    /// Max fee, default is 10000 e8s.
    #[clap(long, validator(icpts_amount_validator))]
    max_fee: Option<String>,

    /// Specify the optional subnet type to create the canister on. If no
    /// subnet type is provided, the canister will be created on a random
    /// default application subnet.
    #[clap(long)]
    subnet_type: Option<String>,
}

pub async fn exec(env: &dyn Environment, opts: CreateCanisterOpts) -> DfxResult {
    let amount = get_icpts_from_args(&opts.amount, &opts.icp, &opts.e8s)?;

    let fee = opts
        .fee
        .as_ref()
        .map_or(Ok(TRANSACTION_FEE), |v| {
            ICPTs::from_str(v).map_err(|err| anyhow!(err))
        })
        .context("Failed to determine fee.")?;

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

    let height = transfer_cmc(agent, memo, amount, fee, opts.from_subaccount, controller).await?;
    println!("Transfer sent at block height {height}");
    let result = notify_create(agent, controller, height, opts.subnet_type).await?;

    match result {
        Ok(principal) => {
            println!("Canister created with id: {:?}", principal.to_text());
        }
        Err(NotifyError::Refunded {
            reason,
            block_index,
        }) => {
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
