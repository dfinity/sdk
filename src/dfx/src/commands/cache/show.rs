use crate::config::cache;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

pub fn exec(env: &dyn Environment) -> DfxResult {
    let v = format!("{}", env.get_version());
    println!("{}", cache::get_bin_cache(&v)?.as_path().display());
    Ok(())
}
