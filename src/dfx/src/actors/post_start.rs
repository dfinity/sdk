use crate::actors::pocketic::PocketIc;
use crate::actors::post_start::signals::{PortReadySignal, PortReadySubscribe};
use crate::lib::progress_bar::ProgressBar;
use actix::{Actor, Addr, AsyncContext, Context, Handler};
use slog::{info, Logger};

pub mod signals {
    use actix::prelude::*;

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct PortReadySignal {
        pub url: String,
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct PortReadySubscribe(pub Recipient<PortReadySignal>);
}

pub struct Config {
    pub logger: Logger,
    pub background: bool,
    pub pocketic: Option<Addr<PocketIc>>,
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
        if let Some(pocketic) = &self.config.pocketic {
            pocketic.do_send(PortReadySubscribe(ctx.address().recipient()));
        }
    }
}

impl Handler<PortReadySignal> for PostStart {
    type Result = ();

    fn handle(&mut self, msg: PortReadySignal, _ctx: &mut Self::Context) -> Self::Result {
        let logger = &self.config.logger;
        let address = msg.url;
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
