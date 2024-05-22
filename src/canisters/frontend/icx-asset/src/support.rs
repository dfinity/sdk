use slog::{Drain, Level, Logger};

pub struct TermLogFormat<D>
where
    D: slog_term::Decorator,
{
    decorator: D,
}

impl<D: slog_term::Decorator> TermLogFormat<D> {
    pub fn new(decorator: D) -> TermLogFormat<D> {
        TermLogFormat { decorator }
    }
}

impl<D: slog_term::Decorator> slog::Drain for TermLogFormat<D> {
    type Ok = ();
    type Err = std::io::Error;

    fn log(
        &self,
        record: &slog::Record<'_>,
        values: &slog::OwnedKVList,
    ) -> std::result::Result<Self::Ok, Self::Err> {
        self.decorator.with_record(record, values, |decorator| {
            if record.level() <= slog::Level::Warning {
                decorator.start_level()?;
                write!(decorator, "{}: ", record.level().as_str())?;
                // start_whitespace resets to normal coloring after printing the level
                decorator.start_whitespace()?;
            }

            decorator.start_msg()?;
            write!(decorator, "{}", record.msg())?;

            decorator.start_whitespace()?;
            writeln!(decorator)?;

            decorator.flush()?;
            Ok(())
        })
    }
}

pub(crate) fn new_logger(level: Level) -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = TermLogFormat::new(decorator).fuse();
    let drain = slog::LevelFilter::new(drain, level).fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    Logger::root(drain, slog::o!())
}
