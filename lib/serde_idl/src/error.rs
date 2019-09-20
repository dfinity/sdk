use std::fmt;
use std::result;
//use serde::de;
use serde::ser;
use std::error;
use std::io;

/// This type represents all possible errors that can occur when serializing or deserializing CBOR
/// data.
pub struct Error(ErrorImpl);

/// Alias for a `Result` with the error type `serde_cbor::Error`.
pub type Result<T> = result::Result<T, Error>;

impl Error {
    pub(crate) fn io(error: io::Error) -> Error {
        Error(ErrorImpl {
            code: ErrorCode::Io(error),
            offset: 0,
        })
    }
    pub(crate) fn message<T: fmt::Display>(_msg: T) -> Error {
        Error(ErrorImpl {
            code: ErrorCode::Message(_msg.to_string()),
            offset: 0,
        })
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.0.code {
            ErrorCode::Io(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.offset == 0 {
            fmt::Display::fmt(&self.0.code, f)
        } else {
            write!(f, "{} at offset {}", self.0.code, self.0.offset)
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, fmt)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::io(e)
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        Error::message(msg)
    }
}

#[derive(Debug)]
struct ErrorImpl {
    code: ErrorCode,
    offset: u64,
}

#[derive(Debug)]
pub(crate) enum ErrorCode {
    Message(String),
    Io(io::Error),
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorCode::Message(ref msg) => f.write_str(msg),
            ErrorCode::Io(ref err) => fmt::Display::fmt(err, f),
        }
    }
}
