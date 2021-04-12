//! Time implementation.

use crate::common::{
    HOURS_PER_DAY, MINUTES_PER_HOUR, SECONDS_PER_MINUTE, USECONDS_MAX, USECONDS_PER_HOUR,
    USECONDS_PER_MINUTE, USECONDS_PER_SECOND,
};
use crate::error::{Error, Result};
use crate::format::{Formatter, LazyFormat, NaiveDateTime};
use std::fmt::Display;

/// Time represents a valid time of day.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Time(i64);

impl Time {
    /// The smallest time that can be represented by `Time`, i.e. `00:00:00.000000`.
    pub const MIN: Self = unsafe { Time::from_hms_unchecked(0, 0, 0, 0) };

    /// The smallest time that can be represented by `Time`, i.e. `59:59:59.999999`.
    pub const MAX: Self = unsafe { Time::from_hms_unchecked(59, 59, 59, 999999) };

    /// Creates a `Time` from the given hour, minute, second and microsecond.
    ///
    /// # Safety
    /// This function is unsafe because the values are not checked for validity!
    /// Before using it, check that the values are all correct.
    #[inline]
    pub const unsafe fn from_hms_unchecked(hour: u32, minute: u32, sec: u32, usec: u32) -> Time {
        let time = hour as i64 * USECONDS_PER_HOUR
            + minute as i64 * USECONDS_PER_MINUTE
            + sec as i64 * USECONDS_PER_SECOND
            + usec as i64;
        Time(time)
    }

    /// Creates a `Time` from the given hour, minute, second and microsecond.
    #[inline]
    pub const fn try_from_hms(hour: u32, minute: u32, sec: u32, usec: u32) -> Result<Time> {
        if Time::is_valid(hour, minute, sec, usec) {
            Ok(unsafe { Time::from_hms_unchecked(hour, minute, sec, usec) })
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// Checks if the given hour, minute, second and microsecond fields are valid.
    #[inline]
    pub const fn is_valid(hour: u32, minute: u32, sec: u32, usec: u32) -> bool {
        if hour >= HOURS_PER_DAY {
            return false;
        }

        if minute >= MINUTES_PER_HOUR {
            return false;
        }

        if sec >= SECONDS_PER_MINUTE {
            return false;
        }

        if usec > USECONDS_MAX {
            return false;
        }

        true
    }

    /// Gets the value of `Time`.
    #[inline]
    pub(crate) const fn value(self) -> i64 {
        self.0
    }

    #[inline(always)]
    pub(crate) const unsafe fn from_value_unchecked(value: i64) -> Self {
        Time(value)
    }

    /// Extracts `(hour, minute, second, microsecond)` from the time.
    #[inline]
    pub const fn extract(self) -> (u32, u32, u32, u32) {
        let mut time = self.0;

        let hour = (time / USECONDS_PER_HOUR) as u32;
        time -= hour as i64 * USECONDS_PER_HOUR;

        let minute = (time / USECONDS_PER_MINUTE) as u32;
        time -= minute as i64 * USECONDS_PER_MINUTE;

        let sec = (time / USECONDS_PER_SECOND) as u32;
        time -= sec as i64 * USECONDS_PER_SECOND;

        let usec = time as u32;

        (hour, minute, sec, usec)
    }

    #[inline]
    pub fn format<S: AsRef<str>>(self, fmt: S) -> Result<impl Display> {
        let fmt = Formatter::parse(fmt)?;
        Ok(LazyFormat::new(fmt, self.into()))
    }
}

impl From<Time> for NaiveDateTime {
    #[inline]
    fn from(time: Time) -> Self {
        let (hour, minute, sec, usec) = time.extract();

        NaiveDateTime {
            hour,
            minute,
            sec,
            usec,
            ..NaiveDateTime::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time() {
        let time = Time::try_from_hms(0, 0, 0, 0).unwrap();
        assert_eq!(time.value(), 0);
        assert_eq!(time.extract(), (0, 0, 0, 0));

        let time = Time::try_from_hms(23, 59, 59, 999999).unwrap();
        assert_eq!(time.value(), 86399999999);
        assert_eq!(time.extract(), (23, 59, 59, 999999));
    }
}
