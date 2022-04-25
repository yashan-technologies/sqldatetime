//! Time implementation.

use crate::common::{
    is_valid_time, HOURS_PER_DAY, MINUTES_PER_HOUR, SECONDS_PER_MINUTE, USECONDS_MAX,
    USECONDS_PER_DAY, USECONDS_PER_HOUR, USECONDS_PER_MINUTE, USECONDS_PER_SECOND,
};
use crate::error::{Error, Result};
use crate::format::{Formatter, LazyFormat, NaiveDateTime};
use crate::{Date, DateTime, IntervalDT, Timestamp};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Display;

/// Time represents a valid time of day.
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Time(i64);

impl Time {
    /// The zero time that can be represented by `Time`, i.e. `00:00:00.000000`.
    pub const ZERO: Self = unsafe { Time::from_hms_unchecked(0, 0, 0, 0) };

    /// The max time that can be represented by `Time`, i.e. `23:59:59.999999`.
    pub const MAX: Self = unsafe { Time::from_hms_unchecked(23, 59, 59, 999999) };

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
        if hour >= HOURS_PER_DAY {
            return Err(Error::TimeOutOfRange);
        }

        if minute >= MINUTES_PER_HOUR {
            return Err(Error::InvalidMinute);
        }

        if sec >= SECONDS_PER_MINUTE {
            return Err(Error::InvalidSecond);
        }

        if usec > USECONDS_MAX {
            return Err(Error::InvalidFraction);
        }

        Ok(unsafe { Time::from_hms_unchecked(hour, minute, sec, usec) })
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

    /// Gets the microseconds of `Time`.
    #[inline(always)]
    pub const fn usecs(self) -> i64 {
        self.0
    }

    /// Creates a `Time` from the given microseconds without checking validity.
    ///
    /// # Safety
    /// This function is unsafe because the microsecond value is not checked for validity!
    /// Before using it, check that the value is correct.
    #[inline(always)]
    pub const unsafe fn from_usecs_unchecked(usecs: i64) -> Self {
        Time(usecs)
    }

    /// Creates a `Time` from the given microseconds
    #[allow(dead_code)]
    #[inline]
    pub const fn try_from_usecs(usecs: i64) -> Result<Self> {
        if is_valid_time(usecs) {
            Ok(unsafe { Time::from_usecs_unchecked(usecs) })
        } else {
            Err(Error::TimeOutOfRange)
        }
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

    /// Formats `Time` by given format string.
    #[inline]
    pub fn format<S: AsRef<str>>(self, fmt: S) -> Result<impl Display> {
        let fmt = Formatter::try_new(fmt)?;
        Ok(LazyFormat::new(fmt, self))
    }

    /// Parses `Time` from given string and format.
    #[inline]
    pub fn parse<S1: AsRef<str>, S2: AsRef<str>>(input: S1, fmt: S2) -> Result<Self> {
        let fmt = Formatter::try_new(fmt)?;
        fmt.parse(input)
    }

    /// `Time` subtracts `Time`
    #[inline]
    pub const fn sub_time(self, time: Time) -> IntervalDT {
        unsafe { IntervalDT::from_usecs_unchecked(self.usecs() - time.usecs()) }
    }

    /// `Time` adds `IntervalDT`
    #[inline]
    pub const fn add_interval_dt(self, interval: IntervalDT) -> Time {
        let temp_result = self.usecs() + interval.usecs() % USECONDS_PER_DAY;
        if temp_result >= 0 {
            unsafe { Time::from_usecs_unchecked(temp_result % USECONDS_PER_DAY) }
        } else {
            unsafe { Time::from_usecs_unchecked(temp_result + USECONDS_PER_DAY) }
        }
    }

    /// `Time` subtracts `IntervalDT`
    #[inline]
    pub const fn sub_interval_dt(self, interval: IntervalDT) -> Time {
        self.add_interval_dt(interval.negate())
    }

    /// `Time` multiplies `f64`
    #[inline]
    pub fn mul_f64(self, number: f64) -> Result<IntervalDT> {
        unsafe { IntervalDT::from_usecs_unchecked(self.usecs()).mul_f64(number) }
    }

    /// 'Time' divides `f64`
    #[inline]
    pub fn div_f64(self, number: f64) -> Result<IntervalDT> {
        unsafe { IntervalDT::from_usecs_unchecked(self.usecs()).div_f64(number) }
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

impl From<Timestamp> for Time {
    #[inline(always)]
    fn from(timestamp: Timestamp) -> Self {
        timestamp.time()
    }
}

impl PartialEq<IntervalDT> for Time {
    #[inline]
    fn eq(&self, other: &IntervalDT) -> bool {
        self.usecs() == other.usecs()
    }
}

impl PartialOrd<IntervalDT> for Time {
    #[inline]
    fn partial_cmp(&self, other: &IntervalDT) -> Option<Ordering> {
        Some(self.usecs().cmp(&other.usecs()))
    }
}

impl TryFrom<&NaiveDateTime> for Time {
    type Error = Error;

    #[inline]
    fn try_from(dt: &NaiveDateTime) -> Result<Self> {
        Time::try_from_hms(dt.hour, dt.minute, dt.sec, dt.usec)
    }
}

impl TryFrom<NaiveDateTime> for Time {
    type Error = Error;

    #[inline]
    fn try_from(dt: NaiveDateTime) -> Result<Self> {
        Time::try_from(&dt)
    }
}

impl DateTime for Time {
    #[inline(always)]
    fn year(&self) -> Option<i32> {
        None
    }

    #[inline(always)]
    fn month(&self) -> Option<i32> {
        None
    }

    #[inline(always)]
    fn day(&self) -> Option<i32> {
        None
    }

    #[inline(always)]
    fn hour(&self) -> Option<i32> {
        Some((self.usecs() / USECONDS_PER_HOUR) as i32)
    }

    #[inline(always)]
    fn minute(&self) -> Option<i32> {
        let remain_time = self.usecs() % USECONDS_PER_HOUR;
        Some((remain_time / USECONDS_PER_MINUTE) as i32)
    }

    #[inline(always)]
    fn second(&self) -> Option<f64> {
        let remain_time = self.usecs() % USECONDS_PER_MINUTE;
        Some(remain_time as f64 / USECONDS_PER_SECOND as f64)
    }

    #[inline(always)]
    fn date(&self) -> Option<Date> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time() {
        assert_eq!(Time::ZERO, Time::try_from_hms(0, 0, 0, 0).unwrap());
        assert_eq!(Time::MAX, Time::try_from_hms(23, 59, 59, 999999).unwrap());

        let time = Time::try_from_hms(0, 0, 0, 0).unwrap();
        assert_eq!(time.usecs(), 0);
        assert_eq!(time.extract(), (0, 0, 0, 0));

        let time = Time::try_from_hms(23, 59, 59, 999999).unwrap();
        assert_eq!(time.usecs(), 86399999999);
        assert_eq!(time.extract(), (23, 59, 59, 999999));
        let time2 = Time::parse("23:59:59.999999", "HH24:MI:SS.FF").unwrap();
        assert_eq!(time2, time);
        let time2 = Time::parse("235959999999", "HH24MISSFF").unwrap();
        assert_eq!(time2, time);

        let time = Time::try_from_hms(23, 59, 5, 0).unwrap();
        let time2 = Time::parse("23595", "HH24MISS").unwrap();
        assert_eq!(time2, time);

        // parse/format with fraction
        {
            let time = Time::try_from_hms(12, 59, 5, 12345).unwrap();
            let time2 = Time::parse("12:59:5.012345", "HH:MI:SS.FF").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 50000).unwrap();
            let time2 = Time::parse("23:59:5.05", "HH24:MI:SS.FF").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 500000).unwrap();
            let time2 = Time::parse("23:59:5.5", "HH24:MI:SS.FF").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 123450).unwrap();
            let time2 = Time::parse("23:59:5.12345", "HH24:MI:SS.FF").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 123450).unwrap();
            let time2 = Time::parse("23:59:5.12345000", "HH24:MI:SS.FF9").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 123450).unwrap();
            let time2 = Time::parse("23:59:5.123450", "HH24:MI:SS.FF").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 500).unwrap();
            let time2 = Time::parse("23:59:5.0005", "HH24:MI:SS.FF").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 123450).unwrap();
            let time2 = Time::parse("23:59:5.12345", "HH24:MI:SS.FF9").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 50000).unwrap();
            let time2 = Time::parse("23:59:5.05", "HH24:MI:SS.FF3").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 50000).unwrap();
            let time2 = Time::parse("05 23:59:5", "ff3 HH24:MI:SS").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(23, 59, 5, 50000).unwrap();
            let fmt = time.format("HH24:MI:SS.FF").unwrap();
            assert_eq!(format!("{}", fmt), "23:59:05.050000");

            let fmt = time.format("HH24:MI:SS.FF4").unwrap();
            assert_eq!(format!("{}", fmt), "23:59:05.0500");

            let time = Time::try_from_hms(23, 59, 5, 123456).unwrap();
            let fmt = time.format("HH24:MI:SS.FF9").unwrap();
            assert_eq!(format!("{}", fmt), "23:59:05.123456000");
        }

        // Out of order
        {
            // Parse
            let time = Time::try_from_hms(23, 59, 59, 999999).unwrap();
            let time2 = Time::parse("999999 59:23:59", "FF MI:HH24:SS").unwrap();
            assert_eq!(time2, time);

            // Format
            let time = Time::try_from_hms(23, 59, 59, 999999).unwrap();
            let fmt = time.format("AM MI-HH/SS\\FF4").unwrap();
            assert_eq!(format!("{}", fmt), "PM 59-11/59\\9999");
        }

        // Default parse
        {
            let time = Time::try_from_hms(0, 0, 1, 0).unwrap();
            let time2 = Time::parse("01", "SS").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(0, 0, 0, 0).unwrap();
            let time2 = Time::parse("", "HH24:MI:SS").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(11, 0, 0, 0).unwrap();
            let time2 = Time::parse("11", "HH24:MI:SS").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(11, 23, 0, 0).unwrap();
            let time2 = Time::parse("11:23", "HH24:MI:SS").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(12, 0, 0, 0).unwrap();
            let time2 = Time::parse("", "HH:MI:SS").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(11, 0, 0, 0).unwrap();
            let time2 = Time::parse("11", "HH:MI:SS").unwrap();
            assert_eq!(time2, time);

            let time = Time::try_from_hms(11, 23, 0, 0).unwrap();
            let time2 = Time::parse("11:23", "HH:MI:SS").unwrap();
            assert_eq!(time2, time);
        }

        // Short Format
        {
            let time = Time::try_from_hms(7, 8, 9, 10).unwrap();
            assert_eq!(format!("{}", time.format("mi").unwrap()), "08");
            assert_eq!(format!("{}", time.format("hh").unwrap()), "07");
            assert_eq!(format!("{}", time.format("ss").unwrap()), "09");
            assert_eq!(format!("{}", time.format("FF").unwrap()), "000010");
        }

        // tolerate absence of time
        {
            // Absence of time
            {
                let time = Time::parse("", "HH24:MI:SS").unwrap();
                assert_eq!(time, Time::try_from_hms(0, 0, 0, 0).unwrap());

                let time = Time::parse("", "HH24-MI-SS").unwrap();
                assert_eq!(time, Time::try_from_hms(0, 0, 0, 0).unwrap());

                let time = Time::parse("11", "HH24:MI:SS").unwrap();
                assert_eq!(time, Time::try_from_hms(11, 0, 0, 0).unwrap());

                let time = Time::parse("11:23", "HH24:MI:SS").unwrap();
                assert_eq!(time, Time::try_from_hms(11, 23, 0, 0).unwrap());

                let time = Time::parse("11:23:25", "HH24:MI:SS.ff").unwrap();
                assert_eq!(time, Time::try_from_hms(11, 23, 25, 0).unwrap());
            }
            // Absence of am/pm
            {
                let time = Time::parse("11:00:23", "HH12:MI:SS PM").unwrap();
                assert_eq!(time, Time::try_from_hms(11, 0, 23, 0).unwrap());

                let time = Time::parse("12:00:23", "HH12:MI:SS PM").unwrap();
                assert_eq!(time, Time::try_from_hms(12, 0, 23, 0).unwrap());

                let time = Time::parse("1:00:23", "HH12:MI:SS PM").unwrap();
                assert_eq!(time, Time::try_from_hms(1, 0, 23, 0).unwrap());

                let time = Time::parse("11:00:23", "HH12:MI:SS AM").unwrap();
                assert_eq!(time, Time::try_from_hms(11, 0, 23, 0).unwrap());

                let time = Time::parse("1:00:23", "HH12:MI:SS AM").unwrap();
                assert_eq!(time, Time::try_from_hms(1, 0, 23, 0).unwrap());

                let time = Time::parse("12:00:23", "HH12:MI:SS AM").unwrap();
                assert_eq!(time, Time::try_from_hms(12, 0, 23, 0).unwrap());

                let time = Time::parse("12:00:23", "HH12:MI:SS").unwrap();
                assert_eq!(time, Time::try_from_hms(12, 0, 23, 0).unwrap());
            }
        }

        // Invalid
        {
            assert!(Time::parse("60", "SS").is_err());
            assert!(Time::parse("60", "mi").is_err());
            assert!(Time::parse("60", "hh").is_err());
            assert!(Time::parse("60", "hh").is_err());
            assert!(Time::parse("13", "hh").is_err());
            assert!(Time::parse("99999999", "FF").is_err());

            assert!(Time::parse("23635", "HHMISS").is_err());
            let time = Time::try_from_hms(1, 2, 3, 4).unwrap();
            assert!(time.format("testtest").is_err());

            assert!(Time::parse("11:12:134", "hh24:mi:ss.ff").is_err());
            assert!(Time::parse("11:12134", "hh24:mi:ss.ff").is_err());

            // todo Add all types check
        }
    }

    #[test]
    fn test_time_sub_time() {
        assert_eq!(
            Time::try_from_hms(0, 0, 0, 0)
                .unwrap()
                .sub_time(Time::try_from_hms(1, 2, 3, 4).unwrap()),
            -IntervalDT::try_from_dhms(0, 1, 2, 3, 4).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(0, 0, 0, 0)
                .unwrap()
                .sub_time(Time::try_from_hms(23, 59, 59, 999999).unwrap()),
            -IntervalDT::try_from_dhms(0, 23, 59, 59, 999999).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(12, 2, 4, 6)
                .unwrap()
                .sub_time(Time::try_from_hms(1, 3, 4, 6).unwrap()),
            IntervalDT::try_from_dhms(0, 10, 59, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_time_add_sub_interval_dt() {
        assert_eq!(
            Time::try_from_hms(12, 30, 2, 3)
                .unwrap()
                .add_interval_dt(IntervalDT::try_from_dhms(123, 1, 2, 3, 4).unwrap()),
            Time::try_from_hms(13, 32, 5, 7).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(12, 30, 2, 3)
                .unwrap()
                .add_interval_dt(IntervalDT::try_from_dhms(0, 15, 8, 59, 4).unwrap()),
            Time::try_from_hms(3, 39, 1, 7).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(12, 30, 2, 3)
                .unwrap()
                .sub_interval_dt(IntervalDT::try_from_dhms(123, 15, 8, 59, 4).unwrap()),
            Time::try_from_hms(21, 21, 2, 999999).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(12, 30, 2, 3)
                .unwrap()
                .sub_interval_dt(IntervalDT::try_from_dhms(123, 1, 2, 3, 4).unwrap()),
            Time::try_from_hms(11, 27, 58, 999999).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(0, 0, 0, 0)
                .unwrap()
                .sub_interval_dt(IntervalDT::try_from_dhms(0, 0, 0, 0, 0).unwrap()),
            Time::try_from_hms(0, 0, 0, 0).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(0, 0, 0, 0)
                .unwrap()
                .sub_interval_dt(IntervalDT::try_from_dhms(0, 1, 0, 0, 0).unwrap()),
            Time::try_from_hms(23, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_time_mul_div() {
        // Normal
        assert_eq!(
            Time::try_from_hms(1, 2, 3, 4)
                .unwrap()
                .mul_f64(5.0)
                .unwrap(),
            IntervalDT::try_from_dhms(0, 5, 10, 15, 20).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(1, 2, 3, 4)
                .unwrap()
                .mul_f64(-5.2)
                .unwrap(),
            -IntervalDT::try_from_dhms(0, 5, 22, 39, 600020).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(2, 3, 4, 5)
                .unwrap()
                .div_f64(-5.2)
                .unwrap(),
            -IntervalDT::try_from_dhms(0, 0, 23, 40, 0).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(2, 3, 4, 5)
                .unwrap()
                .div_f64(-5.0)
                .unwrap(),
            -IntervalDT::try_from_dhms(0, 0, 24, 36, 800001).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(2, 3, 4, 5)
                .unwrap()
                .div_f64(f64::INFINITY)
                .unwrap(),
            IntervalDT::try_from_dhms(0, 0, 0, 0, 0).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(2, 3, 4, 5)
                .unwrap()
                .mul_f64(100.1)
                .unwrap(),
            IntervalDT::try_from_dhms(8, 13, 18, 58, 400500).unwrap()
        );

        // Round
        assert_eq!(
            Time::try_from_hms(2, 3, 4, 5)
                .unwrap()
                .div_f64(-5.1)
                .unwrap(),
            -IntervalDT::try_from_dhms(0, 0, 24, 7, 843138).unwrap()
        );

        assert_eq!(
            Time::try_from_hms(2, 3, 4, 5)
                .unwrap()
                .mul_f64(-5.57)
                .unwrap(),
            -IntervalDT::try_from_dhms(0, 11, 25, 28, 880027).unwrap()
        );

        // Out of range
        assert!(Time::try_from_hms(2, 3, 4, 5)
            .unwrap()
            .mul_f64(-12345678999999999999.6)
            .is_err());

        assert!(Time::try_from_hms(2, 3, 4, 5)
            .unwrap()
            .div_f64(-0.00000000000000001)
            .is_err());

        assert!(Time::try_from_hms(2, 3, 4, 5)
            .unwrap()
            .mul_f64(f64::NEG_INFINITY)
            .is_err());

        assert!(Time::try_from_hms(2, 3, 4, 5)
            .unwrap()
            .div_f64(f64::NAN)
            .is_err());

        assert!(Time::try_from_hms(2, 3, 4, 5)
            .unwrap()
            .mul_f64(f64::NAN)
            .is_err());

        // Divide by zero
        assert!(Time::try_from_hms(2, 3, 4, 5)
            .unwrap()
            .div_f64(0.0)
            .is_err());
    }

    #[allow(clippy::float_cmp)]
    fn test_extract(hour: u32, min: u32, sec: u32, usec: u32) {
        let time = Time::try_from_hms(hour, min, sec, usec).unwrap();

        assert_eq!(hour as i32, time.hour().unwrap());
        assert_eq!(min as i32, time.minute().unwrap());
        assert_eq!(
            (sec as f64) + (usec as f64) / 1_000_000f64,
            time.second().unwrap()
        );

        assert!(time.year().is_none());
        assert!(time.month().is_none());
        assert!(time.day().is_none());
    }

    #[test]
    fn test_time_extract() {
        test_extract(0, 0, 0, 0);
        test_extract(1, 2, 3, 4);
        test_extract(12, 0, 0, 0);
        test_extract(16, 34, 59, 356);
        test_extract(23, 59, 59, 999999);
    }
}
