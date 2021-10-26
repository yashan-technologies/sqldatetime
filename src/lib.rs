//! This crate provides SQL date/time types.
//!
//! # Feature Flags
//!
//!- `serde`: Enable `serde`-based serialization and deserialization. Not enabled by default.
//!- `oracle`: Enable Oracle oriented datetime type: `OracleDate`. Not enabled by default.

#![cfg_attr(docsrs, feature(doc_cfg))]

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
    /// Extracts second from date time.
    fn date(&self) -> Option<Date>;
}

/// Trunc trait for Timestamp/Date/OracleDate
pub trait Trunc: Sized {
    /// Truncates to the first day of the century.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 10, 1).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2001, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_century().unwrap(), result);
    /// ```
    fn trunc_century(self) -> Result<Self, Error>;

    /// Truncates to the first day of the year.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 10, 1).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_year().unwrap(), result);
    /// ```
    fn trunc_year(self) -> Result<Self, Error>;

    /// Truncates to the first day of the first week in the year.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 10, 1).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 1, 4).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_iso_year().unwrap(), result);
    /// ```
    fn trunc_iso_year(self) -> Result<Self, Error>;

    /// Truncates to the first day of the quarter.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 11, 1).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 10, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_quarter().unwrap(), result);
    /// ```
    fn trunc_quarter(self) -> Result<Self, Error>;

    /// Truncates to the first day of the month.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 10, 24).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 10, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_month().unwrap(), result);
    /// ```
    fn trunc_month(self) -> Result<Self, Error>;

    /// Truncates to the same day of the week as the first day of the year.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 1, 7).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_week().unwrap(), result);
    /// ```
    fn trunc_week(self) -> Result<Self, Error>;

    /// Truncates to the monday of the week.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 1, 7).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 1, 4).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_iso_week().unwrap(), result);
    /// ```
    fn trunc_iso_week(self) -> Result<Self, Error>;

    /// Truncates to the same day of the week as the first day of the month.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 10, 7).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 10, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_month_start_week().unwrap(), result);
    /// ```
    fn trunc_month_start_week(self) -> Result<Self, Error>;

    /// Truncates to the day.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 10, 1).unwrap().and_time(Time::MAX);
    /// let result = Date::try_from_ymd(2021, 10, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_day().unwrap(), result);
    /// ```
    fn trunc_day(self) -> Result<Self, Error>;

    /// Truncates to the sunday of the week.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 1, 7).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 1, 3).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.trunc_sunday_start_week().unwrap(), result);
    /// ```
    fn trunc_sunday_start_week(self) -> Result<Self, Error>;

    /// Truncates to the hour.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Timestamp::new( Date::try_from_ymd(2021, 1, 1).unwrap(), Time::try_from_hms(9, 59, 59, 59).unwrap());
    /// let result = Timestamp::new( Date::try_from_ymd(2021, 1, 1).unwrap(), Time::try_from_hms(9, 0, 0, 0).unwrap());
    /// assert_eq!(timestamp.trunc_hour().unwrap(), result);
    /// ```
    fn trunc_hour(self) -> Result<Self, Error>;

    /// Truncates to the minute.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Trunc};
    ///
    /// let timestamp = Timestamp::new( Date::try_from_ymd(2021, 1, 1).unwrap(), Time::try_from_hms(9, 30, 59, 59).unwrap());
    /// let result = Timestamp::new( Date::try_from_ymd(2021, 1, 1).unwrap(), Time::try_from_hms(9, 30, 0, 0).unwrap());
    /// assert_eq!(timestamp.trunc_minute().unwrap(), result);
    /// ```
    fn trunc_minute(self) -> Result<Self, Error>;
}

/// Round trait for Timestamp/Date/OracleDate
pub trait Round: Sized {
    /// If year is more than half of century, rounds to the first day of next century, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Date::try_from_ymd(2051, 1, 1).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2101, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_century().unwrap(), result);
    /// ```
    fn round_century(self) -> Result<Self, Error>;

    /// If month is bigger than June, rounds to the first day of next year, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 7, 1).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2022, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_year().unwrap(), result);
    /// ```
    fn round_year(self) -> Result<Self, Error>;

    /// If month is bigger than June, rounds to the first day of week in next year, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 7, 1).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2022, 1, 3).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_iso_year().unwrap(), result);
    /// ```
    fn round_iso_year(self) -> Result<Self, Error>;

    /// Rounds up on the sixteenth day of the second month of the quarter, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 11, 16).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2022, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_quarter().unwrap(), result);
    /// ```
    fn round_quarter(self) -> Result<Self, Error>;

    /// Rounds up on the sixteenth day of each month, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 12, 16).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2022, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_month().unwrap(), result);
    /// ```
    fn round_month(self) -> Result<Self, Error>;

    /// Rounds up on the fifth day of each week, the same day of the week as the first day of the year, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 1, 5).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 1, 8).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_week().unwrap(), result);
    /// ```
    fn round_week(self) -> Result<Self, Error>;

    /// Rounds up on the fifth day of each week, Monday be the first day of week, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 1, 8).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 1, 11).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_iso_week().unwrap(), result);
    /// ```
    fn round_iso_week(self) -> Result<Self, Error>;

    /// Rounds up on the fifth day of each week, the same day of the week as the first day of the month, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 1, 5).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 1, 8).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_month_start_week().unwrap(), result);
    /// ```
    fn round_month_start_week(self) -> Result<Self, Error>;

    /// Rounds up at 12:00 of each day, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Timestamp::new( Date::try_from_ymd(2021, 12, 31).unwrap(), Time::try_from_hms(12, 0, 0, 0).unwrap());
    /// let result = Date::try_from_ymd(2022, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_day().unwrap(), result);
    /// ```
    fn round_day(self) -> Result<Self, Error>;

    /// Rounds up on the fifth day of each week, Sunday be the first day of week, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Date::try_from_ymd(2021, 1, 8).unwrap().and_time(Time::ZERO);
    /// let result = Date::try_from_ymd(2021, 1, 10).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_sunday_start_week().unwrap(), result);
    /// ```
    fn round_sunday_start_week(self) -> Result<Self, Error>;

    /// Rounds up at half of each hour, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Timestamp::new( Date::try_from_ymd(2021, 12, 31).unwrap(), Time::try_from_hms(23, 30, 0, 0).unwrap());
    /// let result = Date::try_from_ymd(2022, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_hour().unwrap(), result);
    /// ```
    fn round_hour(self) -> Result<Self, Error>;

    /// Rounds up at half of each minute, else truncates.
    ///
    /// ## Example
    ///
    /// ```
    /// use sqldatetime::{Timestamp, Date, Time, Round};
    ///
    /// let timestamp = Timestamp::new( Date::try_from_ymd(2021, 12, 31).unwrap(), Time::try_from_hms(23, 59, 30, 0).unwrap());
    /// let result = Date::try_from_ymd(2022, 1, 1).unwrap().and_time(Time::ZERO);
    /// assert_eq!(timestamp.round_minute().unwrap(), result);
    /// ```
    fn round_minute(self) -> Result<Self, Error>;
}
