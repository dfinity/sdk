use term::{Error, StderrTerminal, Terminal};

/// Produces the standard term::StderrTerminal that can write colors.
/// If there is no such terminal available (such as on Github CI), this produces a stderr-wrapper that skips coloring.
pub fn stderr_wrapper() -> Box<StderrTerminal> {
    term::stderr().unwrap_or_else(|| {
        Box::new(BasicStderr {
            stderr: std::io::stderr(),
        })
    })
}

struct BasicStderr<W> {
    stderr: W,
}

impl<W: std::io::Write> Terminal for BasicStderr<W> {
    type Output = W;
    fn fg(&mut self, _color: term::color::Color) -> term::Result<()> {
        Ok(())
    }

    fn bg(&mut self, _color: term::color::Color) -> term::Result<()> {
        Ok(())
    }

    fn attr(&mut self, _attr: term::Attr) -> term::Result<()> {
        Ok(())
    }

    fn supports_attr(&self, _attr: term::Attr) -> bool {
        true
    }

    fn reset(&mut self) -> term::Result<()> {
        Ok(())
    }

    fn supports_reset(&self) -> bool {
        true
    }

    fn supports_color(&self) -> bool {
        true
    }

    fn cursor_up(&mut self) -> term::Result<()> {
        Err(Error::NotSupported)
    }

    fn delete_line(&mut self) -> term::Result<()> {
        Err(Error::NotSupported)
    }

    fn carriage_return(&mut self) -> term::Result<()> {
        Err(Error::NotSupported)
    }

    fn get_ref(&self) -> &Self::Output {
        &self.stderr
    }

    fn get_mut(&mut self) -> &mut Self::Output {
        &mut self.stderr
    }

    fn into_inner(self) -> Self::Output
    where
        Self: Sized,
    {
        self.stderr
    }
}

impl<W: std::io::Write> std::io::Write for BasicStderr<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.stderr.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stderr.flush()
    }
}
