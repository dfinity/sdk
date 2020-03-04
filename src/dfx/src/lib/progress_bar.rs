use indicatif::{ProgressBar as IndicatifProgressBar, ProgressDrawTarget};

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
    pub fn new_spinner(message: &str) -> Self {
        let progress_bar = IndicatifProgressBar::new_spinner();
        progress_bar.set_draw_target(ProgressDrawTarget::stderr());

        progress_bar.set_message(message);
        progress_bar.enable_steady_tick(80);

        ProgressBar {
            bar: Some(progress_bar),
        }
    }

    forward_fn_impl!(finish_with_message, message: &str);

    pub fn discard() -> Self {
        ProgressBar { bar: None }
    }
}
