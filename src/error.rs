//! Error definitions.

use thiserror::Error;

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can be returned when uses date/time types.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("Date/Time out of range")]
    OutOfRange,
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
    #[error("Failed to format: {0}")]
    FormatError(String),
    #[error("Failed to parse: {0}")]
    ParseError(String),
    #[error("Divide by zero")]
    DivideByZero,
}

impl From<std::fmt::Error> for Error {
    #[inline]
    fn from(e: std::fmt::Error) -> Self {
        Error::FormatError(e.to_string())
    }
}
