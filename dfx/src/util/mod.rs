use crate::commands::CliResult;
use console::{style, Term};
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use rand::Rng;
use std::thread;
use std::time::Duration;

pub mod logo;

pub fn fake_command<F: FnOnce() -> CliResult>(command_impl: F) -> CliResult {
    Term::stderr().write_line(
        format!("{}", style(">>> This is a mocked command.").red().bold()).as_str()
    )?;

    command_impl()
}

const WAIT_MSEC: u64 = 100;

pub type Factory = fn(u64) -> ProgressBar;
pub type OnDone = fn(&ProgressBar) -> ();
pub fn fake_progress(bars: Vec<(std::ops::Range<u64>, Factory, OnDone)>) -> CliResult {
    let mut rng = rand::thread_rng();
    let multi: MultiProgress = MultiProgress::new();

    for (value_iter, factory, on_done) in bars {
        let len = rng.gen_range(value_iter.start, value_iter.end);
        let bar = multi.add(factory(len));
        bar.set_draw_target(ProgressDrawTarget::stderr());
        bar.set_style(ProgressStyle::default_spinner());

        let _ = thread::spawn(move || {
            for _i in 0..(len / WAIT_MSEC) {
                thread::sleep(Duration::from_millis(WAIT_MSEC));
                bar.inc(1);
            }

            on_done(&bar);
        });
    }

    multi.join()
}
