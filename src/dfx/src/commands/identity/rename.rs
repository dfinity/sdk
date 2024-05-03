use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use anyhow::bail;
use clap::Parser;
use dfx_core::error::identity::rename_identity::RenameIdentityError::SwitchDefaultIdentitySettingsFailed;
use slog::info;

/// Renames an existing identity.
#[derive(Parser)]
pub struct RenameOpts {
    /// The current name of the identity.
    from: String,

    /// The new name of the identity.
    to: String,
}

pub fn exec(env: &dyn Environment, opts: RenameOpts) -> DfxResult {
    let from = opts.from.as_str();
    let to = opts.to.as_str();

    let log = env.get_logger();

    let mut identity_manager = env.new_identity_manager()?;
    let result = identity_manager.rename(log, env.get_project_temp_dir()?, from, to);
    if let Err(SwitchDefaultIdentitySettingsFailed(_)) = result {
        bail!("Failed to switch over default identity settings.  Please do this manually by running 'dfx identity use {}'", to);
    }
    let renamed_default = result?;

    info!(log, r#"Renamed identity "{}" to "{}"."#, from, to);
    if renamed_default {
        info!(log, r#"Now using identity: "{}"."#, to);
    }

    Ok(())
}
