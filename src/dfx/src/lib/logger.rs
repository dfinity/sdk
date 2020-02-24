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
