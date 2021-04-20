//! Date implementation.

use crate::common::{
    date2julian, days_of_month, is_valid_date, julian2date, DATE_MAX_YEAR, DATE_MIN_YEAR,
    MONTHS_PER_YEAR, UNIX_EPOCH_JULIAN,
};
use crate::error::{Error, Result};
use crate::format::{Formatter, LazyFormat, NaiveDateTime};
use crate::{IntervalDT, IntervalYM, Time, Timestamp};
use std::cmp::{min, Ordering};
use std::convert::TryFrom;
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

    #[inline]
    pub(crate) const fn try_from_value(value: i32) -> Result<Self> {
        if is_valid_date(value) {
            Ok(unsafe { Date::from_value_unchecked(value) })
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// Extracts `(year, month, day)` from the date.
    #[inline]
    pub const fn extract(self) -> (i32, u32, u32) {
        julian2date(self.0 + UNIX_EPOCH_JULIAN)
    }

    /// Makes a new `Timestamp` from the current date, hour, minute, second and microsecond.
    #[inline]
    pub fn and_hms(self, hour: u32, minute: u32, sec: u32, usec: u32) -> Result<Timestamp> {
        Ok(Timestamp::new(
            self,
            Time::try_from_hms(hour, minute, sec, usec)?,
        ))
    }

    /// Makes a new `Timestamp` from the current date and time.
    #[inline(always)]
    pub const fn and_time(self, time: Time) -> Timestamp {
        Timestamp::new(self, time)
    }

    /// Formats `Date` by given format string.
    #[inline]
    pub fn format<S: AsRef<str>>(self, fmt: S) -> Result<impl Display> {
        let fmt = Formatter::try_new(fmt)?;
        Ok(LazyFormat::new(fmt, self.into()))
    }

    /// Parses `Date` from given string and format.
    #[inline]
    pub fn parse<S1: AsRef<str>, S2: AsRef<str>>(input: S1, fmt: S2) -> Result<Self> {
        let fmt = Formatter::try_new(fmt)?;
        fmt.parse_date(input)
    }

    /// Makes a new `Timestamp` from the current date and 00:00:00.
    #[inline(always)]
    pub(crate) const fn and_zero_time(self) -> Timestamp {
        Timestamp::new(self, Time::ZERO)
    }

    /// `Date` adds days.
    #[inline]
    pub const fn add_days(self, days: i32) -> Result<Date> {
        let result = self.value().checked_add(days);
        match result {
            Some(d) => Date::try_from_value(d),
            None => Err(Error::OutOfRange),
        }
    }

    #[inline]
    pub(crate) fn add_interval_ym_internal(self, interval: IntervalYM) -> Result<Date> {
        let (year, month, day) = self.extract();

        let mut new_month = month as i32 + interval.value();
        let mut new_year = year;

        if new_month > MONTHS_PER_YEAR as i32 {
            new_year += (new_month - 1) / MONTHS_PER_YEAR as i32;
            new_month = (new_month - 1) % MONTHS_PER_YEAR as i32 + 1;
        } else if new_month < 1 {
            new_year += new_month / MONTHS_PER_YEAR as i32 - 1;
            new_month = new_month % MONTHS_PER_YEAR as i32 + MONTHS_PER_YEAR as i32;
        }

        let new_day = if day > 28 {
            min(days_of_month(new_year, new_month as u32), day)
        } else {
            day
        };

        Date::try_from_ymd(new_year, new_month as u32, new_day)
    }

    /// `Date` adds `IntervalYM`
    #[inline]
    pub fn add_interval_ym(self, interval: IntervalYM) -> Result<Timestamp> {
        Ok(self.add_interval_ym_internal(interval)?.and_zero_time())
    }

    /// `Date` adds `IntervalDT`
    #[inline]
    pub const fn add_interval_dt(self, interval: IntervalDT) -> Result<Timestamp> {
        self.and_zero_time().add_interval_dt(interval)
    }

    /// `Date` adds `Time`
    #[inline]
    pub const fn add_time(self, time: Time) -> Timestamp {
        self.and_time(time)
    }

    /// `Date` subtracts `Date`. Returns the difference in days between two `Date`
    #[inline]
    pub const fn sub_date(self, date: Date) -> i32 {
        self.value() - date.value()
    }

    ///`Date` subtracts days.
    #[inline]
    pub const fn sub_days(self, days: i32) -> Result<Date> {
        let result = self.value().checked_sub(days);
        match result {
            Some(d) => Date::try_from_value(d),
            None => Err(Error::OutOfRange),
        }
    }

    /// `Date` subtracts `Timestamp`
    #[inline]
    pub const fn sub_timestamp(self, timestamp: Timestamp) -> IntervalDT {
        self.and_zero_time().sub_timestamp(timestamp)
    }

    /// `Date` subtracts `IntervalYM`
    #[inline]
    pub fn sub_interval_ym(self, interval: IntervalYM) -> Result<Timestamp> {
        Ok(self.add_interval_ym_internal(-interval)?.and_zero_time())
    }

    /// `Date` subtracts `IntervalDT`
    #[inline]
    pub const fn sub_interval_dt(self, interval: IntervalDT) -> Result<Timestamp> {
        self.and_zero_time().sub_interval_dt(interval)
    }

    /// `Date` subtracts `Time`
    #[inline]
    pub const fn sub_time(self, time: Time) -> Result<Timestamp> {
        self.and_zero_time().sub_time(time)
    }
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

impl PartialEq<Timestamp> for Date {
    #[inline]
    fn eq(&self, other: &Timestamp) -> bool {
        self.and_zero_time() == *other
    }
}

impl PartialOrd<Timestamp> for Date {
    #[inline]
    fn partial_cmp(&self, other: &Timestamp) -> Option<Ordering> {
        Some(self.and_zero_time().value().cmp(&other.value()))
    }
}

impl TryFrom<&NaiveDateTime> for Date {
    type Error = Error;

    #[inline]
    fn try_from(dt: &NaiveDateTime) -> Result<Self> {
        Date::try_from_ymd(dt.year, dt.month, dt.day)
    }
}

impl TryFrom<NaiveDateTime> for Date {
    type Error = Error;

    #[inline]
    fn try_from(dt: NaiveDateTime) -> Result<Self> {
        Date::try_from(&dt)
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
        let date2 = Date::parse("9999-12-31", "YYYY-MM-DD").unwrap();
        assert_eq!(date2, date);
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
            Date::try_from_ymd(year, month, day).unwrap(),
            Time::try_from_hms(hour, min, sec, usec).unwrap(),
        )
    }

    fn generate_date(year: i32, month: u32, day: u32) -> Date {
        Date::try_from_ymd(year, month, day).unwrap()
    }

    #[test]
    fn test_add_sub_days() {
        let upper_date = Date::try_from_ymd(9999, 12, 31).unwrap();
        let lower_date = Date::try_from_ymd(1, 1, 1).unwrap();

        // Out of range
        assert!(lower_date.add_days(i32::MAX).is_err());
        assert!(lower_date.add_days(234253258).is_err());
        assert!(upper_date.add_days(1).is_err());
        assert!(upper_date.add_days(i32::MIN).is_err());

        assert!(lower_date.sub_days(1).is_err());
        assert!(upper_date.sub_days(234253258).is_err());
        assert!(lower_date.sub_days(i32::MAX).is_err());
        assert!(upper_date.sub_days(i32::MIN).is_err());

        // Normal
        assert_eq!(upper_date.sub_days(0).unwrap(), upper_date);
        assert_eq!(upper_date.add_days(0).unwrap(), upper_date);
        assert_eq!(
            upper_date.sub_days(366).unwrap(),
            Date::try_from_ymd(9998, 12, 30).unwrap()
        );
        assert_eq!(
            lower_date.add_days(366).unwrap(),
            Date::try_from_ymd(0002, 1, 2).unwrap()
        );

        let date = Date::try_from_ymd(5000, 6, 15).unwrap();
        assert_eq!(
            date.add_days(718).unwrap(),
            Date::try_from_ymd(5002, 06, 03).unwrap()
        );
        assert_eq!(
            date.sub_days(718).unwrap(),
            Date::try_from_ymd(4998, 06, 27).unwrap()
        );
        assert_eq!(date.sub_days(718).unwrap(), date.add_days(-718).unwrap());
        assert_eq!(date.sub_days(-718).unwrap(), date.add_days(718).unwrap());
    }

    #[test]
    fn test_and_time() {
        assert_eq!(
            Date::try_from_ymd(9999, 12, 31)
                .unwrap()
                .and_time(Time::try_from_hms(23, 59, 59, 999999).unwrap()),
            generate_ts(9999, 12, 31, 23, 59, 59, 999999)
        );

        assert_eq!(
            Date::try_from_ymd(1, 1, 1).unwrap().and_zero_time(),
            generate_ts(1, 1, 1, 0, 0, 0, 0)
        );

        assert!(Date::try_from_ymd(1, 1, 1)
            .unwrap()
            .and_hms(25, 6, 6, 7)
            .is_err());
    }

    #[test]
    fn test_date_add_sub_interval_dt() {
        // Normal add positive interval test
        let date = generate_date(2001, 3, 31);
        let interval = IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        let expect = generate_ts(2001, 4, 1, 2, 3, 4, 5);
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

        // Normal sub negative interval test
        let interval = -IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        // Add positive interval with carry test
        let date = generate_date(2001, 12, 31);
        let interval = IntervalDT::try_from_dhms(1, 0, 0, 0, 1).unwrap();
        let expect = generate_ts(2002, 1, 1, 0, 0, 0, 1);
        assert_eq!(date.clone().add_interval_dt(interval).unwrap(), expect);

        // Sub negative interval with carry test
        let interval = -IntervalDT::try_from_dhms(1, 0, 0, 0, 1).unwrap();
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        // Normal add negative interval test
        let date = generate_date(2001, 3, 31);
        let interval = -IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        let expect = generate_ts(2001, 3, 29, 21, 56, 55, 999995);
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

        // Normal sub positive interval test
        let interval = IntervalDT::try_from_dhms(1, 2, 3, 4, 5).unwrap();
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        // Add negative interval with carry test
        let date = generate_date(1970, 1, 1);
        let interval = -IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        let expect = generate_ts(1969, 12, 31, 23, 59, 59, 999999);
        assert_eq!(date.clone().add_interval_dt(interval).unwrap(), expect);

        // Sub positive interval with carry test
        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        // Boundary test
        let date = generate_date(9999, 12, 31);
        let interval = IntervalDT::try_from_dhms(5, 4, 3, 2, 1).unwrap();
        let expect = generate_ts(9999, 12, 25, 19, 56, 57, 999999);
        assert_eq!(date.clone().sub_interval_dt(interval).unwrap(), expect);

        let interval = IntervalDT::try_from_dhms(1, 0, 0, 0, 1).unwrap();
        assert!(date.clone().add_interval_dt(interval).is_err());

        let interval = IntervalDT::try_from_dhms(12345, 12, 3, 5, 6).unwrap();
        assert!(date.add_interval_dt(interval).is_err());

        let date = generate_date(0001, 1, 1);
        let interval = IntervalDT::try_from_dhms(5, 4, 3, 2, 1).unwrap();
        let expect = generate_ts(0001, 1, 6, 4, 3, 2, 1);
        assert_eq!(date.clone().add_interval_dt(interval).unwrap(), expect);

        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert!(date.clone().sub_interval_dt(interval).is_err());

        let interval = IntervalDT::try_from_dhms(12345, 12, 3, 5, 6).unwrap();
        assert!(date.sub_interval_dt(interval).is_err());
    }

    #[test]
    fn test_date_add_sub_interval_ym() {
        // Add positive
        let date = generate_date(2001, 3, 31);
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_ts(2001, 5, 31, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_ts(2002, 5, 31, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_ts(2002, 4, 30, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_ts(2002, 2, 28, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(2, 11).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_ts(2004, 2, 29, 0, 0, 0, 0)
        );

        // Sub negative
        let date = generate_date(2001, 3, 31);
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_ts(2001, 5, 31, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_ts(2002, 5, 31, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_ts(2002, 4, 30, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_ts(2002, 2, 28, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(2, 11).unwrap();
        assert_eq!(
            date.sub_interval_ym(-interval).unwrap(),
            generate_ts(2004, 2, 29, 0, 0, 0, 0)
        );

        // Sub positive
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_ts(2001, 1, 31, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_ts(2000, 1, 31, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_ts(2000, 2, 29, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_ts(2000, 4, 30, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_ts(1999, 2, 28, 0, 0, 0, 0)
        );

        // Add negative
        let interval = IntervalYM::try_from_ym(0, 2).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_ts(2001, 1, 31, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(1, 2).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_ts(2000, 1, 31, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_ts(2000, 2, 29, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_ts(2000, 4, 30, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert_eq!(
            date.add_interval_ym(-interval).unwrap(),
            generate_ts(1999, 2, 28, 0, 0, 0, 0)
        );

        let date = generate_date(2001, 2, 28);
        let interval = IntervalYM::try_from_ym(2, 0).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_ts(2003, 2, 28, 0, 0, 0, 0)
        );

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_ts(2003, 3, 28, 0, 0, 0, 0)
        );

        assert_eq!(
            date.sub_interval_ym(interval).unwrap(),
            generate_ts(1999, 1, 28, 0, 0, 0, 0)
        );

        let date = generate_date(2000, 2, 29);
        let interval = IntervalYM::try_from_ym(2, 0).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_ts(2002, 2, 28, 0, 0, 0, 0)
        );

        let interval = -IntervalYM::try_from_ym(0, 1).unwrap();
        assert_eq!(
            date.add_interval_ym(interval).unwrap(),
            generate_ts(2000, 1, 29, 0, 0, 0, 0)
        );

        // Boundary test
        let upper_date = generate_date(9999, 12, 31);
        let lower_date = generate_date(0001, 1, 1);
        let interval = IntervalYM::try_from_ym(0, 1).unwrap();

        assert!(upper_date.add_interval_ym(interval).is_err());
        assert!(lower_date.sub_interval_ym(interval).is_err());
    }

    #[test]
    fn test_add_sub_time() {
        assert_eq!(
            Date::try_from_ymd(9999, 12, 31)
                .unwrap()
                .add_time(Time::try_from_hms(12, 34, 56, 999999).unwrap()),
            generate_ts(9999, 12, 31, 12, 34, 56, 999999)
        );

        assert_eq!(
            Date::try_from_ymd(9999, 12, 31)
                .unwrap()
                .sub_time(Time::try_from_hms(12, 34, 56, 999999).unwrap())
                .unwrap(),
            generate_ts(9999, 12, 30, 11, 25, 03, 000001)
        );

        // Out of range
        assert!(Date::try_from_ymd(0001, 1, 1)
            .unwrap()
            .sub_time(Time::try_from_hms(12, 34, 56, 999999).unwrap())
            .is_err());
    }

    #[test]
    fn test_timestamp_sub_timestamp() {
        let upper_date = generate_date(9999, 12, 31);
        let lower_date = generate_date(0001, 1, 1);
        let upper_ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let lower_ts = generate_ts(0001, 1, 1, 0, 0, 0, 0);
        let ts = generate_ts(5000, 6, 15, 12, 30, 30, 500000);

        assert_eq!(
            upper_date.sub_timestamp(lower_ts),
            IntervalDT::try_from_dhms(3652058, 0, 0, 0, 0).unwrap()
        );

        assert_eq!(
            upper_date.sub_timestamp(ts),
            IntervalDT::try_from_dhms(1826045, 11, 29, 29, 500000).unwrap()
        );

        assert_eq!(
            lower_date.sub_timestamp(upper_ts),
            -IntervalDT::try_from_dhms(3652058, 23, 59, 59, 999999).unwrap()
        );
    }

    #[test]
    fn test_date_cmp_timestamp() {
        let ts = generate_ts(1970, 1, 1, 1, 1, 1, 1);
        let date = generate_date(1970, 1, 1);
        assert!(date < ts);
        let ts = generate_ts(1970, 1, 1, 0, 0, 0, 0);
        assert!(date == ts);
    }
}
