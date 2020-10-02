use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use ::actix::fut;
use actix::prelude::RecipientRequest;
use actix::{
    Actor, ActorFuture, AsyncContext, Context, ContextFutureSpawner, Handler, Recipient, Running,
    System, WrapFuture,
};
use futures::{FutureExt, TryFutureExt};
use slog::Logger;
use std::time::Duration;

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
    logger: Logger,
    config: Config,

    shutdown_subscribers: Vec<Recipient<signals::outbound::Shutdown>>,
}

impl ShutdownController {
    pub fn new(config: Config) -> Self {
        let logger =
            (config.logger.clone()).unwrap_or_else(|| Logger::root(slog::Discard, slog::o!()));
        ShutdownController {
            logger,
            config,
            shutdown_subscribers: Vec::new(),
        }
    }
    pub fn shutdown(&mut self, ctx: &mut Context<Self>) {
        use actix::prelude::*;
        use futures::prelude::*;

        eprintln!("ShutdownController::shutdown");
        let futures: Vec<_> = self
            .shutdown_subscribers
            .iter()
            .map(|recipient| recipient.send(Shutdown {}))
            .map(|future| future.then(|_| future::ok::<(), ()>(())))
            .collect();

        // let joined = future::join_all(f);
        //let y = x.and_then()
        //let z = wrap_future( x );
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

        // .spawn(ctx);
    }
}

impl Actor for ShutdownController {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        eprintln!("ShutdownController::started");

        // ctrlc::set_handler(move || {
        //     eprintln!("ctrlc handler called");
        //     ctx.address().do_send(ShutdownTriggered());
        // }).expect("Error setting Ctrl-C handler");
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        eprintln!("ShutdownController::stopping");
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        eprintln!("ShutdownController::stopped");
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
