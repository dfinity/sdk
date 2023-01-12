use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

pub fn exec(env: &dyn Environment) -> DfxResult<()> {
    env.get_cache().list_installed_extensions();
    Ok(())
}
