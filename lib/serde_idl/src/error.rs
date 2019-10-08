use serde::{de, ser};

use super::de::RawValue;
use std::collections::VecDeque;
use std::fmt::{self, Debug, Display};
use std::io;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    message: String,
    states: Option<ErrorState>,
}

pub struct ErrorState {
    input: Vec<u8>,
    current_type: VecDeque<RawValue>,
}

impl Error {
    pub fn msg<T: Display>(msg: T) -> Self {
        Error {
            message: msg.to_string(),
            states: None,
        }
    }
    pub fn dump_states(&mut self, input: &[u8], current_type: &VecDeque<RawValue>) {
        self.states = Some(ErrorState {
            input: input.to_vec(),
            current_type: current_type.clone(),
        });
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::msg(msg.to_string())
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::msg(msg.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(std::error::Error::description(self))
    }
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&format!("\nMessage: \"{}\"\n", self.message))?;
        if let Some(ref states) = self.states {
            formatter.write_str(&format!("{:?}\n", states))?;
        }
        Ok(())
    }
}

impl Debug for ErrorState {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&format!("Trailing types {:?}\n", self.current_type))?;
        formatter.write_str(&format!("Trailing bytes {:x?}\n", self.input))?;
        Ok(())
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::msg(format!("io error: {}", e))
    }
}
