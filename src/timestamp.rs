//! Timestamp implementation.

use crate::common::USECONDS_PER_DAY;
use crate::error::Result;
use crate::format::{Formatter, LazyFormat, NaiveDateTime};
use crate::{Date, Error, Time};
use std::convert::TryFrom;
use std::fmt::Display;

/// Timestamp represents a valid time at a valid Gregorian date.
///
/// This is an SQL `TIMESTAMP` value, with the specification of fractional seconds to a precision of microseconds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    /// The smallest timestamp that can be represented by `Date`, i.e. `0001-01-01 00:00:00.000000`.
    pub const MIN: Self = Timestamp::new(Date::MIN, Time::MIN);

    /// The largest timestamp that can be represented by `Date`, i.e. `9999-12-31 23:59:59.999999`.
    pub const MAX: Self = Timestamp::new(Date::MAX, Time::MAX);

    /// Creates a new `Timestamp` from a date and a time.
    #[inline]
    pub const fn new(date: Date, time: Time) -> Self {
        let value = date.value() as i64 * USECONDS_PER_DAY + time.value();
        Timestamp(value)
    }

    #[inline]
    pub const fn extract(self) -> (Date, Time) {
        let date = self.0 / USECONDS_PER_DAY;
        let time = self.0 - date * USECONDS_PER_DAY;
        unsafe {
            (
                Date::from_value_unchecked(date as i32),
                Time::from_value_unchecked(time),
            )
        }
    }

    /// Gets the value of `Timestamp`.
    #[allow(dead_code)]
    #[inline]
    pub(crate) const fn value(self) -> i64 {
        self.0
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub(crate) const unsafe fn from_value_unchecked(value: i64) -> Self {
        Timestamp(value)
    }

    /// Formats `Timestamp` by given format string.
    #[inline]
    pub fn format<S: AsRef<str>>(self, fmt: S) -> Result<impl Display> {
        let fmt = Formatter::try_new(fmt)?;
        Ok(LazyFormat::new(fmt, self.into()))
    }

    /// Parses `Timestamp` from given string and format.
    #[inline]
    pub fn parse<S1: AsRef<str>, S2: AsRef<str>>(input: S1, fmt: S2) -> Result<Self> {
        let fmt = Formatter::try_new(fmt)?;
        fmt.parse_timestamp(input)
    }
}

impl From<Timestamp> for NaiveDateTime {
    #[inline]
    fn from(ts: Timestamp) -> Self {
        let (date, time) = ts.extract();
        let (year, month, day) = date.extract();
        let (hour, minute, sec, usec) = time.extract();

        NaiveDateTime {
            year,
            month,
            day,
            hour,
            minute,
            sec,
            usec,
            ampm: None,
            is_interval: false,
            negate: false,
        }
    }
}

impl TryFrom<NaiveDateTime> for Timestamp {
    type Error = Error;

    #[inline]
    fn try_from(dt: NaiveDateTime) -> Result<Self> {
        Ok(Date::try_from(&dt)?.and_time(Time::try_from(&dt)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp() {
        {
            let date = Date::try_from_ymd(1970, 1, 1).unwrap();
            let time = Time::try_from_hms(0, 0, 0, 0).unwrap();
            let ts = Timestamp::new(date, time);
            assert_eq!(ts.value(), 0);

            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1970, 1, 1));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let fmt = format!("{}", ts.format("yyyy-mm-dd hh24:mi:ss.ff3").unwrap());
            assert_eq!(fmt, "1970-01-01 00:00:00.000");

            let ts2 =
                Timestamp::parse("1970-01-01 00:00:00.000", "yyyy-mm-dd hh24:mi:ss.ff3").unwrap();
            assert_eq!(ts2, ts);
        }

        {
            let ts = Date::try_from_ymd(1, 1, 1)
                .unwrap()
                .and_hms(0, 0, 0, 0)
                .unwrap();

            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1, 1, 1));
            assert_eq!(time.extract(), (0, 0, 0, 0));
        }

        {
            let time = Time::try_from_hms(23, 59, 59, 999999).unwrap();
            let ts = Date::try_from_ymd(9999, 12, 31).unwrap().and_time(time);

            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (9999, 12, 31));
            assert_eq!(time.extract(), (23, 59, 59, 999999));

            let fmt = format!("{}", ts.format("yyyy-mm-dd hh:mi:ss.ff1 AM").unwrap());
            assert_eq!(fmt, "9999-12-31 11:59:59.9 PM");

            let ts2 =
                Timestamp::parse("9999-12-31 11:59:59.999999 PM", "yyyy-mm-dd hh:mi:ss.ff AM")
                    .unwrap();
            assert_eq!(ts2, ts);
        }
    }
}
