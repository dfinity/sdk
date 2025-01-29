use crate::actors::pocketic_proxy::PocketIcProxy;
use crate::actors::post_start::signals::{PocketIcProxyReadySignal, PocketIcProxyReadySubscribe};
use crate::lib::progress_bar::ProgressBar;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use slog::{info, Logger};

pub mod signals {
    use std::net::SocketAddr;

    use actix::prelude::*;

    #[derive(Message, Copy, Clone)]
    #[rtype(result = "()")]
    pub struct PocketIcProxyReadySignal(pub SocketAddr);

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
    spinner: ProgressBar,
}

impl PostStart {
    pub fn new(config: Config, spinner: ProgressBar) -> Self {
        Self { config, spinner }
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

    fn handle(&mut self, msg: PocketIcProxyReadySignal, _ctx: &mut Self::Context) -> Self::Result {
        let logger = &self.config.logger;
        let address = msg.0;
        self.spinner.finish_and_clear();
        if self.config.background {
            info!(logger, "Replica API running in the background on {address}");
        } else {
            info!(
                logger,
                "Replica API running on {address}. You must open a new terminal to continue developing. If you'd prefer to stop, quit with 'Ctrl-C'."
            )
        }
    }
}
