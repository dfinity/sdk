use crate::actors::pocketic_proxy::PocketIcProxy;
use crate::actors::post_start::signals::{PocketIcProxyReadySignal, PocketIcProxyReadySubscribe};
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use slog::{info, Logger};

pub mod signals {
    use actix::prelude::*;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct PocketIcProxyReadySignal;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct PocketIcProxyReadySubscribe(pub Recipient<PocketIcProxyReadySignal>);
}

pub struct Config {
    pub logger: Logger,
    pub background: bool,
    pub pocketic_proxy: Option<Addr<PocketIcProxy>>,
}

pub struct PostStart {
    config: Config,
}

impl PostStart {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

impl Actor for PostStart {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Register the PostStart recipent to PocketIcProxy.
        if let Some(pocketic_proxy) = &self.config.pocketic_proxy {
            pocketic_proxy.do_send(PocketIcProxyReadySubscribe(ctx.address().recipient()));
        }
    }
}

impl Handler<PocketIcProxyReadySignal> for PostStart {
    type Result = ();

    fn handle(&mut self, _msg: PocketIcProxyReadySignal, _ctx: &mut Self::Context) -> Self::Result {
        let logger = &self.config.logger;
        if self.config.background {
            info!(logger, "Success! The dfx server is running in the background.")
        } else {
            info!(logger, "Success! The dfx server is running.\nYou must open a new terminal to continue developing. If you'd prefer to stop, quit with 'Ctrl-C'.");
        }
    }
}
