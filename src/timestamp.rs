//! Timestamp implementation.

use crate::common::USECONDS_PER_DAY;
use crate::{Date, Time};

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
        }
    }
}
