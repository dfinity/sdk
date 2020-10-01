use ::actix::fut;
use futures::{future, FutureExt};
use crate::actors::shutdown_controller::signals::outbound::Shutdown;
use slog::Logger;
use actix::{AsyncContext, Recipient, Context, WrapFuture, Running, Handler, Actor, System, ActorFuture, ContextFutureSpawner};
use actix::prelude::RecipientRequest;

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
            .map(|rr: RecipientRequest<Shutdown>| rr.then(|_x| {
                future::ok(())
            }));
        let f: Vec<_> = f
            .collect();

        let joined = future::join_all(f);
        //let y = x.and_then()
        //let z = wrap_future( x );
        let joined_actor_future = joined
            .into_actor(self);
        let stop_system_future = joined_actor_future
            .then(move |_,_,_| {

                System::current().stop();

                fut::ok(())
            });

        // fails with
        // error[E0599]: no method named `spawn` found for struct `actix::fut::then::Then<actix::fut::FutureWrap<futures_util::future::join_all::JoinAll<futures_util::future::future::Then<actix::address::message::RecipientRequest<actors::shutdown_controller::signals::outbound::Shutdown>, futures_util::future::ready::Ready<std::result::Result<(), _>>, [closure@src/dfx/src/actors/shutdown_controller.rs:57:59: 59:14]>>, actors::shutdown_controller::ShutdownController>, actix::fut::result::FutureResult<(), _, actors::shutdown_controller::ShutdownController>, [closure@src/dfx/src/actors/shutdown_controller.rs:69:19: 74:14]>` in the current scope
        //    --> src/dfx/src/actors/shutdown_controller.rs:75:28
        //     |
        //  83 |           stop_system_future.spawn(ctx);
        //     |                              ^^^^^ method not found in `actix::fut::then::Then<actix::fut::FutureWrap<futures_util::future::join_all::JoinAll<futures_util::future::future::Then<actix::address::message::RecipientRequest<actors::shutdown_controller::signals::outbound::Shutdown>, futures_util::future::ready::Ready<std::result::Result<(), _>>, [closure@src/dfx/src/actors/shutdown_controller.rs:57:59: 59:14]>>, actors::shutdown_controller::ShutdownController>, actix::fut::result::FutureResult<(), _, actors::shutdown_controller::ShutdownController>, [closure@src/dfx/src/actors/shutdown_controller.rs:69:19: 74:14]>`
        //
        stop_system_future.spawn(ctx);

        // fails with
        // error[E0271]: type mismatch resolving `<actix::fut::then::Then<actix::fut::FutureWrap<futures_util::future::join_all::JoinAll<futures_util::future::future::Then<actix::address::message::RecipientRequest<actors::shutdown_controller::signals::outbound::Shutdown>, futures_util::future::ready::Ready<std::result::Result<(), _>>, [closure@src/dfx/src/actors/shutdown_controller.rs:57:59: 59:14]>>, actors::shutdown_controller::ShutdownController>, actix::fut::result::FutureResult<(), _, actors::shutdown_controller::ShutdownController>, [closure@src/dfx/src/actors/shutdown_controller.rs:69:19: 74:14]> as actix::fut::ActorFuture>::Output == ()`
        //    --> src/dfx/src/actors/shutdown_controller.rs:84:13
        //     |
        //  95 |         ctx.spawn(stop_system_future);
        //     |             ^^^^^ expected enum `std::result::Result`, found `()`
        //     |
        // = note:   expected enum `std::result::Result<(), _>`
        // found unit type `()`

        ctx.spawn(stop_system_future);
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
