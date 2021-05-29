//! This crate provides SQL date/time types.

mod common;
mod date;
mod error;
mod format;
mod interval;
mod time;
mod timestamp;

#[cfg(feature = "oracle")]
mod oracle;
#[cfg(feature = "serde")]
mod serialize;

pub use crate::date::{Date, Month, WeekDay};
pub use crate::error::Error;
pub use crate::format::Formatter;
pub use crate::interval::{IntervalDT, IntervalYM};
pub use crate::time::Time;
pub use crate::timestamp::Timestamp;

#[cfg(feature = "oracle")]
pub use crate::oracle::Date as OracleDate;

/// General trait for all date time types.
pub trait DateTime {
    /// Extracts year from date time.
    fn year(&self) -> Option<i32>;
    /// Extracts month from date time.
    fn month(&self) -> Option<i32>;
    /// Extracts day from date time.
    fn day(&self) -> Option<i32>;
    /// Extracts hour from date time.
    fn hour(&self) -> Option<i32>;
    /// Extracts minute from date time.
    fn minute(&self) -> Option<i32>;
    /// Extracts second from date time.
    fn second(&self) -> Option<f64>;
}
