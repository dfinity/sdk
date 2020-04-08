use crate::config::cache::delete_version;
use crate::config::dfinity::CacheDefaultsConfig;
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

pub fn exec(env: &dyn Environment, cfg: &CacheDefaultsConfig) -> DfxResult {
    match &cfg.version {
        None => env.get_cache().delete(),
        Some(version) => delete_version(&version).map(|_| {}),
    }
}
