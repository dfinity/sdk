#![allow(clippy::disallowed_types)]
use indicatif::{MultiProgress, ProgressBar as IndicatifProgressBar, ProgressStyle};
use std::{borrow::Cow, time::Duration};

pub struct ProgressBar {
    bar: Option<IndicatifProgressBar>,
}

macro_rules! forward_fn_impl {
    ($name: ident) => {
        pub fn $name(&self) {
            if let Some(ref progress_bar) = self.bar {
                progress_bar.$name();
            }
        }
    };

    ($name: ident, $( $tname: ident: $t: ty )+) => {
        pub fn $name(&self, $($tname: $t,)+) {
            if let Some(ref progress_bar) = self.bar {
                progress_bar.$name( $($tname,)+ );
            }
        }
    }
}

impl ProgressBar {
    pub fn new_spinner(message: Cow<'static, str>, set: &MultiProgress) -> Self {
        let progress_bar = IndicatifProgressBar::new_spinner();
        set.add(progress_bar.clone());
        progress_bar.set_message(message);
        progress_bar.enable_steady_tick(Duration::from_millis(80));

        ProgressBar {
            bar: Some(progress_bar),
        }
    }

    pub fn new_progress(total_size: u64, set: &MultiProgress) -> Self {
        let progress_bar = IndicatifProgressBar::new(total_size);
        set.add(progress_bar.clone());

        ProgressBar {
            bar: Some(progress_bar),
        }
    }

    forward_fn_impl!(finish);
    forward_fn_impl!(finish_and_clear);
    forward_fn_impl!(set_message, message: Cow<'static, str>);
    forward_fn_impl!(set_position, position: u64);
    forward_fn_impl!(set_style, style: ProgressStyle);

    pub fn discard() -> Self {
        ProgressBar { bar: None }
    }
}
