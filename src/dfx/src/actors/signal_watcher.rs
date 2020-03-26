use actix::{Actor, Context, System};
use std::thread;

/// A watchdog that watches for SIGINT or SIGTERM and stop the system gracefully.
/// This does not send or handle messages. It will stop the system it is running inside
/// of, when the signal has been sent to this process.
///
/// SIGINT/TERM are used because they are available on both Windows and POSIX, and
/// are semantically similar to the intent.
#[derive(Default)]
pub struct SignalWatchdog {
    join: Option<thread::JoinHandle<()>>,
}

impl SignalWatchdog {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Actor for SignalWatchdog {
    type Context = Context<Self>;

    // TODO(hansl): move this function to use tokio::signal once we support async/await. See below.
    // Essentially this hides a huge problem; tokio::signal moved to use async/await and
    // std::future::Future, which is fine. To support this 2 signals watchdog actor we need
    // to be able to select the signals together using futures::select. This is where it gets
    // wacky; we need to update our version of futures for this, but the latest isn't backward
    // compatible with the current version we use.
    // The best way to manage this is to move to async/await everywhere, upgrade futures
    // at the same time, then here move to using tokio::signal and futures::select the two
    // receivers.
    fn started(&mut self, _ctx: &mut Self::Context) {
        let signals =
            signal_hook::iterator::Signals::new(&[signal_hook::SIGTERM, signal_hook::SIGINT])
                .expect("Could not create a signal handler.");

        let system = System::current();

        let handle = thread::spawn(move || {
            let _ = signals.forever().next();
            system.stop();
        });
        self.join = Some(handle);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}
