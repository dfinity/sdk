use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use actix::{Actor, Context, Handler, Recipient, Addr, AsyncContext};
use slog::Logger;
use std::time::Duration;
use crate::actors::shutdown_controller::signals::ShutdownTriggered;

pub mod signals {
    use actix::prelude::*;

    pub mod outbound {
        use super::*;

        #[derive(Message)]
        #[rtype(result = "Result<(), ()>")]
        pub struct Shutdown {}
    }

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct ShutdownSubscribe(pub Recipient<outbound::Shutdown>);

    #[derive(Message)]
    #[rtype(result = "()")]
    pub struct ShutdownTriggered();
}

pub struct Config {
    pub logger: Option<Logger>,
}

pub struct ShutdownController {
    _logger: Logger,

    shutdown_subscribers: Vec<Recipient<signals::outbound::Shutdown>>,
}

impl ShutdownController {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        ShutdownController {
            _logger: logger,
            shutdown_subscribers: Vec::new(),
        }
    }

    // This is copied with minor changes from
    //   https://github.com/getsentry/relay/blob/master/relay-server/src/actors/controller.rs
    // any concerns about license stuff?
    pub fn shutdown(&mut self, ctx: &mut Context<Self>) {
        use actix::prelude::*;
        use futures::prelude::*;

        let futures: Vec<_> = self
            .shutdown_subscribers
            .iter()
            .map(|recipient| recipient.send(Shutdown {}))
            .map(|response| response.then(|_| future::ok::<(), ()>(())))
            .collect();

        futures::future::join_all(futures)
            .into_actor(self)
            .then(|_, _, ctx| {
                // Once all shutdowns have completed, we can schedule a stop of the actix system. It is
                // performed with a slight delay to give pending synced futures a chance to perform their
                // error handlers.
                //
                // Delay the shutdown for 100ms to allow synchronized futures to execute their error
                // handlers. Once `System::stop` is called, futures won't be polled anymore and we will not
                // be able to print error messages.
                let when = Duration::from_secs(0) + Duration::from_millis(100);

                ctx.run_later(when, |_, _| {
                    System::current().stop();
                });

                fut::wrap_future(async { () })
            })
            .spawn(ctx)
    }

    fn install_ctrlc_handler(&self, shutdown_controller: Addr<ShutdownController>)
    {
        ctrlc::set_handler(move || {
            shutdown_controller.do_send(ShutdownTriggered());
        })
            .expect("Error setting Ctrl-C handler");
    }

}

impl Actor for ShutdownController {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.install_ctrlc_handler(ctx.address());
    }
}

impl Handler<signals::ShutdownSubscribe> for ShutdownController {
    type Result = ();

    fn handle(&mut self, msg: signals::ShutdownSubscribe, _: &mut Self::Context) {
        self.shutdown_subscribers.push(msg.0);
    }
}

impl Handler<signals::ShutdownTriggered> for ShutdownController {
    type Result = ();

    fn handle(&mut self, _msg: signals::ShutdownTriggered, ctx: &mut Self::Context) {
        self.shutdown(ctx);
    }
}
