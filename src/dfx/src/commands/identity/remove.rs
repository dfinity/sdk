use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use clap::Parser;
use slog::info;

/// Removes an existing identity.
#[derive(Parser)]
pub struct RemoveOpts {
    /// The identity to remove.
    removed_identity: String,

    /// Required if the identity has wallets configured so that users do not accidentally lose access to wallets.
    #[arg(long)]
    drop_wallets: bool,
}

pub fn exec(env: &dyn Environment, opts: RemoveOpts) -> DfxResult {
    let name = opts.removed_identity.as_str();

    let log = env.get_logger();

    env.new_identity_manager()?
        .remove(log, name, opts.drop_wallets, Some(log))?;

    info!(log, r#"Removed identity "{}"."#, name);
    Ok(())
}
