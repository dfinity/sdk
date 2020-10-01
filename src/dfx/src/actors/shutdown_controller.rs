use std::fmt;
use std::time::Duration;

use ::actix::fut;
//use ::actix::prelude::*;
use futures::{future, FutureExt};
//use futures::prelude::*;
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use slog::Logger;
use actix::fut::{FutureWrap, wrap_future};
use actix::{Recipient, Context, WrapFuture, Running, Handler, ContextFutureSpawner, Actor, System, ActorFuture};
use actix::prelude::RecipientRequest;
use actix::clock::delay_for;

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
    pub logger: Option<Logger>
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
        eprintln!("ShutdownController::shutdown");
        let f = self
            .shutdown_subscribers
            .iter();
        let f = f
            .map(|recipient: &Recipient<Shutdown>| recipient.send(Shutdown{}));
        let f = f
            .map(|future: RecipientRequest<Shutdown>| future.then(|_x| {
                future::ok(())
            }));
        let f: Vec<_> = f
            .collect();

        let x = future::join_all(f);
        //let y = x.and_then()
        //let z = wrap_future( x );
        let x = x
            .into_actor(self);
        let x = x
            .then(move |_,_,_| {

                System::current().stop();

                fut::ok(())
            });
        ctx.spawn(x);
        //let x = x
        //    .spawn(ctx);

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
        ctx.wait(wrap_future(delay_for(Duration::from_secs(10))));

    }
}
