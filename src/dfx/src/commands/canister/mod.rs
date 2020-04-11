use crate::config::dfinity::{CanisterCommand, ConfigDefaultsCanister};
use crate::lib::environment::{AgentEnvironment, Environment};
use crate::lib::error::{DfxError, DfxResult};
use ic_http_agent::Waiter;
use std::time::Duration;

mod call;
//mod install;
//mod query;
//mod request_status;

const RETRY_PAUSE: Duration = Duration::from_millis(100);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

pub fn create_waiter() -> Waiter {
    Waiter::builder()
        .throttle(RETRY_PAUSE)
        .timeout(REQUEST_TIMEOUT)
        .build()
}










pub fn exec(env: &dyn Environment, cfg: ConfigDefaultsCanister) -> DfxResult {
    
    let mut store: Option<AgentEnvironment<'_>> = None;
    let env = match cfg.client {
        None => env,
        Some(url) => {
            store = Some(AgentEnvironment::new(env, url.as_str()));
            store.as_ref().unwrap()
        }
    };

    match cfg.command {
        CanisterCommand::Call(cfg) => call::exec(env, &cfg),
        //Install(cfg) => ,
        //Query(cfg) => ,
        //RequestStatus(cfg) => ,
        _ => panic!(":(")
    }
}
