use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;

/// Renames a canister.
#[derive(Parser)]
#[command(override_usage = "dfx canister rename <FROM_CANISTER> --rename-to <RENAME_TO>")]
pub struct CanisterRenameOpts {
    /// Specifies the name of the canister to rename.
    from_canister: String,

    /// Specifies the new name of the canister.
    #[arg(long)]
    rename_to: String,
}

pub async fn exec(env: &dyn Environment, opts: CanisterRenameOpts) -> DfxResult {
    println!(
        "Renaming canister from {} to {}",
        opts.from_canister, opts.rename_to
    );

    let log = env.get_logger();
    let canister_id_store = env.get_canister_id_store()?;
    let canister_id = canister_id_store.get(opts.from_canister.as_str())?;

    // TODO: Implement the renaming logic.

    Ok(())
}
