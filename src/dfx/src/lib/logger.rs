use crate::config::dfx_version_str;
use slog::{Drain, Level, Logger};
use slog_async;
use slog_term;
use std::fs::File;
use std::path::PathBuf;

/// The logging mode to use.
pub enum LoggingMode {
    /// The default mode for logging; output without any decoration, to STDERR.
    Stderr,

    /// Tee logging to a file (in addition to STDERR). This mimics the verbose flag.
    /// So it would be similar to `dfx ... |& tee /some/file.txt
    Tee(PathBuf),

    /// Output Debug logs and up to a file, regardless of verbosity, keep the STDERR output
    /// the same (with verbosity).
    File(PathBuf),
}

/// A Slog formatter that writes to a term decorator, without any formatting.
pub struct PlainFormat<D>
where
    D: slog_term::Decorator,
{
    decorator: D,
}

impl<D: slog_term::Decorator> PlainFormat<D> {
    pub fn new(decorator: D) -> PlainFormat<D> {
        PlainFormat { decorator }
    }
}

impl<D: slog_term::Decorator> slog::Drain for PlainFormat<D> {
    type Ok = ();
    type Err = std::io::Error;

    fn log(
        &self,
        record: &slog::Record<'_>,
        values: &slog::OwnedKVList,
    ) -> Result<Self::Ok, Self::Err> {
        self.decorator.with_record(record, values, |decorator| {
            decorator.start_msg()?;
            write!(decorator, "{}", record.msg())?;

            decorator.start_whitespace()?;
            writeln!(decorator)?;

            decorator.flush()?;
            Ok(())
        })
    }
}

/// Create a log drain.
fn create_drain(mode: LoggingMode) -> Logger {
    match mode {
        LoggingMode::Stderr => Logger::root(
            PlainFormat::new(slog_term::PlainSyncDecorator::new(std::io::stderr())).fuse(),
            slog::o!(),
        ),
        LoggingMode::File(out) => {
            let file = File::create(out).expect("Couldn't open log file");
            let decorator = slog_term::PlainDecorator::new(file);
            let drain = slog_term::FullFormat::new(decorator).build().fuse();
            Logger::root(slog_async::Async::new(drain).build().fuse(), slog::o!())
        }
        // A Tee mode is basically 2 drains duplicated.
        LoggingMode::Tee(out) => Logger::root(
            slog::Duplicate::new(
                create_drain(LoggingMode::Stderr),
                create_drain(LoggingMode::File(out)),
            )
            .fuse(),
            slog::o!(),
        ),
    }
}

/// Create a root logger.
/// The verbose_level can be negative, in which case it's a quiet mode which removes warnings,
/// then errors entirely.
pub fn create_root_logger(verbose_level: i64, mode: LoggingMode) -> Logger {
    let log_level = match verbose_level {
        -3 => Level::Critical,
        -2 => Level::Error,
        -1 => Level::Warning,
        0 => Level::Info,
        1 => Level::Debug,
        x => {
            if x > 0 {
                Level::Trace
            } else {
                return Logger::root(slog::Discard, slog::o!());
            }
        }
    };

    let drain = slog::LevelFilter::new(create_drain(mode), log_level).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    Logger::root(drain, slog::o!("version" => dfx_version_str()))
}
