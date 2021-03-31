//! Error definitions.

use thiserror::Error;

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can be returned when uses date/time types.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("invalid: {0}")]
    Invalid(String),
    #[error("Date/Time out of range")]
    OutOfRange,
}
