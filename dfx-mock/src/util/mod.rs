use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget};
use rand::Rng;
use std::thread;
use std::time::Duration;

pub mod logo;

pub struct FakeProgress {
    multi: MultiProgress,
}

// 55 fps.
const WAIT_MSEC: u64 = 40;

impl FakeProgress {
    pub fn new() -> FakeProgress {
        let multi: MultiProgress = MultiProgress::new();
        multi.set_draw_target(ProgressDrawTarget::hidden());
        FakeProgress{ multi }
    }

    pub fn join(&self) -> () {
        self.multi.set_draw_target(ProgressDrawTarget::stderr());
        self.multi.join().unwrap();
    }

    pub fn add<S, D>(&mut self,
                     time: std::ops::Range<u64>,
                     on_style: S,
                     on_done: D,
    ) where
        S: 'static + Send + FnOnce(&ProgressBar) -> (),
        D: 'static + Send + FnOnce(&ProgressBar) -> (),
    {
        let mut rng = rand::thread_rng();
        let len = rng.gen_range(time.start, time.end);
        self.add_with_len(len, time, on_style, on_done);
    }
    pub fn add_with_len<S, D>(&mut self,
                              len: u64,
                              time: std::ops::Range<u64>,
                              on_style: S,
                              on_done: D,
    ) where
        S: 'static + Send + FnOnce(&ProgressBar) -> (),
        D: 'static + Send + FnOnce(&ProgressBar) -> (),
    {
        let mut rng = rand::thread_rng();
        let time_len = rng.gen_range(time.start, time.end);
        let bar = self.multi.add(ProgressBar::new(len));
        on_style(&bar);

        // For simplicity, we use a fixed point for calculating the actual increase.
        let factor = 100000;
        let mut i = 0;

        thread::spawn(move || {
            let n_updates = time_len / WAIT_MSEC;
            for _i in 0..n_updates {
                i += factor / n_updates;
                thread::sleep(Duration::from_millis(WAIT_MSEC));
                bar.set_position((i * len) / factor);
            }

            on_done(&bar);
        });
    }
}
