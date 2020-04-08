use crate::config::dfinity::{CacheCommand, ConfigDefaultsCache};
use crate::lib::environment::Environment;
use crate::lib::error::DfxResult;
use crate::lib::message::UserMessage;

//mod delete;
//mod install;
//mod list;
//mod show;










pub fn exec(env: &dyn Environment, cmd: CacheCommand) -> DfxResult {
    Ok(())
    //match cfg.command {
      //  CacheCommand::Delete(cfg2) => delete::exec(env, &cfg2),
        //CacheCommand::Install() => install::exec(env),
        //CacheCommand::List() => list::exec(env),
        //CacheCommand::Show() => show::exec(env),
    //}
}
