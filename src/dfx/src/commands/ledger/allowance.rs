use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::ledger_types::MAINNET_LEDGER_CANISTER_ID;
use crate::lib::nns_types::icpts::ICPTs;
use crate::lib::operations::ledger;
use crate::lib::root_key::fetch_root_key_if_needed;
use crate::util::clap::parsers::icrc_subaccount_parser;
use candid::Principal;
use clap::Parser;
use icrc_ledger_types::icrc1::{self, account::Subaccount};

/// Get the ICP allowance that the spender account can transfer from the owner account.
#[derive(Parser)]
pub struct AllowanceOpts {
    /// Specifies a principal to get the allowance of.
    /// If not specified, the principal of the current identity is used.
    #[arg(long)]
    owner: Option<Principal>,

    /// Subaccount of the specified principal to get the allowance of.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    owner_subaccount: Option<Subaccount>,

    /// Specifies a spender principal to get the allowance of.
    #[arg(long)]
    spender: Principal,

    /// Subaccount of the spender principal to get the allowance of.
    #[arg(long, value_parser = icrc_subaccount_parser)]
    spender_subaccount: Option<Subaccount>,

    #[arg(long)]
    /// Canister ID of the ledger canister.
    ledger_canister_id: Option<Principal>,
}

pub async fn exec(env: &dyn Environment, opts: AllowanceOpts) -> DfxResult {
    fetch_root_key_if_needed(env).await?;

    let agent = env.get_agent();

    let canister_id = opts
        .ledger_canister_id
        .unwrap_or(MAINNET_LEDGER_CANISTER_ID);

    let owner = opts.owner.unwrap_or_else(|| {
        env.get_selected_identity_principal()
            .expect("Selected identity not instantiated.")
    });

    let owner = icrc1::account::Account {
        owner: owner,
        subaccount: opts.owner_subaccount,
    };
    let spender = icrc1::account::Account {
        owner: opts.spender,
        subaccount: opts.spender_subaccount,
    };

    let allowance = ledger::allowance(agent, &canister_id, owner, spender).await?;

    let icp = ICPTs::from_e8s(allowance.allowance.0.try_into()?);

    match allowance.expires_at {
        Some(expires_at) => {
            slog::info!(
                env.get_logger(),
                "Allowance {} expires at {}",
                icp,
                expires_at
            );
        }
        None => {
            slog::info!(env.get_logger(), "Allowance {}", icp);
        }
    }

    Ok(())
}
