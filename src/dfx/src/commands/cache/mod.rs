use crate::config::dfinity::{CacheCommand, ConfigDefaultsCache};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;

mod delete;
mod install;
mod list;
mod show;

pub fn exec(env: &dyn Environment, cfg: ConfigDefaultsCache) -> DfxResult {
    match cfg.command {
        CacheCommand::Delete(cfg) => delete::exec(env, &cfg),
        CacheCommand::Install => install::exec(env),
        CacheCommand::List => list::exec(env),
        CacheCommand::Show => show::exec(env),
    }
}
