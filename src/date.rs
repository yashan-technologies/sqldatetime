//! Date implementation.

use crate::common::{
    date2julian, julian2date, DATE_MAX_YEAR, DATE_MIN_YEAR, MONTHS_PER_YEAR, UNIX_EPOCH_JULIAN,
};
use crate::error::{Error, Result};
use crate::format::{Formatter, LazyFormat, NaiveDateTime};
use crate::{Time, Timestamp};
use std::fmt::Display;

/// Date represents a valid Gregorian date.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Date(i32);

impl Date {
    /// The smallest date that can be represented by `Date`, i.e. `0001-01-01`.
    pub const MIN: Self = unsafe { Date::from_ymd_unchecked(1, 1, 1) };

    /// The largest date that can be represented by `Date`, i.e. `9999-12-31`.
    pub const MAX: Self = unsafe { Date::from_ymd_unchecked(9999, 12, 31) };

    /// Creates a `Date` from the given year, month, and day.
    ///
    /// # Safety
    /// This function is unsafe because the values are not checked for validity!
    /// Before using it, check that the values are all correct.
    #[inline]
    pub const unsafe fn from_ymd_unchecked(year: i32, month: u32, day: u32) -> Date {
        let date = date2julian(year, month, day) - UNIX_EPOCH_JULIAN;
        Date(date)
    }

    /// Creates a `Date` from the given year, month, and day.
    #[inline]
    pub const fn try_from_ymd(year: i32, month: u32, day: u32) -> Result<Date> {
        if Date::is_valid(year, month, day) {
            Ok(unsafe { Date::from_ymd_unchecked(year, month, day) })
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// Checks if the given year, month, and day fields are valid.
    #[inline]
    pub const fn is_valid(year: i32, month: u32, day: u32) -> bool {
        if year < DATE_MIN_YEAR || year > DATE_MAX_YEAR {
            return false;
        }

        if month < 1 || month > MONTHS_PER_YEAR {
            return false;
        }

        if day < 1 || day > 31 {
            return false;
        }

        if day > days_of_month(year, month) {
            return false;
        }

        true
    }

    /// Gets the value of `Date`.
    #[inline(always)]
    pub(crate) const fn value(self) -> i32 {
        self.0
    }

    #[inline(always)]
    pub(crate) const unsafe fn from_value_unchecked(value: i32) -> Self {
        Date(value)
    }

    /// Extracts `(year, month, day)` from the date.
    #[inline]
    pub const fn extract(self) -> (i32, u32, u32) {
        julian2date(self.0 + UNIX_EPOCH_JULIAN)
    }

    /// Makes a new `Timestamp` from the current date, hour, minute, second and microsecond.
    #[inline]
    pub fn and_hms(self, hour: u32, minute: u32, sec: u32, usec: u32) -> Result<Timestamp> {
        let time = Time::try_from_hms(hour, minute, sec, usec)?;
        Ok(Timestamp::new(self, time))
    }

    /// Makes a new `Timestamp` from the current date and time.
    #[inline]
    pub const fn and_time(self, time: Time) -> Timestamp {
        Timestamp::new(self, time)
    }

    /// Formats `Date` by given format string.
    #[inline]
    pub fn format<S: AsRef<str>>(self, fmt: S) -> Result<impl Display> {
        let fmt = Formatter::try_new(fmt)?;
        Ok(LazyFormat::new(fmt, self.into()))
    }
}

#[inline(always)]
const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && ((year % 100) != 0 || (year % 400) == 0)
}

#[inline(always)]
const fn days_of_month(year: i32, month: u32) -> u32 {
    const DAY_TABLE: [[u32; 12]; 2] = [
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    ];

    DAY_TABLE[is_leap_year(year) as usize][month as usize - 1]
}

impl From<Date> for NaiveDateTime {
    #[inline]
    fn from(date: Date) -> Self {
        let (year, month, day) = date.extract();

        NaiveDateTime {
            year,
            month,
            day,
            ..NaiveDateTime::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_date() {
        let date = Date::try_from_ymd(1970, 1, 1).unwrap();
        assert_eq!(date.value(), 0);
        assert_eq!(date.extract(), (1970, 1, 1));

        let date = Date::try_from_ymd(1, 1, 1).unwrap();
        assert_eq!(date.extract(), (1, 1, 1));

        let date = Date::try_from_ymd(9999, 12, 31).unwrap();
        assert_eq!(date.extract(), (9999, 12, 31));
    }
}
