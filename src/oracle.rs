use crate::common::{is_valid_timestamp, USECONDS_PER_DAY, USECONDS_PER_SECOND};
use crate::error::{Error, Result};
use crate::format::{DateTimeFormat, LazyFormat, NaiveDateTime};
use crate::{Date as SqlDate, DateTime, Formatter, IntervalDT, IntervalYM, Time, Timestamp};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Display;

/// Oracle oriented `Date` type.
#[cfg_attr(docsrs, doc(cfg(feature = "oracle")))]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[repr(transparent)]
pub struct Date(Timestamp);

impl Date {
    /// The smallest date that can be represented by `Date`, i.e. `0001-01-01 00:00:00`.
    pub const MIN: Self = Date(Timestamp::MIN);

    /// The largest date that can be represented by `Date`, i.e. `9999-12-31 23:59:59`.
    pub const MAX: Self = Date(Timestamp::new(SqlDate::MAX, unsafe {
        Time::from_hms_unchecked(23, 59, 59, 0)
    }));

    /// Creates a new Oracle `Date` from a date and a time.
    #[inline]
    pub const fn new(date: SqlDate, time: Time) -> Self {
        let time = if time.usecs() % USECONDS_PER_SECOND != 0 {
            unsafe {
                Time::from_usecs_unchecked(time.usecs() / USECONDS_PER_SECOND * USECONDS_PER_SECOND)
            }
        } else {
            time
        };
        Date(Timestamp::new(date, time))
    }

    /// Gets the microsecond value from Unix Epoch of `Date`.
    #[inline(always)]
    pub const fn usecs(self) -> i64 {
        self.0.usecs()
    }

    /// Extracts `(Date, Time)` from the Oracle `Date`.
    #[inline]
    pub const fn extract(self) -> (SqlDate, Time) {
        self.0.extract()
    }

    #[inline]
    fn date(self) -> SqlDate {
        self.0.date()
    }

    #[inline]
    fn time(self) -> Time {
        self.0.time()
    }

    /// Creates a `Date` from the given microseconds from Unix Epoch without checking validity.
    ///
    /// # Safety
    /// This function is unsafe because the microsecond value is not checked for validity!
    /// Before using it, check that the value is correct.
    #[inline(always)]
    pub const unsafe fn from_usecs_unchecked(usecs: i64) -> Self {
        Date(Timestamp::from_usecs_unchecked(usecs))
    }

    /// Creates a `Date` from the given microseconds from Unix Epoch
    #[inline]
    pub const fn try_from_usecs(usecs: i64) -> Result<Self> {
        if Self::is_valid_date(usecs) {
            Ok(unsafe { Date(Timestamp::from_usecs_unchecked(usecs)) })
        } else {
            Err(Error::OutOfRange)
        }
    }

    #[inline]
    const fn is_valid_date(usecs: i64) -> bool {
        is_valid_timestamp(usecs) && usecs % USECONDS_PER_SECOND == 0
    }

    /// Formats `Date` by given format string.
    #[inline]
    pub fn format<S: AsRef<str>>(self, fmt: S) -> Result<impl Display> {
        let fmt = Formatter::try_new(fmt)?;
        Ok(LazyFormat::new(fmt, self))
    }

    /// Parses `Date` from given string and format.
    #[inline]
    pub fn parse<S1: AsRef<str>, S2: AsRef<str>>(input: S1, fmt: S2) -> Result<Self> {
        let fmt = Formatter::try_new(fmt)?;
        fmt.parse(input)
    }

    /// `Date` adds `IntervalDT`
    #[inline]
    pub fn add_interval_dt(self, interval: IntervalDT) -> Result<Date> {
        Ok(Date::from(self.0.add_interval_dt(interval)?))
    }

    /// `Date` adds `IntervalYM`
    #[inline]
    pub fn add_interval_ym(self, interval: IntervalYM) -> Result<Date> {
        Ok(Date::from(self.0.add_interval_ym(interval)?))
    }

    /// `Date` adds days
    #[inline]
    pub fn add_days(self, days: f64) -> Result<Date> {
        let timestamp = self.0.add_days(days)?;
        Ok(Date(Timestamp::try_from_usecs(
            ((timestamp.usecs() as f64) / USECONDS_PER_SECOND as f64).round() as i64
                * USECONDS_PER_SECOND,
        )?))
    }

    /// `Date` subtracts `Date`
    #[inline]
    pub fn sub_date(self, date: Date) -> f64 {
        (self.usecs() - date.usecs()) as f64 / USECONDS_PER_DAY as f64
    }

    /// `Date` subtracts `Timestamp`
    #[inline]
    pub fn sub_timestamp(self, timestamp: Timestamp) -> IntervalDT {
        self.0.sub_timestamp(timestamp)
    }

    /// `Date` subtracts `IntervalDT`
    #[inline]
    pub fn sub_interval_dt(self, interval: IntervalDT) -> Result<Date> {
        self.add_interval_dt(-interval)
    }

    /// `Date` subtracts `IntervalYM`
    #[inline]
    pub fn sub_interval_ym(self, interval: IntervalYM) -> Result<Date> {
        self.add_interval_ym(-interval)
    }

    /// `Date` subtracts days
    #[inline]
    pub fn sub_days(self, days: f64) -> Result<Date> {
        self.add_days(-days)
    }
}

impl Timestamp {
    /// `Timestamp` subtracts `Date`
    #[inline]
    pub const fn oracle_sub_date(self, date: Date) -> IntervalDT {
        self.sub_timestamp(date.0)
    }

    /// `Timestamp` add days
    #[inline]
    pub fn oracle_add_days(self, days: f64) -> Result<Date> {
        Date::from(self).add_days(days)
    }

    /// `Timestamp` subtracts days
    #[inline]
    pub fn oracle_sub_days(self, days: f64) -> Result<Date> {
        Date::from(self).add_days(-days)
    }
}

impl DateTime for Date {
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

impl From<Timestamp> for Date {
    #[inline]
    fn from(timestamp: Timestamp) -> Self {
        let usecs = timestamp.usecs();
        let temp = usecs / USECONDS_PER_SECOND * USECONDS_PER_SECOND;
        let result = if usecs < 0 && temp > usecs {
            temp - USECONDS_PER_SECOND
        } else {
            temp
        };

        unsafe { Date(Timestamp::from_usecs_unchecked(result)) }
    }
}

impl From<Date> for Timestamp {
    #[inline]
    fn from(input: Date) -> Self {
        input.0
    }
}

impl From<Date> for NaiveDateTime {
    #[inline]
    fn from(dt: Date) -> Self {
        let (date, time) = dt.extract();
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

impl TryFrom<NaiveDateTime> for Date {
    type Error = Error;

    #[inline]
    fn try_from(dt: NaiveDateTime) -> Result<Self> {
        Ok(Date::from(Timestamp::try_from(dt)?))
    }
}

impl DateTimeFormat for Date {}

impl PartialEq<Date> for Timestamp {
    #[inline]
    fn eq(&self, other: &Date) -> bool {
        *self == other.0
    }
}

impl PartialOrd<Date> for Timestamp {
    #[inline]
    fn partial_cmp(&self, other: &Date) -> Option<Ordering> {
        self.partial_cmp(&other.0)
    }
}

impl PartialEq<Timestamp> for Date {
    #[inline]
    fn eq(&self, other: &Timestamp) -> bool {
        self.0 == *other
    }
}

impl PartialOrd<Timestamp> for Date {
    #[inline]
    fn partial_cmp(&self, other: &Timestamp) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}

impl PartialEq<Date> for SqlDate {
    #[inline]
    fn eq(&self, other: &Date) -> bool {
        self.and_zero_time() == other.0
    }
}

impl PartialOrd<Date> for SqlDate {
    #[inline]
    fn partial_cmp(&self, other: &Date) -> Option<Ordering> {
        self.and_zero_time().partial_cmp(&other.0)
    }
}

impl PartialEq<SqlDate> for Date {
    #[inline]
    fn eq(&self, other: &SqlDate) -> bool {
        self.0 == other.and_zero_time()
    }
}

impl PartialOrd<SqlDate> for Date {
    #[inline]
    fn partial_cmp(&self, other: &SqlDate) -> Option<Ordering> {
        self.0.partial_cmp(&other.and_zero_time())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_date(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> Date {
        Date::new(
            SqlDate::try_from_ymd(year, month, day).unwrap(),
            Time::try_from_hms(hour, min, sec, 0).unwrap(),
        )
    }

    fn generate_ts(
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
        min: u32,
        sec: u32,
        usec: u32,
    ) -> Timestamp {
        Timestamp::new(
            SqlDate::try_from_ymd(year, month, day).unwrap(),
            Time::try_from_hms(hour, min, sec, usec).unwrap(),
        )
    }

    fn generate_sql_date(year: i32, month: u32, day: u32) -> SqlDate {
        SqlDate::try_from_ymd(year, month, day).unwrap()
    }

    fn generate_time(hour: u32, min: u32, sec: u32, usec: u32) -> Time {
        Time::try_from_hms(hour, min, sec, usec).unwrap()
    }

    #[test]
    fn test_date() {
        {
            let date = SqlDate::try_from_ymd(1970, 1, 1).unwrap();
            let time = Time::try_from_hms(1, 2, 3, 4).unwrap();
            let date = Date::new(date, time);
            assert_eq!(date.usecs(), generate_ts(1970, 1, 1, 1, 2, 3, 0).usecs());

            let date = SqlDate::try_from_ymd(1969, 1, 1).unwrap();
            let time = Time::try_from_hms(1, 2, 3, 4).unwrap();
            let date = Date::new(date, time);
            assert_eq!(date.usecs(), generate_ts(1969, 1, 1, 1, 2, 3, 0).usecs());

            let date = SqlDate::try_from_ymd(1970, 1, 1).unwrap();
            let time = Time::try_from_hms(0, 0, 0, 0).unwrap();
            let date = Date::new(date, time);
            assert_eq!(date.usecs(), 0);

            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1970, 1, 1));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let date = generate_date(1, 1, 1, 0, 0, 0);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1, 1, 1));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let date = generate_date(1, 1, 1, 23, 59, 59);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1, 1, 1));
            assert_eq!(time.extract(), (23, 59, 59, 0));

            let date = generate_date(1, 12, 31, 0, 0, 0);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1, 12, 31));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let date = generate_date(1, 12, 31, 23, 59, 59);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1, 12, 31));
            assert_eq!(time.extract(), (23, 59, 59, 0));

            let date = generate_date(1969, 12, 30, 0, 0, 0);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1969, 12, 30));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let date = generate_date(1969, 12, 30, 23, 59, 59);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1969, 12, 30));
            assert_eq!(time.extract(), (23, 59, 59, 0));

            let date = generate_date(1969, 12, 31, 0, 0, 0);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1969, 12, 31));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let date = generate_date(1969, 12, 31, 23, 59, 59);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1969, 12, 31));
            assert_eq!(time.extract(), (23, 59, 59, 0));

            let date = generate_date(1970, 1, 1, 0, 0, 0);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1970, 1, 1));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let date = generate_date(1970, 1, 1, 23, 59, 59);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1970, 1, 1));
            assert_eq!(time.extract(), (23, 59, 59, 0));

            let date = generate_date(1970, 3, 4, 23, 12, 30);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1970, 3, 4));
            assert_eq!(time.extract(), (23, 12, 30, 0));

            let date = generate_date(9999, 12, 31, 0, 0, 0);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (9999, 12, 31));
            assert_eq!(time.extract(), (0, 0, 0, 0));

            let date = generate_date(9999, 12, 31, 23, 59, 59);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (9999, 12, 31));
            assert_eq!(time.extract(), (23, 59, 59, 0));

            let date = generate_date(1969, 10, 31, 1, 1, 1);
            let (sql_date, time) = date.extract();
            assert_eq!(sql_date.extract(), (1969, 10, 31));
            assert_eq!(time.extract(), (1, 1, 1, 0));

            // Out of order
            {
                // Parse
                let date = generate_date(9999, 12, 31, 23, 59, 59);
                let ts2 =
                    Date::parse("PM 9999\\12-31 11/59:59", "AM yyyy\\mm-dd hh/mi:ss").unwrap();
                assert_eq!(ts2, date);

                let ts2 = Date::parse("PM 11-9999-59 12-59-31", "PM HH-YYYY-MI MM-SS-DD").unwrap();
                assert_eq!(ts2, date);

                let ts2 = Date::parse("23-9999-59 12 59 31", "HH24-YYYY-MI MM SS DD").unwrap();
                assert_eq!(date, ts2);

                let ts2 =
                    Date::parse("T23--59 12 59 31.9999;", "THH24--MI MM SS DD.YYYY;").unwrap();
                assert_eq!(date, ts2);

                // Format
                let fmt = date.format("TAM HH\\YYYY\\MI MM-SS/DD").unwrap();
                assert_eq!(format!("{}", fmt), "TPM 11\\9999\\59 12-59/31");

                let fmt = date.format("HH\\YYYY\\MI MM-SS/DD;").unwrap();
                assert_eq!(format!("{}", fmt), "11\\9999\\59 12-59/31;");
            }

            // Duplicate parse
            {
                // Parse
                assert!(
                    Date::parse("AM PM 9999\\12-31 11/59:59", "AM PM yyyy\\mm-dd hh/mi:ss")
                        .is_err()
                );

                assert!(
                    Date::parse("pm PM 9999\\12-31 11/59:59", "AM PM yyyy\\mm-dd hh/mi:ss")
                        .is_err()
                );

                assert!(Date::parse(
                    "9999 9999\\12-31 11/59:59.999999",
                    "yyyy yyyy\\mm-dd hh/mi:ss.ff"
                )
                .is_err());

                assert!(Date::parse("9999\\12-31 11/59:59 59", "yyyy\\mm-dd hh/mi:ss mi").is_err());

                // todo duplication special check, including parse and format
            }

            // Default
            {
                let date = generate_date(1, 1, 1, 0, 0, 5);
                let ts2 = Date::parse("5", "ss").unwrap();
                assert_eq!(date, ts2);

                let date = generate_date(1, 1, 1, 0, 0, 0);
                let ts2 = Date::parse("", "").unwrap();
                assert_eq!(date, ts2);

                let ts2 = Date::parse("jan", "MONTH").unwrap();
                assert_eq!(date, ts2);

                let ts2 = Date::parse("January", "mon").unwrap();
                assert_eq!(date, ts2)
            }

            // Short format
            {
                let date = generate_date(1234, 8, 6, 7, 8, 9);
                assert_eq!(format!("{}", date.format("YYYY").unwrap()), "1234");
                assert_eq!(format!("{}", date.format("DD").unwrap()), "06");
                assert_eq!(format!("{}", date.format("MON").unwrap()), "AUG");
                assert_eq!(format!("{}", date.format("Mon").unwrap()), "Aug");
                assert_eq!(format!("{}", date.format("mon").unwrap()), "aug");
                assert_eq!(format!("{}", date.format("MONTH").unwrap()), "AUGUST");
                assert_eq!(format!("{}", date.format("MONtH").unwrap()), "AUGUST");
                assert_eq!(format!("{}", date.format("Month").unwrap()), "August");
                assert_eq!(format!("{}", date.format("month").unwrap()), "august");
                assert_eq!(format!("{}", date.format("DAY").unwrap()), "SUNDAY");
                assert_eq!(format!("{}", date.format("DAy").unwrap()), "SUNDAY");
                assert_eq!(format!("{}", date.format("Day").unwrap()), "Sunday");
                assert_eq!(format!("{}", date.format("DaY").unwrap()), "Sunday");
                assert_eq!(format!("{}", date.format("day").unwrap()), "sunday");
                assert_eq!(format!("{}", date.format("daY").unwrap()), "sunday");
                assert_eq!(format!("{}", date.format("DY").unwrap()), "SUN");
                assert_eq!(format!("{}", date.format("Dy").unwrap()), "Sun");
                assert_eq!(format!("{}", date.format("dy").unwrap()), "sun");
                assert_eq!(format!("{}", date.format("mi").unwrap()), "08");
                assert_eq!(format!("{}", date.format("hh").unwrap()), "07");
                assert_eq!(format!("{}", date.format("ss").unwrap()), "09");

                let date = generate_date(1970, 1, 1, 7, 8, 9);
                assert_eq!(format!("{}", date.format("day").unwrap()), "thursday");

                let date = generate_date(1970, 1, 2, 7, 8, 9);
                assert_eq!(format!("{}", date.format("day").unwrap()), "friday");

                let date = generate_date(1969, 12, 31, 7, 8, 9);
                assert_eq!(format!("{}", date.format("day").unwrap()), "wednesday");

                let date = generate_date(1969, 10, 1, 7, 8, 9);
                assert_eq!(format!("{}", date.format("day").unwrap()), "wednesday");

                let date = generate_date(9999, 11, 14, 7, 8, 9);
                assert_eq!(format!("{}", date.format("day").unwrap()), "sunday");
            }

            // Normal
            {
                let date = generate_date(2000, 1, 1, 0, 0, 0);
                let fmt = format!("{}", date.format("yyyy-MONTH-dd hh:mi:ss").unwrap());
                assert_eq!(fmt, "2000-JANUARY-01 12:00:00");

                let fmt = format!("{}", date.format("yyyy-Mon-dd hh:mi:ss.ff1").unwrap());
                assert_eq!(fmt, "2000-Jan-01 12:00:00.0");

                let fmt = format!("{}", date.format("Day yyyy-Mon-dd hh:mi:ss.ff1").unwrap());
                assert_eq!(fmt, "Saturday 2000-Jan-01 12:00:00.0");

                let fmt = format!("{}", date.format("yyyyMMdd hh24miss.ff1").unwrap());
                assert_eq!(fmt, "20000101 000000.0");

                let date = generate_date(2001, 1, 2, 3, 4, 5);
                assert_eq!(
                    format!("{}", date.format("YYYYMMDDHHMISS").unwrap()),
                    "20010102030405"
                );

                assert_eq!(
                    date,
                    Date::parse("20010102030405", "YYYYMMDDHHMISS").unwrap()
                );

                assert_eq!(
                    date,
                    Date::parse("2001012 030405", "YYYYMMDD HHMISS").unwrap()
                );
            }

            // Day parse check
            {
                let date = generate_date(2021, 4, 22, 3, 4, 5);
                let ts2 =
                    Date::parse("2021-04-22 03:04:05 thu", "yyyy-mm-dd hh24:mi:ss dy").unwrap();
                assert_eq!(date, ts2);

                let ts2 = Date::parse("2021-04-22 03:04:05 thursday", "yyyy-mm-dd hh24:mi:ss dy")
                    .unwrap();
                assert_eq!(date, ts2);

                let ts2 =
                    Date::parse("2021-04-22 03:04:05 thu", "yyyy-mm-dd hh24:mi:ss day").unwrap();
                assert_eq!(date, ts2);

                let ts2 = Date::parse("2021-04-22 03:04:05 thursday", "yyyy-mm-dd hh24:mi:ss Dy")
                    .unwrap();
                assert_eq!(date, ts2);

                let ts2 =
                    Date::parse("2021-04-22 03:04:05 Thu", "yyyy-mm-dd hh24:mi:ss dy").unwrap();
                assert_eq!(date, ts2);

                assert!(
                    Date::parse("2021-04-23 03:04:05 thu", "yyyy-mm-dd hh24:mi:ss dy",).is_err()
                );
            }

            // Duplicate format
            {
                let date = generate_date(2021, 4, 25, 3, 4, 5);
                assert_eq!(
                    format!(
                        "{}",
                        date.format("DAY DaY DY MM MM yyyy YYYY MI MI").unwrap()
                    ),
                    "SUNDAY Sunday SUN 04 04 2021 2021 04 04"
                );
            }

            // Invalid
            {
                // Parse
                assert!(Date::parse("2021-04-22 03:04:05", "yyyy-mmX-dd hh24:mi:ss",).is_err());

                assert!(Date::parse("2021-04-22 03:04:05", "yyyy-mm-dd mi:ss",).is_err());

                assert!(Date::parse("2021-04-22 03:04:05", "yyy-mm-dd hh24:mi:ss",).is_err());

                assert!(Date::parse("2021-04-32 03:04:05", "yyyy-mm-dd mi:ss",).is_err());

                assert!(Date::parse("10000-04-31 03:04:05", "yyyy-mm-dd mi:ss",).is_err());

                assert!(Date::parse("10000-04-31 33:04:05", "yyyy-mm-dd mi:ss",).is_err());

                assert!(Date::parse("2021-04-22 03:04:05", "ABCD-mm-dd hh24:mi:ss",).is_err());

                assert!(
                    Date::parse("2021-04-23 03:04:05 thur", "yyyy-mm-dd hh24:mi:ss dy",).is_err()
                );

                assert!(Date::parse("2021423 03:04:05", "yyyymmdd hh24:mi:ss",).is_err());

                assert!(Date::parse("2021423 03:04:05", "yyyymmdd hh24:mi:ss",).is_err());

                let date = generate_date(1234, 5, 6, 7, 8, 9);
                assert!(date.format("testtest").is_err());
            }

            // todo
            // Wrong order of some specific Field, wrong format, extra format.
        }
    }

    #[test]
    fn test_date_to_sql_date_time() {
        let date = generate_date(1, 1, 1, 0, 0, 0);
        assert_eq!(date.date(), generate_sql_date(1, 1, 1));
        assert_eq!(date.time(), generate_time(0, 0, 0, 0));

        let date = generate_date(1, 1, 1, 23, 59, 59);
        assert_eq!(date.date(), generate_sql_date(1, 1, 1));
        assert_eq!(date.time(), generate_time(23, 59, 59, 0));

        let date = generate_date(1969, 12, 30, 0, 0, 0);
        assert_eq!(date.date(), generate_sql_date(1969, 12, 30));
        assert_eq!(date.time(), generate_time(0, 0, 0, 0));

        let date = generate_date(1969, 12, 30, 23, 59, 59);
        assert_eq!(date.date(), generate_sql_date(1969, 12, 30));
        assert_eq!(date.time(), generate_time(23, 59, 59, 0));

        let date = generate_date(1969, 12, 31, 0, 0, 0);
        assert_eq!(date.date(), generate_sql_date(1969, 12, 31));
        assert_eq!(date.time(), generate_time(0, 0, 0, 0));

        let date = generate_date(1969, 12, 31, 23, 59, 59);
        assert_eq!(date.date(), generate_sql_date(1969, 12, 31));
        assert_eq!(date.time(), generate_time(23, 59, 59, 0));

        let date = generate_date(1970, 1, 1, 0, 0, 0);
        assert_eq!(date.date(), generate_sql_date(1970, 1, 1));
        assert_eq!(date.time(), generate_time(0, 0, 0, 0));

        let date = generate_date(1970, 1, 1, 23, 59, 59);
        assert_eq!(date.date(), generate_sql_date(1970, 1, 1));
        assert_eq!(date.time(), generate_time(23, 59, 59, 0));

        let date = generate_date(9999, 1, 1, 0, 0, 0);
        assert_eq!(date.date(), generate_sql_date(9999, 1, 1));
        assert_eq!(date.time(), generate_time(0, 0, 0, 0));

        let date = generate_date(9999, 1, 1, 23, 59, 59);
        assert_eq!(date.date(), generate_sql_date(9999, 1, 1));
        assert_eq!(date.time(), generate_time(23, 59, 59, 0));

        let date = generate_date(9999, 12, 31, 0, 0, 0);
        assert_eq!(date.date(), generate_sql_date(9999, 12, 31));
        assert_eq!(date.time(), generate_time(0, 0, 0, 0));

        let date = generate_date(9999, 12, 31, 23, 59, 59);
        assert_eq!(date.date(), generate_sql_date(9999, 12, 31));
        assert_eq!(date.time(), generate_time(23, 59, 59, 0));
    }

    #[test]
    fn test_date_add_sub_interval_dt() {
        // Normal add positive interval test
        let date = generate_date(2001, 3, 31, 12, 5, 6);
        let interval = IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        let expect = generate_date(2001, 4, 1, 14, 8, 10);
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

        // Normal sub negative interval test
        let interval = -IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        // Add positive interval with carry test
        let date = generate_date(2001, 12, 31, 23, 59, 59);
        let interval = IntervalDT::try_from_dhms(0, 0, 0, 1, 1).unwrap();
        let expect = generate_date(2002, 1, 1, 0, 0, 0);
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

        // Sub negative interval with carry test
        let interval = -IntervalDT::try_from_dhms(0, 0, 0, 1, 1).unwrap();
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        // Normal add negative interval test
        let date = generate_date(2001, 3, 31, 12, 5, 6);
        let interval = -IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        let expect = generate_date(2001, 3, 30, 10, 2, 1);
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

        // Normal sub positive interval test
        let interval = IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        // Add negative interval with carry test
        let date = generate_date(1970, 1, 1, 0, 0, 0);
        let interval = -IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        let expect = generate_date(1969, 12, 31, 23, 59, 59);
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

        // Sub positive interval with carry test
        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        // Boundary test
        let date = generate_date(9999, 12, 31, 23, 59, 59);
        let interval = IntervalDT::try_from_dhms(5, 4, 3, 2, 1).unwrap();
        let expect = generate_date(9999, 12, 26, 19, 56, 56);
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        let interval = IntervalDT::try_from_dhms(0, 0, 0, 1, 0).unwrap();
        assert!(date.add_interval_dt(interval).is_err());

        let interval = IntervalDT::try_from_dhms(12345, 12, 3, 5, 6).unwrap();
        assert!(date.add_interval_dt(interval).is_err());

        let date = generate_date(1, 1, 1, 0, 0, 0);
        let interval = IntervalDT::try_from_dhms(5, 4, 3, 2, 1).unwrap();
        let expect = generate_date(1, 1, 6, 4, 3, 2);
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert!(date.sub_interval_dt(interval).is_err());

        let interval = IntervalDT::try_from_dhms(12345, 12, 3, 5, 6).unwrap();
        assert!(date.sub_interval_dt(interval).is_err());
    }

    #[test]
    fn test_date_add_sub_interval_ym() {
        // Add positive
        let date = generate_date(2001, 3, 31, 12, 5, 6);
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_date(2001, 5, 31, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_date(2002, 5, 31, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_date(2002, 4, 30, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_date(2002, 2, 28, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(2, 11).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_date(2004, 2, 29, 12, 5, 6)
        );

        // Sub negative
        let date = generate_date(2001, 3, 31, 12, 5, 6);
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_date(2001, 5, 31, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_date(2002, 5, 31, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_date(2002, 4, 30, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_date(2002, 2, 28, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(2, 11).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_date(2004, 2, 29, 12, 5, 6)
        );

        // Sub positive
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_date(2001, 1, 31, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_date(2000, 1, 31, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_date(2000, 2, 29, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_date(2000, 4, 30, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_date(1999, 2, 28, 12, 5, 6)
        );

        // Add negative
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_date(2001, 1, 31, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_date(2000, 1, 31, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_date(2000, 2, 29, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_date(2000, 4, 30, 12, 5, 6)
        );

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_date(1999, 2, 28, 12, 5, 6)
        );

        // Boundary test
        let upper_ts = generate_date(9999, 12, 31, 23, 59, 59);
        let lower_ts = generate_date(1, 1, 1, 0, 0, 0);
        let interval = IntervalYM::try_from_ym(0, 1).unwrap();

        assert!(upper_ts.add_interval_ym(interval).is_err());
        assert!(lower_ts.sub_interval_ym(interval).is_err());
    }

    #[test]
    fn test_date_sub_date() {
        let upper_ts = generate_date(9999, 12, 31, 23, 59, 59);
        let lower_ts = generate_date(1, 1, 1, 0, 0, 0);
        let date = generate_date(5000, 6, 15, 12, 30, 30);
        dbg!(upper_ts.sub_date(lower_ts));
        dbg!(upper_ts.sub_date(date));
        dbg!(lower_ts.sub_date(upper_ts));
    }

    #[test]
    fn test_date_sub_timestamp() {
        let upper_ts = generate_date(9999, 12, 31, 23, 59, 59);
        let lower_ts = generate_date(1, 1, 1, 0, 0, 0);
        let lower_timestamp = generate_ts(1, 1, 1, 0, 0, 0, 0);
        let upper_timestamp = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let timestamp = generate_ts(5000, 6, 15, 12, 30, 30, 500000);

        assert_eq!(
            upper_ts.sub_timestamp(lower_timestamp),
            IntervalDT::try_from_dhms(3652058, 23, 59, 59, 000000).unwrap()
        );

        assert_eq!(
            upper_ts.sub_timestamp(timestamp),
            IntervalDT::try_from_dhms(1826046, 11, 29, 28, 500000).unwrap()
        );

        assert_eq!(
            lower_ts.sub_timestamp(upper_timestamp),
            -IntervalDT::try_from_dhms(3652058, 23, 59, 59, 999999).unwrap()
        );
    }

    #[test]
    fn test_date_add_sub_days() {
        let upper_ts = generate_date(9999, 12, 31, 23, 59, 59);
        let lower_ts = generate_date(1, 1, 1, 0, 0, 0);

        // Out of range
        assert!(lower_ts.add_days(213435445784784.13).is_err());
        assert!(lower_ts.add_days(f64::NAN).is_err());
        assert!(lower_ts.add_days(f64::INFINITY).is_err());
        assert!(lower_ts.add_days(f64::NEG_INFINITY).is_err());
        assert!(lower_ts.add_days(f64::MAX).is_err());
        assert!(lower_ts.add_days(f64::MIN).is_err());
        assert!(upper_ts.add_days(0.0001).is_err());

        assert!(lower_ts.sub_days(213435445784784.13).is_err());
        assert!(lower_ts.sub_days(f64::NAN).is_err());
        assert!(lower_ts.sub_days(f64::INFINITY).is_err());
        assert!(lower_ts.sub_days(f64::NEG_INFINITY).is_err());
        assert!(lower_ts.sub_days(f64::MAX).is_err());
        assert!(lower_ts.sub_days(f64::MIN).is_err());
        assert!(lower_ts.sub_days(0.0001).is_err());

        // Round
        assert_eq!(
            lower_ts.add_days(1.123456789).unwrap(),
            generate_date(1, 1, 2, 2, 57, 47)
        );
        assert_eq!(
            upper_ts.sub_days(1.123456789).unwrap(),
            generate_date(9999, 12, 30, 21, 2, 12)
        );

        // Normal
        assert_eq!(upper_ts.sub_days(0.0).unwrap(), upper_ts);
        assert_eq!(upper_ts.add_days(0.0).unwrap(), upper_ts);
        assert_eq!(
            upper_ts.sub_days(1.0).unwrap(),
            generate_date(9999, 12, 30, 23, 59, 59)
        );
        assert_eq!(
            lower_ts.add_days(1.0).unwrap(),
            generate_date(1, 1, 2, 0, 0, 0)
        );

        let date = generate_date(5000, 6, 15, 12, 30, 30);
        assert_eq!(
            date.sub_days(1.12).unwrap(),
            generate_date(5000, 6, 14, 9, 37, 42)
        );
        assert_eq!(
            date.add_days(1.12).unwrap(),
            generate_date(5000, 6, 16, 15, 23, 18)
        );
        assert_eq!(date.sub_days(1.12).unwrap(), date.add_days(-1.12).unwrap());
        assert_eq!(date.sub_days(-1.12).unwrap(), date.add_days(1.12).unwrap());
    }

    #[test]
    fn test_date_cmp_timestamp() {
        let date = generate_date(1970, 1, 1, 1, 1, 1);
        let timestamp = generate_ts(1971, 1, 1, 12, 4, 5, 0);
        assert!(date < timestamp);
        let date = generate_date(1971, 1, 1, 12, 4, 5);
        assert!(date == timestamp);
    }

    #[allow(clippy::float_cmp)]
    fn test_extract(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) {
        let date = generate_date(year, month, day, hour, min, sec);
        assert_eq!(year, date.year().unwrap());
        assert_eq!(month as i32, date.month().unwrap());
        assert_eq!(day as i32, date.day().unwrap());
        assert_eq!(hour as i32, date.hour().unwrap());
        assert_eq!(min as i32, date.minute().unwrap());
        assert_eq!(sec as f64, date.second().unwrap());
    }

    #[test]
    fn test_timestamp_extract() {
        test_extract(1960, 12, 31, 23, 59, 59);
        test_extract(1, 1, 1, 0, 0, 0);
        test_extract(1, 1, 1, 1, 1, 1);
        test_extract(1969, 12, 31, 1, 2, 3);
        test_extract(1969, 12, 30, 23, 59, 59);
        test_extract(1969, 12, 30, 0, 0, 0);
        test_extract(1970, 1, 1, 0, 0, 0);
        test_extract(1970, 1, 1, 12, 30, 30);
        test_extract(1999, 10, 21, 12, 30, 30);
        test_extract(9999, 12, 31, 23, 59, 59);
    }

    #[test]
    fn test_from_timestamp() {
        let timestamp = generate_ts(1969, 12, 31, 23, 0, 0, 0);
        assert_eq!(Date::from(timestamp), generate_date(1969, 12, 31, 23, 0, 0));

        let timestamp = generate_ts(1970, 1, 1, 0, 0, 0, 0);
        assert_eq!(Date::from(timestamp), generate_date(1970, 1, 1, 0, 0, 0));

        let timestamp = generate_ts(1969, 12, 30, 0, 0, 1, 0);
        assert_eq!(Date::from(timestamp), generate_date(1969, 12, 30, 0, 0, 1));

        let timestamp = generate_ts(1969, 12, 31, 0, 0, 0, 0);
        assert_eq!(Date::from(timestamp), generate_date(1969, 12, 31, 0, 0, 0));

        let timestamp = generate_ts(1970, 1, 1, 0, 0, 1, 0);
        assert_eq!(Date::from(timestamp), generate_date(1970, 1, 1, 0, 0, 1));

        let timestamp = generate_ts(9999, 12, 31, 23, 59, 59, 0);
        assert_eq!(
            Date::from(timestamp),
            generate_date(9999, 12, 31, 23, 59, 59)
        );

        let timestamp = generate_ts(1, 1, 1, 0, 0, 0, 0);
        assert_eq!(Date::from(timestamp), generate_date(1, 1, 1, 0, 0, 0));

        let timestamp = generate_ts(1, 1, 1, 0, 0, 0, 1);
        assert_eq!(Date::from(timestamp), generate_date(1, 1, 1, 0, 0, 0));

        let timestamp = generate_ts(1, 1, 1, 0, 0, 0, 999999);
        assert_eq!(Date::from(timestamp), generate_date(1, 1, 1, 0, 0, 0));

        let timestamp = generate_ts(2000, 1, 1, 0, 0, 0, 999999);
        assert_eq!(Date::from(timestamp), generate_date(2000, 1, 1, 0, 0, 0));

        let timestamp = generate_ts(2000, 1, 1, 0, 0, 0, 1);
        assert_eq!(Date::from(timestamp), generate_date(2000, 1, 1, 0, 0, 0));
    }

    #[test]
    fn test_timestamp_sub_date() {
        let upper_ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let lower_ts = generate_ts(1, 1, 1, 0, 0, 0, 0);
        let upper_date = generate_date(9999, 12, 31, 23, 59, 59);
        let lower_date = generate_date(1, 1, 1, 0, 0, 0);
        let date = generate_date(5000, 6, 15, 12, 30, 30);

        assert_eq!(
            upper_ts.oracle_sub_date(lower_date),
            IntervalDT::try_from_dhms(3652058, 23, 59, 59, 999999).unwrap()
        );

        assert_eq!(
            upper_ts.oracle_sub_date(date),
            IntervalDT::try_from_dhms(1826046, 11, 29, 29, 999999).unwrap()
        );

        assert_eq!(
            lower_ts.oracle_sub_date(upper_date),
            -IntervalDT::try_from_dhms(3652058, 23, 59, 59, 0).unwrap()
        );
    }

    #[test]
    fn test_timestamp_add_sub_days() {
        let upper_ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let lower_ts = generate_ts(1, 1, 1, 0, 0, 0, 0);
        let upper_date = generate_date(9999, 12, 31, 23, 59, 59);

        // Out of range
        assert!(lower_ts.oracle_add_days(213435445784784.13).is_err());
        assert!(lower_ts.oracle_add_days(f64::NAN).is_err());
        assert!(lower_ts.oracle_add_days(f64::INFINITY).is_err());
        assert!(lower_ts.oracle_add_days(f64::NEG_INFINITY).is_err());
        assert!(lower_ts.oracle_add_days(f64::MAX).is_err());
        assert!(lower_ts.oracle_add_days(f64::MIN).is_err());
        assert!(upper_ts.oracle_add_days(0.0001).is_err());

        assert!(lower_ts.oracle_sub_days(213435445784784.13).is_err());
        assert!(lower_ts.oracle_sub_days(f64::NAN).is_err());
        assert!(lower_ts.oracle_sub_days(f64::INFINITY).is_err());
        assert!(lower_ts.oracle_sub_days(f64::NEG_INFINITY).is_err());
        assert!(lower_ts.oracle_sub_days(f64::MAX).is_err());
        assert!(lower_ts.oracle_sub_days(f64::MIN).is_err());
        assert!(lower_ts.oracle_sub_days(0.0001).is_err());

        // Round
        assert_eq!(
            lower_ts.oracle_add_days(1.123456789).unwrap(),
            generate_date(1, 1, 2, 2, 57, 47)
        );
        assert_eq!(
            upper_ts.oracle_sub_days(1.123456789).unwrap(),
            generate_date(9999, 12, 30, 21, 2, 12)
        );

        // Normal
        assert_eq!(upper_ts.oracle_sub_days(0.0).unwrap(), upper_date);
        assert_eq!(upper_ts.oracle_add_days(0.0).unwrap(), upper_date);
        assert_eq!(
            upper_ts.oracle_sub_days(1.0).unwrap(),
            generate_date(9999, 12, 30, 23, 59, 59)
        );
        assert_eq!(
            lower_ts.add_days(1.0).unwrap(),
            generate_date(1, 1, 2, 0, 0, 0)
        );

        let ts = generate_ts(5000, 6, 15, 12, 30, 30, 555555);
        assert_eq!(
            ts.oracle_sub_days(1.12).unwrap(),
            generate_date(5000, 6, 14, 9, 37, 42)
        );
        assert_eq!(
            ts.oracle_sub_days(1.12).unwrap(),
            ts.oracle_add_days(-1.12).unwrap()
        );
        assert_eq!(
            ts.oracle_add_days(1.12).unwrap(),
            generate_date(5000, 6, 16, 15, 23, 18)
        );
        assert_eq!(
            ts.oracle_sub_days(-1.12).unwrap(),
            ts.oracle_add_days(1.12).unwrap()
        );

        let ts = generate_ts(1, 1, 1, 0, 0, 0, 8);
        assert_eq!(
            ts.oracle_add_days(0.00000578).unwrap(),
            generate_date(1, 1, 1, 0, 0, 0)
        );

        let ts = generate_ts(1971, 1, 1, 0, 0, 0, 8);
        assert_eq!(
            ts.oracle_add_days(0.00000578).unwrap(),
            generate_date(1971, 1, 1, 0, 0, 0)
        );
    }
}
