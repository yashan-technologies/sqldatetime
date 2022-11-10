//! Error definitions.

use std::collections::TryReserveError;
use std::fmt;

/// A type alias for `Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// An error that can be returned when uses date/time types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    DateOutOfRange,
    TimeOutOfRange,
    IntervalOutOfRange,
    InvalidNumber,
    InvalidMonth,
    InvalidDay,
    InvalidMinute,
    InvalidSecond,
    InvalidFraction,
    InvalidDate,
    NumericOverflow,
    DivideByZero,
    InvalidFormat(String),
    FormatError(String),
    ParseError(String),
    TryReserveError(TryReserveError),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            Error::TryReserveError(ref e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Error::DateOutOfRange => write!(f, "(full) year must be between 1 and 9999"),
            Error::TimeOutOfRange => write!(f, "(full) hour must be between 0 and 23"),
            Error::IntervalOutOfRange => {
                write!(f, "the leading precision of the interval is too small")
            }
            Error::InvalidNumber => write!(f, "invalid number"),
            Error::InvalidMonth => write!(f, "not a valid month"),
            Error::InvalidDay => write!(f, "day of month must be between 1 and last day of month"),
            Error::InvalidMinute => write!(f, "minutes must be between 0 and 59"),
            Error::InvalidSecond => write!(f, "seconds must be between 0 and 59"),
            Error::InvalidFraction => {
                write!(f, "the fractional seconds must be between 0 and 999999")
            }
            Error::InvalidDate => write!(f, "date not valid for month specified"),
            Error::NumericOverflow => write!(f, "numeric overflow"),
            Error::DivideByZero => write!(f, "divisor is equal to zero"),
            Error::InvalidFormat(ref e) => write!(f, "{}", e),
            Error::FormatError(ref e) => write!(f, "{}", e),
            Error::ParseError(ref e) => write!(f, "{}", e),
            Error::TryReserveError(ref e) => write!(f, "{}", e),
        }
    }
}

impl From<fmt::Error> for Error {
    #[inline]
    fn from(e: fmt::Error) -> Self {
        match try_format!("{}", e) {
            Ok(s) => Error::FormatError(s),
            Err(e) => e,
        }
    }
}

impl From<TryReserveError> for Error {
    #[inline]
    fn from(e: TryReserveError) -> Self {
        Error::TryReserveError(e)
    }
}
