use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

use clap::Parser;

#[derive(Parser)]
pub struct UpgradeOpts {
    /// Specifies the name of the extension to upgrade.
    extension_name: String,
}

pub fn exec(_env: &dyn Environment, _opts: UpgradeOpts) -> DfxResult<()> {
    todo!()
    Ok(())
}
