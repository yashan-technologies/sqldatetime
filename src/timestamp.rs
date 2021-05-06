//! Timestamp implementation.

use crate::common::{is_valid_timestamp, USECONDS_PER_DAY};
use crate::error::{Error, Result};
use crate::format::{Formatter, LazyFormat, NaiveDateTime};
use crate::{Date, DateTime, IntervalDT, IntervalYM, Time};
use std::cmp::Ordering;
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
    pub const MIN: Self = Timestamp::new(Date::MIN, Time::ZERO);

    /// The largest timestamp that can be represented by `Date`, i.e. `9999-12-31 23:59:59.999999`.
    pub const MAX: Self = Timestamp::new(Date::MAX, Time::MAX);

    /// Creates a new `Timestamp` from a date and a time.
    #[inline]
    pub const fn new(date: Date, time: Time) -> Self {
        let value = date.value() as i64 * USECONDS_PER_DAY + time.value();
        Timestamp(value)
    }

    /// Extracts `(Date, Time)` from the timestamp.
    #[inline]
    pub const fn extract(self) -> (Date, Time) {
        let (date, time) = if self.0.is_negative() {
            let temp_time = self.0 % USECONDS_PER_DAY;
            if temp_time.is_negative() {
                (self.0 / USECONDS_PER_DAY - 1, temp_time + USECONDS_PER_DAY)
            } else {
                (self.0 / USECONDS_PER_DAY, temp_time)
            }
        } else {
            (self.0 / USECONDS_PER_DAY, self.0 % USECONDS_PER_DAY)
        };

        unsafe {
            (
                Date::from_value_unchecked(date as i32),
                Time::from_value_unchecked(time),
            )
        }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn date(self) -> Date {
        let date = if self.0.is_negative() && self.0 % USECONDS_PER_DAY != 0 {
            self.0 / USECONDS_PER_DAY - 1
        } else {
            self.0 / USECONDS_PER_DAY
        };
        unsafe { Date::from_value_unchecked(date as i32) }
    }

    #[allow(dead_code)]
    #[inline]
    pub(crate) fn time(self) -> Time {
        let temp_time = self.0 % USECONDS_PER_DAY;
        if temp_time.is_negative() {
            unsafe { Time::from_value_unchecked(temp_time as i64 + USECONDS_PER_DAY) }
        } else {
            unsafe { Time::from_value_unchecked(temp_time as i64) }
        }
    }

    /// Gets the value of `Timestamp`.
    #[inline(always)]
    pub(crate) const fn value(self) -> i64 {
        self.0
    }

    #[inline(always)]
    pub(crate) const unsafe fn from_value_unchecked(value: i64) -> Self {
        Timestamp(value)
    }

    /// Formats `Timestamp` by given format string.
    #[inline]
    pub fn format<S: AsRef<str>>(self, fmt: S) -> Result<impl Display> {
        let fmt = Formatter::try_new(fmt)?;
        Ok(LazyFormat::new(fmt, self))
    }

    /// Parses `Timestamp` from given string and format.
    #[inline]
    pub fn parse<S1: AsRef<str>, S2: AsRef<str>>(input: S1, fmt: S2) -> Result<Self> {
        let fmt = Formatter::try_new(fmt)?;
        fmt.parse(input)
    }

    #[inline]
    pub(crate) const fn try_from_value(value: i64) -> Result<Self> {
        if is_valid_timestamp(value) {
            Ok(unsafe { Timestamp::from_value_unchecked(value) })
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// `Timestamp` adds `IntervalDT`
    #[inline]
    pub const fn add_interval_dt(self, interval: IntervalDT) -> Result<Timestamp> {
        let result = self.value().checked_add(interval.value());
        match result {
            Some(ts) => Timestamp::try_from_value(ts),
            None => Err(Error::OutOfRange),
        }
    }

    /// `Timestamp` adds `IntervalYM`
    #[inline]
    pub fn add_interval_ym(self, interval: IntervalYM) -> Result<Timestamp> {
        let (date, time) = self.extract();

        Ok(Timestamp::new(
            date.add_interval_ym_internal(interval)?,
            time,
        ))
    }

    /// `Timestamp` add `Time`
    #[inline]
    pub const fn add_time(self, time: Time) -> Result<Timestamp> {
        Timestamp::try_from_value(self.value() + time.value())
    }

    /// `Timestamp` add days
    #[inline]
    pub fn add_days(self, days: f64) -> Result<Timestamp> {
        let microseconds = (days * USECONDS_PER_DAY as f64).round();
        if !microseconds.is_finite() {
            return Err(Error::OutOfRange);
        }
        let result = self.value().checked_add(microseconds as i64);
        match result {
            Some(d) => Timestamp::try_from_value(d),
            None => Err(Error::OutOfRange),
        }
    }

    /// `Timestamp` subtracts `Date`
    #[inline]
    pub const fn sub_date(self, date: Date) -> IntervalDT {
        let temp_timestamp = date.and_zero_time();
        self.sub_timestamp(temp_timestamp)
    }

    /// `Timestamp` subtracts `Time`
    #[inline]
    pub const fn sub_time(self, time: Time) -> Result<Timestamp> {
        Timestamp::try_from_value(self.value() - time.value())
    }

    /// `Timestamp` subtracts `Timestamp`
    #[inline]
    pub const fn sub_timestamp(self, timestamp: Timestamp) -> IntervalDT {
        let microseconds = self.value() - timestamp.value();
        unsafe { IntervalDT::from_value_unchecked(microseconds) }
    }

    /// `Timestamp` subtracts `IntervalDT`
    #[inline]
    pub const fn sub_interval_dt(self, interval: IntervalDT) -> Result<Timestamp> {
        self.add_interval_dt(interval.negate())
    }

    /// `Timestamp` subtracts `IntervalYM`
    #[inline]
    pub fn sub_interval_ym(self, interval: IntervalYM) -> Result<Timestamp> {
        self.add_interval_ym(interval.negate())
    }

    /// `Timestamp` subtracts days
    #[inline]
    pub fn sub_days(self, days: f64) -> Result<Timestamp> {
        self.add_days(-days)
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
            date: Some(date),
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

impl PartialEq<Date> for Timestamp {
    #[inline]
    fn eq(&self, other: &Date) -> bool {
        *self == other.and_zero_time()
    }
}

impl PartialOrd<Date> for Timestamp {
    #[inline]
    fn partial_cmp(&self, other: &Date) -> Option<Ordering> {
        Some(self.value().cmp(&other.and_zero_time().value()))
    }
}

impl From<Date> for Timestamp {
    #[inline]
    fn from(date: Date) -> Self {
        date.and_zero_time()
    }
}

impl DateTime for Timestamp {
    #[inline]
    fn year(&self) -> Option<i32> {
        self.date().year()
    }

    #[inline]
    fn month(&self) -> Option<i32> {
        self.date().month()
    }

    #[inline]
    fn day(&self) -> Option<i32> {
        self.date().day()
    }

    #[inline]
    fn hour(&self) -> Option<i32> {
        self.time().hour()
    }

    #[inline]
    fn minute(&self) -> Option<i32> {
        self.time().minute()
    }

    #[inline]
    fn second(&self) -> Option<f64> {
        self.time().second()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_ts(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        usec: u32,
    ) -> Timestamp {
        let timestamp = Timestamp::new(
            Date::try_from_ymd(year, month, day).unwrap(),
            Time::try_from_hms(hour, min, sec, usec).unwrap(),
        );
        timestamp
    }

    fn generate_date(year: i32, month: u32, day: u32) -> Date {
        Date::try_from_ymd(year, month, day).unwrap()
    }

    fn generate_time(hour: u32, min: u32, sec: u32, usec: u32) -> Time {
        Time::try_from_hms(hour, min, sec, usec).unwrap()
    }

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

            let ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1, 1, 1));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let ts = generate_ts(0001, 1, 1, 23, 59, 59, 999999);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1, 1, 1));
            assert_eq!(time.extract(), (23, 59, 59, 999999));

            let ts = generate_ts(0001, 12, 31, 0, 0, 0, 0);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1, 12, 31));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let ts = generate_ts(0001, 12, 31, 23, 59, 59, 999999);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1, 12, 31));
            assert_eq!(time.extract(), (23, 59, 59, 999999));

            let ts = generate_ts(1969, 12, 30, 0, 0, 0, 0);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1969, 12, 30));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let ts = generate_ts(1969, 12, 30, 23, 59, 59, 999999);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1969, 12, 30));
            assert_eq!(time.extract(), (23, 59, 59, 999999));

            let ts = generate_ts(1969, 12, 31, 0, 0, 0, 0);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1969, 12, 31));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let ts = generate_ts(1969, 12, 31, 23, 59, 59, 999999);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1969, 12, 31));
            assert_eq!(time.extract(), (23, 59, 59, 999999));

            let ts = generate_ts(1970, 1, 1, 0, 0, 0, 0);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1970, 1, 1));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let ts = generate_ts(1970, 1, 1, 23, 59, 59, 999999);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1970, 1, 1));
            assert_eq!(time.extract(), (23, 59, 59, 999999));

            let ts = generate_ts(1970, 3, 4, 23, 12, 30, 123456);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1970, 3, 4));
            assert_eq!(time.extract(), (23, 12, 30, 123456));

            let ts = generate_ts(9999, 12, 31, 0, 0, 0, 0);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (9999, 12, 31));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (9999, 12, 31));
            assert_eq!(time.extract(), (23, 59, 59, 999999));

            let ts = generate_ts(1969, 10, 31, 1, 1, 1, 1);
            let (date, time) = ts.extract();
            assert_eq!(date.extract(), (1969, 10, 31));
            assert_eq!(time.extract(), (1, 1, 1, 1));

            // Out of order
            {
                // Parse
                let time = Time::try_from_hms(23, 59, 59, 999999).unwrap();
                let ts = Date::try_from_ymd(9999, 12, 31).unwrap().and_time(time);
                let ts2 = Timestamp::parse(
                    "PM 9999\\12-31 11/59:59.999999",
                    "AM yyyy\\mm-dd hh/mi:ss.ff",
                )
                .unwrap();
                assert_eq!(ts2, ts);

                let ts2 =
                    Timestamp::parse("PM 11-9999-59.999999 12-59-31", "PM HH-YYYY-MI.FF MM-SS-DD")
                        .unwrap();
                assert_eq!(ts2, ts);

                let ts2 =
                    Timestamp::parse("23-9999-59.999999 12 59 31", "HH24-YYYY-MI.FF MM SS DD")
                        .unwrap();
                assert_eq!(ts, ts2);

                let ts2 = Timestamp::parse(
                    "T23--59.999999 12 59 31.9999;",
                    "THH24--MI.FF MM SS DD.yyyy;",
                )
                .unwrap();
                assert_eq!(ts, ts2);

                // Format
                let fmt = ts.format("TAM HH\\YYYY\\MI.FF MM-SS/DD").unwrap();
                assert_eq!(format!("{}", fmt), "TPM 11\\9999\\59.999999 12-59/31");

                let fmt = ts.format("HH\\YYYY\\MI MM-SS/DD.FF4;").unwrap();
                assert_eq!(format!("{}", fmt), "11\\9999\\59 12-59/31.9999;");
            }

            // Duplicate parse
            {
                // Parse
                assert!(Timestamp::parse(
                    "AM PM 9999\\12-31 11/59:59.999999",
                    "AM PM yyyy\\mm-dd hh/mi:ss.ff"
                )
                .is_err());

                assert!(Timestamp::parse(
                    "pm PM 9999\\12-31 11/59:59.999999",
                    "AM PM yyyy\\mm-dd hh/mi:ss.ff"
                )
                .is_err());

                assert!(Timestamp::parse(
                    "9999 9999\\12-31 11/59:59.999999",
                    "yyyy yyyy\\mm-dd hh/mi:ss.ff"
                )
                .is_err());

                assert!(Timestamp::parse(
                    "9999\\12-31 11/59:59.999999 59",
                    "yyyy\\mm-dd hh/mi:ss.ff mi"
                )
                .is_err());

                // todo duplication special check, including parse and format
            }

            // Default
            {
                let ts = generate_ts(0001, 1, 1, 0, 0, 5, 0);
                let ts2 = Timestamp::parse("5", "ss").unwrap();
                assert_eq!(ts, ts2);

                let ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);
                let ts2 = Timestamp::parse("", "").unwrap();
                assert_eq!(ts, ts2);

                let ts2 = Timestamp::parse("jan", "MONTH").unwrap();
                assert_eq!(ts, ts2);

                let ts2 = Timestamp::parse("January", "mon").unwrap();
                assert_eq!(ts, ts2)
            }

            // Short format
            {
                let ts = generate_ts(1234, 8, 6, 7, 8, 9, 10);
                assert_eq!(format!("{}", ts.format("YYYY").unwrap()), "1234");
                assert_eq!(format!("{}", ts.format("DD").unwrap()), "06");
                assert_eq!(format!("{}", ts.format("MON").unwrap()), "AUG");
                assert_eq!(format!("{}", ts.format("Mon").unwrap()), "Aug");
                assert_eq!(format!("{}", ts.format("mon").unwrap()), "aug");
                assert_eq!(format!("{}", ts.format("MONTH").unwrap()), "AUGUST");
                assert_eq!(format!("{}", ts.format("MONtH").unwrap()), "AUGUST");
                assert_eq!(format!("{}", ts.format("Month").unwrap()), "August");
                assert_eq!(format!("{}", ts.format("month").unwrap()), "august");
                assert_eq!(format!("{}", ts.format("DAY").unwrap()), "SUNDAY");
                assert_eq!(format!("{}", ts.format("DAy").unwrap()), "SUNDAY");
                assert_eq!(format!("{}", ts.format("Day").unwrap()), "Sunday");
                assert_eq!(format!("{}", ts.format("DaY").unwrap()), "Sunday");
                assert_eq!(format!("{}", ts.format("day").unwrap()), "sunday");
                assert_eq!(format!("{}", ts.format("daY").unwrap()), "sunday");
                assert_eq!(format!("{}", ts.format("DY").unwrap()), "SUN");
                assert_eq!(format!("{}", ts.format("Dy").unwrap()), "Sun");
                assert_eq!(format!("{}", ts.format("dy").unwrap()), "sun");
                assert_eq!(format!("{}", ts.format("mi").unwrap()), "08");
                assert_eq!(format!("{}", ts.format("hh").unwrap()), "07");
                assert_eq!(format!("{}", ts.format("ss").unwrap()), "09");
                assert_eq!(format!("{}", ts.format("FF").unwrap()), "000010");

                let ts = generate_ts(1970, 1, 1, 7, 8, 9, 10);
                assert_eq!(format!("{}", ts.format("day").unwrap()), "thursday");

                let ts = generate_ts(1970, 1, 2, 7, 8, 9, 10);
                assert_eq!(format!("{}", ts.format("day").unwrap()), "friday");

                let ts = generate_ts(1969, 12, 31, 7, 8, 9, 10);
                assert_eq!(format!("{}", ts.format("day").unwrap()), "wednesday");

                let ts = generate_ts(1969, 10, 1, 7, 8, 9, 10);
                assert_eq!(format!("{}", ts.format("day").unwrap()), "wednesday");

                let ts = generate_ts(9999, 11, 14, 7, 8, 9, 10);
                assert_eq!(format!("{}", ts.format("day").unwrap()), "sunday");
            }

            // Normal
            {
                let ts = generate_ts(2000, 1, 1, 0, 0, 0, 0);
                let fmt = format!("{}", ts.format("yyyy-MONTH-dd hh:mi:ss.ff1").unwrap());
                assert_eq!(fmt, "2000-JANUARY-01 12:00:00.0");

                let fmt = format!("{}", ts.format("yyyy-Mon-dd hh:mi:ss.ff1").unwrap());
                assert_eq!(fmt, "2000-Jan-01 12:00:00.0");

                let fmt = format!("{}", ts.format("Day yyyy-Mon-dd hh:mi:ss.ff1").unwrap());
                assert_eq!(fmt, "Saturday 2000-Jan-01 12:00:00.0");

                let fmt = format!("{}", ts.format("yyyyMMdd hh24miss.ff1").unwrap());
                assert_eq!(fmt, "20000101 000000.0");

                let ts = generate_ts(2001, 1, 2, 3, 4, 5, 6);
                assert_eq!(
                    format!("{}", ts.format("YYYYMMDDHHMISSFF").unwrap()),
                    "20010102030405000006"
                );

                assert_eq!(
                    ts,
                    Timestamp::parse("20010102030405000006", "YYYYMMDDHHMISSFF").unwrap()
                );

                assert_eq!(
                    ts,
                    Timestamp::parse("2001012 030405000006", "YYYYMMDD HHMISSFF").unwrap()
                );
            }

            // Day parse check
            {
                let ts = generate_ts(2021, 4, 22, 3, 4, 5, 6);
                let ts2 = Timestamp::parse(
                    "2021-04-22 03:04:05.000006 thu",
                    "yyyy-mm-dd hh24:mi:ss.FF6 dy",
                )
                .unwrap();
                assert_eq!(ts, ts2);

                let ts2 = Timestamp::parse(
                    "2021-04-22 03:04:05.000006 thursday",
                    "yyyy-mm-dd hh24:mi:ss.FF6 dy",
                )
                .unwrap();
                assert_eq!(ts, ts2);

                let ts2 = Timestamp::parse(
                    "2021-04-22 03:04:05.000006 thu",
                    "yyyy-mm-dd hh24:mi:ss.FF6 day",
                )
                .unwrap();
                assert_eq!(ts, ts2);

                let ts2 = Timestamp::parse(
                    "2021-04-22 03:04:05.000006 thursday",
                    "yyyy-mm-dd hh24:mi:ss.FF6 Dy",
                )
                .unwrap();
                assert_eq!(ts, ts2);

                let ts2 = Timestamp::parse(
                    "2021-04-22 03:04:05.000006 Thu",
                    "yyyy-mm-dd hh24:mi:ss.FF6 dy",
                )
                .unwrap();
                assert_eq!(ts, ts2);

                assert!(Timestamp::parse(
                    "2021-04-23 03:04:05.000006 thu",
                    "yyyy-mm-dd hh24:mi:ss.FF6 dy",
                )
                .is_err());
            }

            // Duplicate format
            {
                let ts = generate_ts(2021, 4, 25, 3, 4, 5, 6);
                assert_eq!(
                    format!("{}", ts.format("DAY DaY DY MM MM yyyy YYYY MI MI").unwrap()),
                    "SUNDAY Sunday SUN 04 04 2021 2021 04 04"
                );
            }

            // Invalid
            {
                // Parse
                assert!(Timestamp::parse(
                    "2021-04-22 03:04:05.000006",
                    "yyyy-mmX-dd hh24:mi:ss.FF6",
                )
                .is_err());

                assert!(
                    Timestamp::parse("2021-04-22 03:04:05.000006", "yyyy-mm-dd mi:ss.FF7",)
                        .is_err()
                );

                assert!(
                    Timestamp::parse("2021-04-22 03:04:05.000006", "yyy-mm-dd hh24:mi:ss.FF7",)
                        .is_err()
                );

                assert!(
                    Timestamp::parse("2021-04-32 03:04:05.000006", "yyyy-mm-dd mi:ss.FF7",)
                        .is_err()
                );

                assert!(
                    Timestamp::parse("10000-04-31 03:04:05.000006", "yyyy-mm-dd mi:ss.FF6",)
                        .is_err()
                );

                assert!(
                    Timestamp::parse("10000-04-31 33:04:05.000006", "yyyy-mm-dd mi:ss.FF6",)
                        .is_err()
                );

                assert!(Timestamp::parse(
                    "2021-04-22 03:04:05.000006",
                    "ABCD-mm-dd hh24:mi:ss.FF10",
                )
                .is_err());

                assert!(Timestamp::parse(
                    "2021-04-23 03:04:05.000006 thur",
                    "yyyy-mm-dd hh24:mi:ss.FF6 dy",
                )
                .is_err());

                assert!(
                    Timestamp::parse("2021423 03:04:05.000006", "yyyymmdd hh24:mi:ss.FF6",)
                        .is_err()
                );

                assert!(
                    Timestamp::parse("2021423 03:04:05.000006", "yyyymmdd hh24:mi:ss.FF3",)
                        .is_err()
                );

                let timestamp = generate_ts(1234, 5, 6, 7, 8, 9, 10);
                assert!(timestamp.format("testtest").is_err());
            }

            // todo
            // Wrong order of some specific Field, wrong format, extra format.
        }
    }

    #[test]
    fn test_timestamp_date_time() {
        let ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);
        assert_eq!(ts.date(), generate_date(0001, 1, 1));
        assert_eq!(ts.time(), generate_time(0, 0, 0, 0));

        let ts = generate_ts(0001, 1, 1, 23, 59, 59, 999999);
        assert_eq!(ts.date(), generate_date(0001, 1, 1));
        assert_eq!(ts.time(), generate_time(23, 59, 59, 999999));

        let ts = generate_ts(1969, 12, 30, 0, 0, 0, 0);
        assert_eq!(ts.date(), generate_date(1969, 12, 30));
        assert_eq!(ts.time(), generate_time(0, 0, 0, 0));

        let ts = generate_ts(1969, 12, 30, 23, 59, 59, 999999);
        assert_eq!(ts.date(), generate_date(1969, 12, 30));
        assert_eq!(ts.time(), generate_time(23, 59, 59, 999999));

        let ts = generate_ts(1969, 12, 31, 0, 0, 0, 0);
        assert_eq!(ts.date(), generate_date(1969, 12, 31));
        assert_eq!(ts.time(), generate_time(0, 0, 0, 0));

        let ts = generate_ts(1969, 12, 31, 23, 59, 59, 999999);
        assert_eq!(ts.date(), generate_date(1969, 12, 31));
        assert_eq!(ts.time(), generate_time(23, 59, 59, 999999));

        let ts = generate_ts(1970, 1, 1, 0, 0, 0, 0);
        assert_eq!(ts.date(), generate_date(1970, 1, 1));
        assert_eq!(ts.time(), generate_time(0, 0, 0, 0));

        let ts = generate_ts(1970, 1, 1, 23, 59, 59, 999999);
        assert_eq!(ts.date(), generate_date(1970, 1, 1));
        assert_eq!(ts.time(), generate_time(23, 59, 59, 999999));

        let ts = generate_ts(9999, 1, 1, 0, 0, 0, 0);
        assert_eq!(ts.date(), generate_date(9999, 1, 1));
        assert_eq!(ts.time(), generate_time(0, 0, 0, 0));

        let ts = generate_ts(9999, 1, 1, 23, 59, 59, 999999);
        assert_eq!(ts.date(), generate_date(9999, 1, 1));
        assert_eq!(ts.time(), generate_time(23, 59, 59, 999999));

        let ts = generate_ts(9999, 12, 31, 0, 0, 0, 0);
        assert_eq!(ts.date(), generate_date(9999, 12, 31));
        assert_eq!(ts.time(), generate_time(0, 0, 0, 0));

        let ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        assert_eq!(ts.date(), generate_date(9999, 12, 31));
        assert_eq!(ts.time(), generate_time(23, 59, 59, 999999));
    }

    #[test]
    fn test_timestamp_add_sub_interval_dt() {
        // Normal add positive interval test
        let ts = generate_ts(2001, 3, 31, 12, 5, 6, 7);
        let interval = IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        let expect = generate_ts(2001, 4, 1, 14, 8, 10, 12);
        assert_eq!(ts.add_interval_dt(interval).unwrap(), expect);

        // Normal sub negative interval test
        let interval = -IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        assert_eq!(ts.sub_interval_dt(interval).unwrap(), expect);

        // Add positive interval with carry test
        let ts = generate_ts(2001, 12, 31, 23, 59, 59, 999999);
        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        let expect = generate_ts(2002, 1, 1, 0, 0, 0, 0);
        assert_eq!(ts.add_interval_dt(interval).unwrap(), expect);

        // Sub negative interval with carry test
        let interval = -IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert_eq!(ts.sub_interval_dt(interval).unwrap(), expect);

        // Normal add negative interval test
        let ts = generate_ts(2001, 3, 31, 12, 5, 6, 7);
        let interval = -IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        let expect = generate_ts(2001, 3, 30, 10, 2, 2, 2);
        assert_eq!(ts.add_interval_dt(interval).unwrap(), expect);

        // Normal sub positive interval test
        let interval = IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        assert_eq!(ts.sub_interval_dt(interval).unwrap(), expect);

        // Add negative interval with carry test
        let ts = generate_ts(1970, 1, 1, 0, 0, 0, 0);
        let interval = -IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        let expect = generate_ts(1969, 12, 31, 23, 59, 59, 999999);
        assert_eq!(ts.add_interval_dt(interval).unwrap(), expect);

        // Sub positive interval with carry test
        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert_eq!(ts.sub_interval_dt(interval).unwrap(), expect);

        // Boundary test
        let ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let interval = IntervalDT::try_from_dhms(5, 4, 3, 2, 1).unwrap();
        let expect = generate_ts(9999, 12, 26, 19, 56, 57, 999998);
        assert_eq!(ts.sub_interval_dt(interval).unwrap(), expect);

        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert!(ts.add_interval_dt(interval).is_err());

        let interval = IntervalDT::try_from_dhms(12345, 12, 3, 5, 6).unwrap();
        assert!(ts.add_interval_dt(interval).is_err());

        let ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);
        let interval = IntervalDT::try_from_dhms(5, 4, 3, 2, 1).unwrap();
        let expect = generate_ts(0001, 1, 6, 4, 3, 2, 1);
        assert_eq!(ts.add_interval_dt(interval).unwrap(), expect);

        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert!(ts.sub_interval_dt(interval).is_err());

        let interval = IntervalDT::try_from_dhms(12345, 12, 3, 5, 6).unwrap();
        assert!(ts.sub_interval_dt(interval).is_err());
    }

    #[test]
    fn test_timestamp_add_sub_interval_ym() {
        // Add positive
        let ts = generate_ts(2001, 3, 31, 12, 5, 6, 7);
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            ts.add_interval_ym(interval).unwrap(),
            generate_ts(2001, 5, 31, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            ts.add_interval_ym(interval).unwrap(),
            generate_ts(2002, 5, 31, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            ts.add_interval_ym(interval).unwrap(),
            generate_ts(2002, 4, 30, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            ts.add_interval_ym(interval).unwrap(),
            generate_ts(2002, 2, 28, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(2, 11).unwrap();
        assert_eq!(
            ts.add_interval_ym(interval).unwrap(),
            generate_ts(2004, 2, 29, 12, 5, 6, 7)
        );

        // Sub negative
        let ts = generate_ts(2001, 3, 31, 12, 5, 6, 7);
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            ts.sub_interval_ym(-interval).unwrap(),
            generate_ts(2001, 5, 31, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            ts.sub_interval_ym(-interval).unwrap(),
            generate_ts(2002, 5, 31, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            ts.sub_interval_ym(-interval).unwrap(),
            generate_ts(2002, 4, 30, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            ts.sub_interval_ym(-interval).unwrap(),
            generate_ts(2002, 2, 28, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(2, 11).unwrap();
        assert_eq!(
            ts.sub_interval_ym(-interval).unwrap(),
            generate_ts(2004, 2, 29, 12, 5, 6, 7)
        );

        // Sub positive
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            ts.sub_interval_ym(interval).unwrap(),
            generate_ts(2001, 1, 31, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            ts.sub_interval_ym(interval).unwrap(),
            generate_ts(2000, 1, 31, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            ts.sub_interval_ym(interval).unwrap(),
            generate_ts(2000, 2, 29, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            ts.sub_interval_ym(interval).unwrap(),
            generate_ts(2000, 4, 30, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert_eq!(
            ts.sub_interval_ym(interval).unwrap(),
            generate_ts(1999, 2, 28, 12, 5, 6, 7)
        );

        // Add negative
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            ts.add_interval_ym(-interval).unwrap(),
            generate_ts(2001, 1, 31, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            ts.add_interval_ym(-interval).unwrap(),
            generate_ts(2000, 1, 31, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            ts.add_interval_ym(-interval).unwrap(),
            generate_ts(2000, 2, 29, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            ts.add_interval_ym(-interval).unwrap(),
            generate_ts(2000, 4, 30, 12, 5, 6, 7)
        );

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert_eq!(
            ts.add_interval_ym(-interval).unwrap(),
            generate_ts(1999, 2, 28, 12, 5, 6, 7)
        );

        // Boundary test
        let upper_ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let lower_ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);
        let interval = IntervalYM::try_from_ym(0, 1).unwrap();

        assert!(upper_ts.add_interval_ym(interval).is_err());
        assert!(lower_ts.sub_interval_ym(interval).is_err());
    }

    #[test]
    fn test_timestamp_add_sub_time() {
        // Normal test
        let ts = generate_ts(1234, 5, 6, 7, 8, 9, 10);
        let time = Time::try_from_hms(1, 2, 3, 4).unwrap();
        let expect = generate_ts(1234, 5, 6, 8, 10, 12, 14);
        assert_eq!(ts.add_time(time).unwrap(), expect);

        let expect = generate_ts(1234, 5, 6, 6, 6, 6, 6);
        assert_eq!(ts.sub_time(time).unwrap(), expect);

        let time = Time::try_from_hms(23, 59, 59, 999999).unwrap();
        let expect = generate_ts(1234, 5, 7, 7, 8, 9, 9);
        assert_eq!(ts.add_time(time).unwrap(), expect);

        let expect = generate_ts(1234, 5, 5, 7, 8, 9, 11);
        assert_eq!(ts.sub_time(time).unwrap(), expect);

        // Boundary test
        let ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let time = Time::try_from_hms(5, 4, 3, 2).unwrap();
        assert!(ts.add_time(time).is_err());

        let time = Time::try_from_hms(0, 0, 0, 1).unwrap();
        assert!(ts.add_time(time).is_err());

        let ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);
        let time = Time::try_from_hms(5, 4, 3, 2).unwrap();
        assert!(ts.sub_time(time).is_err());

        let time = Time::try_from_hms(0, 0, 0, 1).unwrap();
        assert!(ts.sub_time(time).is_err());
    }

    #[test]
    fn test_timestamp_sub_timestamp() {
        let upper_ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let lower_ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);
        let ts = generate_ts(5000, 6, 15, 12, 30, 30, 500000);

        assert_eq!(
            upper_ts.sub_timestamp(lower_ts),
            IntervalDT::try_from_dhms(3652058, 23, 59, 59, 999999).unwrap()
        );

        assert_eq!(
            upper_ts.sub_timestamp(ts),
            IntervalDT::try_from_dhms(1826046, 11, 29, 29, 499999).unwrap()
        );

        assert_eq!(
            lower_ts.sub_timestamp(upper_ts),
            -IntervalDT::try_from_dhms(3652058, 23, 59, 59, 999999).unwrap()
        );
    }

    #[test]
    fn test_timestamp_sub_date() {
        let upper_ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let lower_ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);
        let lower_date = Date::try_from_ymd(0001, 1, 1).unwrap();
        let upper_date = Date::try_from_ymd(9999, 12, 31).unwrap();
        let date = Date::try_from_ymd(5000, 6, 15).unwrap();

        assert_eq!(
            upper_ts.sub_date(lower_date),
            IntervalDT::try_from_dhms(3652058, 23, 59, 59, 999999).unwrap()
        );

        assert_eq!(
            upper_ts.sub_date(date),
            IntervalDT::try_from_dhms(1826046, 23, 59, 59, 999999).unwrap()
        );

        assert_eq!(
            lower_ts.sub_date(upper_date),
            -IntervalDT::try_from_dhms(3652058, 0, 0, 0, 0).unwrap()
        );
    }

    #[test]
    fn test_timestamp_add_sub_days() {
        let upper_ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let lower_ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);

        // Out of range
        assert!(lower_ts.add_days(213435445784784.123).is_err());
        assert!(lower_ts.add_days(f64::NAN).is_err());
        assert!(lower_ts.add_days(f64::INFINITY).is_err());
        assert!(lower_ts.add_days(f64::NEG_INFINITY).is_err());
        assert!(lower_ts.add_days(f64::MAX).is_err());
        assert!(lower_ts.add_days(f64::MIN).is_err());
        assert!(upper_ts.add_days(0.0001).is_err());

        assert!(lower_ts.sub_days(213435445784784.123).is_err());
        assert!(lower_ts.sub_days(f64::NAN).is_err());
        assert!(lower_ts.sub_days(f64::INFINITY).is_err());
        assert!(lower_ts.sub_days(f64::NEG_INFINITY).is_err());
        assert!(lower_ts.sub_days(f64::MAX).is_err());
        assert!(lower_ts.sub_days(f64::MIN).is_err());
        assert!(lower_ts.sub_days(0.0001).is_err());

        // Round
        assert_eq!(
            lower_ts.add_days(1.123456789).unwrap(),
            generate_ts(0001, 1, 2, 2, 57, 46, 666570)
        );
        assert_eq!(
            upper_ts.sub_days(1.123456789).unwrap(),
            generate_ts(9999, 12, 30, 21, 2, 13, 333429)
        );

        // Normal
        assert_eq!(upper_ts.sub_days(0.0).unwrap(), upper_ts);
        assert_eq!(upper_ts.add_days(0.0).unwrap(), upper_ts);
        assert_eq!(
            upper_ts.sub_days(1.0).unwrap(),
            generate_ts(9999, 12, 30, 23, 59, 59, 999999)
        );
        assert_eq!(
            lower_ts.add_days(1.0).unwrap(),
            generate_ts(0001, 1, 2, 0, 0, 0, 0)
        );

        let ts = generate_ts(5000, 6, 15, 12, 30, 30, 555555);
        assert_eq!(ts.sub_days(1.12).unwrap(), ts.add_days(-1.12).unwrap());
        assert_eq!(ts.sub_days(-1.12).unwrap(), ts.add_days(1.12).unwrap());
    }

    #[test]
    fn test_timestamp_cmp_date() {
        let ts = generate_ts(1970, 1, 1, 1, 1, 1, 1);
        let date = generate_date(1970, 1, 1);
        assert!(ts > date);
        let ts = generate_ts(1970, 1, 1, 0, 0, 0, 0);
        assert!(ts == date);
    }

    fn test_extract(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32, usec: u32) {
        let ts = generate_ts(year, month, day, hour, min, sec, usec);
        assert_eq!(year, ts.year().unwrap());
        assert_eq!(month as i32, ts.month().unwrap());
        assert_eq!(day as i32, ts.day().unwrap());
        assert_eq!(hour as i32, ts.hour().unwrap());
        assert_eq!(min as i32, ts.minute().unwrap());
        assert_eq!(
            (sec as f64 + (usec as f64) / 1_000_000f64),
            ts.second().unwrap()
        );
    }

    #[test]
    fn test_timestamp_extract() {
        test_extract(1960, 12, 31, 23, 59, 59, 999999);
        test_extract(0001, 1, 1, 0, 0, 0, 0);
        test_extract(0001, 1, 1, 1, 1, 1, 1);
        test_extract(1969, 12, 31, 1, 2, 3, 4);
        test_extract(1969, 12, 30, 23, 59, 59, 999999);
        test_extract(1969, 12, 30, 0, 0, 0, 0);
        test_extract(1970, 1, 1, 0, 0, 0, 0);
        test_extract(1970, 1, 1, 12, 30, 30, 30);
        test_extract(1999, 10, 21, 12, 30, 30, 30);
        test_extract(9999, 12, 31, 23, 59, 59, 999999);
    }
}
