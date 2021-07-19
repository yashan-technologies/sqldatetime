//! Error definitions.

use thiserror::Error;

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can be returned when uses date/time types.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("(full) year must be between 1 and 9999")]
    DateOutOfRange,
    #[error("(full) hour must be between 0 and 23")]
    TimeOutOfRange,
    #[error("the leading precision of the interval is too small")]
    IntervalOutOfRange,
    #[error("invalid number")]
    InvalidNumber,
    #[error("not a valid month")]
    InvalidMonth,
    #[error("day of month must be between 1 and last day of month")]
    InvalidDay,
    #[error("minutes must be between 0 and 59")]
    InvalidMinute,
    #[error("seconds must be between 0 and 59")]
    InvalidSecond,
    #[error("the fractional seconds must be between 0 and 999999")]
    InvalidFraction,
    #[error("date not valid for month specified")]
    InvalidDate,
    #[error("numeric overflow")]
    NumericOverflow,
    #[error("divisor is equal to zero")]
    DivideByZero,
    #[error("{0}")]
    InvalidFormat(String),
    #[error("{0}")]
    FormatError(String),
    #[error("{0}")]
    ParseError(String),
}

impl From<std::fmt::Error> for Error {
    #[inline]
    fn from(e: std::fmt::Error) -> Self {
        Error::FormatError(e.to_string())
    }
}
