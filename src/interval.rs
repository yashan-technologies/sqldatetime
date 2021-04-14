//! Interval implementation.

use crate::common::{
    HOURS_PER_DAY, MINUTES_PER_HOUR, MONTHS_PER_YEAR, SECONDS_PER_MINUTE, USECONDS_MAX,
    USECONDS_PER_DAY, USECONDS_PER_HOUR, USECONDS_PER_MINUTE, USECONDS_PER_SECOND,
};
use crate::error::{Error, Result};
use crate::format::{LazyFormat, NaiveDateTime};
use crate::Formatter;
use std::convert::TryFrom;
use std::fmt::Display;

const INTERVAL_MAX_YEAR: i32 = 178000000;
const INTERVAL_MAX_DAY: i32 = 100000000;

/// `Year-Month Interval` represents the duration of a period of time,
/// has an interval precision that includes a YEAR field or a MONTH field, or both.
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct IntervalYM(i32);

impl IntervalYM {
    /// The smallest interval that can be represented by `IntervalYM`, i.e. `-178000000-00`.
    pub const MIN: Self = unsafe { IntervalYM::from_ym_unchecked(-178000000, 0) };

    /// The largest interval that can be represented by `IntervalYM`, i.e. `178000000-00`.
    pub const MAX: Self = unsafe { IntervalYM::from_ym_unchecked(178000000, 0) };

    /// The zero value of interval, i.e. `00-00`.
    pub const ZERO: Self = IntervalYM(0);

    /// Creates a `IntervalYM` from the given year and month.
    ///
    /// # Safety
    /// This function is unsafe because the values are not checked for validity!
    /// Before using it, check that the values are all correct.
    #[inline]
    pub const unsafe fn from_ym_unchecked(year: i32, month: i32) -> Self {
        let months = if year >= 0 {
            year * MONTHS_PER_YEAR as i32 + month
        } else {
            year * MONTHS_PER_YEAR as i32 - month
        };
        IntervalYM(months)
    }

    /// Creates a `IntervalYM` from the given month.
    ///
    /// # Safety
    /// This function is unsafe because the values are not checked for validity!
    /// Before using it, check that the values are all correct.
    #[inline]
    pub const unsafe fn from_month_unchecked(month: i32) -> Self {
        IntervalYM(month)
    }

    /// Creates a `IntervalYM` from the given year and month.
    #[inline]
    pub const fn try_from_ym(year: i32, month: i32) -> Result<Self> {
        if IntervalYM::is_valid_ym(year, month) {
            Ok(unsafe { IntervalYM::from_ym_unchecked(year, month) })
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// Creates a `IntervalYM` from the given month.
    #[inline]
    pub const fn try_from_month(month: i32) -> Result<Self> {
        if IntervalYM::is_valid_month(month) {
            Ok(unsafe { IntervalYM::from_month_unchecked(month) })
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// Checks if the given year and month are valid.
    #[inline]
    pub const fn is_valid_ym(year: i32, month: i32) -> bool {
        if (year >= INTERVAL_MAX_YEAR || year <= -INTERVAL_MAX_YEAR)
            && ((year != INTERVAL_MAX_YEAR && year != -INTERVAL_MAX_YEAR) || month != 0)
        {
            return false;
        }

        if month < 0 || month >= MONTHS_PER_YEAR as i32 {
            return false;
        }

        true
    }

    /// Checks if the given year and month are valid.
    #[inline]
    pub const fn is_valid_month(month: i32) -> bool {
        if month < -(MONTHS_PER_YEAR as i32) || month >= MONTHS_PER_YEAR as i32 {
            return false;
        }

        true
    }

    /// Gets the value of `IntervalYM`.
    #[allow(dead_code)]
    #[inline(always)]
    pub(crate) const fn value(self) -> i32 {
        self.0
    }

    /// Extracts `(year, month)` from the interval.
    #[inline]
    pub const fn extract(self) -> (i32, i32) {
        let year = self.0 / MONTHS_PER_YEAR as i32;
        let month = if year >= 0 {
            self.0 - year * MONTHS_PER_YEAR as i32
        } else {
            -self.0 + year * MONTHS_PER_YEAR as i32
        };
        (year, month)
    }

    /// Formats `IntervalYM` by given format string.
    #[inline]
    pub fn format<S: AsRef<str>>(self, fmt: S) -> Result<impl Display> {
        let fmt = Formatter::try_new(fmt)?;
        Ok(LazyFormat::new(fmt, self.into()))
    }

    /// Parses `IntervalYM` from given string and format.
    #[inline]
    pub fn parse<S1: AsRef<str>, S2: AsRef<str>>(input: S1, fmt: S2) -> Result<Self> {
        let fmt = Formatter::try_new(fmt)?;
        fmt.parse_interval_ym(input)
    }
}

impl From<IntervalYM> for NaiveDateTime {
    #[inline]
    fn from(interval: IntervalYM) -> Self {
        let (year, month) = interval.extract();
        let negate = year < 0 || month < 0;
        NaiveDateTime {
            year: year.abs(),
            month: month.abs() as u32,
            is_interval: true,
            negate,
            ..NaiveDateTime::new()
        }
    }
}

impl TryFrom<NaiveDateTime> for IntervalYM {
    type Error = Error;

    #[inline]
    fn try_from(dt: NaiveDateTime) -> Result<Self> {
        IntervalYM::try_from_ym(dt.year, dt.month as i32)
    }
}

/// `Day-Time Interval` represents the duration of a period of time,
/// has an interval precision that includes DAY, HOUR, MINUTE, SECOND, MICROSECOND.
#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct IntervalDT(i64);

impl IntervalDT {
    /// The smallest interval that can be represented by `IntervalDT`, i.e. `-100000000 00:00:00.000000`.
    pub const MIN: Self = unsafe { IntervalDT::from_dhms_unchecked(-100000000, 0, 0, 0, 0) };

    /// The largest interval that can be represented by `IntervalDT`, i.e. `100000000 00:00:00.000000`.
    pub const MAX: Self = unsafe { IntervalDT::from_dhms_unchecked(100000000, 0, 0, 0, 0) };

    /// The zero value of interval, i.e. `0 00:00:00.000000`.
    pub const ZERO: Self = IntervalDT(0);

    /// Creates a `IntervalDT` from the given day, hour, minute, second and microsecond.
    ///
    /// # Safety
    /// This function is unsafe because the values are not checked for validity!
    /// Before using it, check that the values are all correct.
    #[inline]
    pub const unsafe fn from_dhms_unchecked(
        day: i32,
        hour: i32,
        minute: i32,
        sec: i32,
        usec: i32,
    ) -> Self {
        let time = hour as i64 * USECONDS_PER_HOUR
            + minute as i64 * USECONDS_PER_MINUTE
            + sec as i64 * USECONDS_PER_SECOND
            + usec as i64;
        let us = if day > 0 {
            day as i64 * USECONDS_PER_DAY + time
        } else {
            day as i64 * USECONDS_PER_DAY - time
        };
        IntervalDT(us)
    }

    /// Creates a `IntervalDT` from the given day, hour, minute, second and microsecond.
    #[inline]
    pub const fn try_from_dhms(
        day: i32,
        hour: i32,
        minute: i32,
        sec: i32,
        usec: i32,
    ) -> Result<Self> {
        if IntervalDT::is_valid(day, hour, minute, sec, usec) {
            Ok(unsafe { IntervalDT::from_dhms_unchecked(day, hour, minute, sec, usec) })
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// Checks if the given day, hour, minute, second and microsecond fields are valid.
    #[inline]
    pub const fn is_valid(day: i32, hour: i32, minute: i32, sec: i32, usec: i32) -> bool {
        if (day <= -INTERVAL_MAX_DAY || day >= INTERVAL_MAX_DAY)
            && ((day != -INTERVAL_MAX_DAY && day != INTERVAL_MAX_DAY)
                || hour != 0
                || minute != 0
                || sec != 0
                || usec != 0)
        {
            return false;
        }

        if hour < 0 || hour >= HOURS_PER_DAY as i32 {
            return false;
        }

        if minute < 0 || minute >= MINUTES_PER_HOUR as i32 {
            return false;
        }

        if sec < 0 || sec >= SECONDS_PER_MINUTE as i32 {
            return false;
        }

        if usec < 0 || usec > USECONDS_MAX as i32 {
            return false;
        }

        true
    }

    /// Gets the value of `IntervalDT`.
    #[allow(dead_code)]
    #[inline(always)]
    pub(crate) const fn value(self) -> i64 {
        self.0
    }

    /// Extracts `(day, hour, minute, second, microsecond)` from the interval.
    #[inline]
    pub const fn extract(self) -> (i32, i32, i32, i32, i32) {
        let day = self.0 / USECONDS_PER_DAY;
        let mut time = if day >= 0 {
            self.0 - day * USECONDS_PER_DAY
        } else {
            -self.0 + day * USECONDS_PER_DAY
        };

        let hour = time / USECONDS_PER_HOUR;
        time -= hour * USECONDS_PER_HOUR;

        let minute = time / USECONDS_PER_MINUTE;
        time -= minute * USECONDS_PER_MINUTE;

        let sec = time / USECONDS_PER_SECOND;
        let usec = time - sec * USECONDS_PER_SECOND;

        (
            day as i32,
            hour as i32,
            minute as i32,
            sec as i32,
            usec as i32,
        )
    }

    /// Formats `IntervalDT` by given format string.
    #[inline]
    pub fn format<S: AsRef<str>>(self, fmt: S) -> Result<impl Display> {
        let fmt = Formatter::try_new(fmt)?;
        Ok(LazyFormat::new(fmt, self.into()))
    }

    /// Parses `IntervalDT` from given string and format.
    #[inline]
    pub fn parse<S1: AsRef<str>, S2: AsRef<str>>(input: S1, fmt: S2) -> Result<Self> {
        let fmt = Formatter::try_new(fmt)?;
        fmt.parse_interval_dt(input)
    }
}

impl From<IntervalDT> for NaiveDateTime {
    #[inline]
    fn from(interval: IntervalDT) -> Self {
        let (day, hour, minute, second, microsecond) = interval.extract();
        let negate = day < 0 || hour < 0 || minute < 0 || second < 0 || microsecond < 0;
        NaiveDateTime {
            day: day.abs() as u32,
            hour: hour.abs() as u32,
            minute: minute.abs() as u32,
            sec: second.abs() as u32,
            usec: microsecond.abs() as u32,
            is_interval: true,
            negate,
            ..NaiveDateTime::new()
        }
    }
}

impl TryFrom<NaiveDateTime> for IntervalDT {
    type Error = Error;

    #[inline]
    fn try_from(dt: NaiveDateTime) -> Result<Self> {
        IntervalDT::try_from_dhms(
            dt.day as i32,
            dt.hour as i32,
            dt.minute as i32,
            dt.sec as i32,
            dt.usec as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_ym() {
        let interval = IntervalYM::try_from_ym(0, 0).unwrap();
        assert_eq!(interval.value(), 0);
        assert_eq!(interval.extract(), (0, 0));

        let interval = IntervalYM::try_from_ym(178000000, 0).unwrap();
        assert_eq!(interval.extract(), (178000000, 0));
        let fmt = format!("{}", interval.format("yyyy-mm").unwrap());
        assert_eq!(fmt, "178000000-00");
        let interval2 = IntervalYM::parse("178000000-00", "yyyy-mm").unwrap();
        assert_eq!(interval2, interval);

        let interval = IntervalYM::try_from_ym(-178000000, 0).unwrap();
        assert_eq!(interval.extract(), (-178000000, 0));
        let fmt = format!("{}", interval.format("yyyy-mm").unwrap());
        assert_eq!(fmt, "-178000000-00");
        let interval2 = IntervalYM::parse("-178000000-00", "yyyy-mm").unwrap();
        assert_eq!(interval2, interval);

        let interval = IntervalYM::try_from_ym(177999999, 11).unwrap();
        assert_eq!(interval.extract(), (177999999, 11));

        let interval = IntervalYM::try_from_ym(-177999999, 11).unwrap();
        assert_eq!(interval.extract(), (-177999999, 11));

        let interval = IntervalYM::try_from_month(0).unwrap();
        assert_eq!(interval.extract(), (0, 0));

        let interval = IntervalYM::try_from_month(-11).unwrap();
        assert_eq!(interval.extract(), (0, -11));
        let fmt = format!("{}", interval.format("yyyy-mm").unwrap());
        assert_eq!(fmt, "-0000-11");

        // TODO
        // let interval2 = IntervalYM::parse("-0000-11", "yyyy-mm").unwrap();
        // assert_eq!(interval2, interval);
    }

    #[test]
    fn test_interval_dt() {
        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 0).unwrap();
        assert_eq!(interval.value(), 0);
        assert_eq!(interval.extract(), (0, 0, 0, 0, 0));
        let fmt = format!("{}", interval.format("DD HH:MI:SS").unwrap());
        assert_eq!(fmt, "00 00:00:00");

        let interval = IntervalDT::try_from_dhms(100000000, 0, 0, 0, 0).unwrap();
        assert_eq!(interval.extract(), (100000000, 0, 0, 0, 0));
        let fmt = format!("{}", interval.format("DD HH:MI:SS").unwrap());
        assert_eq!(fmt, "100000000 00:00:00");
        let interval2 = IntervalDT::parse("100000000 00:00:00", "DD HH:MI:SS").unwrap();
        assert_eq!(interval2, interval);

        let interval = IntervalDT::try_from_dhms(-100000000, 0, 0, 0, 0).unwrap();
        assert_eq!(interval.extract(), (-100000000, 0, 0, 0, 0));

        let interval = IntervalDT::try_from_dhms(99999999, 23, 59, 59, 999999).unwrap();
        assert_eq!(interval.extract(), (99999999, 23, 59, 59, 999999));

        let interval = IntervalDT::try_from_dhms(-99999999, 23, 59, 59, 999999).unwrap();
        assert_eq!(interval.extract(), (-99999999, 23, 59, 59, 999999));
        let fmt = format!("{}", interval.format("DD HH:MI:SS.FF6").unwrap());
        assert_eq!(fmt, "-99999999 23:59:59.999999");
    }
}
