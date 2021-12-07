//! Date implementation.

use crate::common::{
    date2julian, days_of_month, is_valid_date, julian2date, DATE_MAX_YEAR, DATE_MIN_YEAR,
    MONTHS_PER_YEAR, UNIX_EPOCH_JULIAN,
};
use crate::error::{Error, Result};
use crate::format::{Formatter, LazyFormat, NaiveDateTime};
use crate::{DateTime, IntervalDT, IntervalYM, Round, Time, Timestamp, Trunc};
use chrono::{Datelike, Local};
use std::cmp::Ordering;
use std::convert::TryFrom;
use std::fmt::Display;

type DateSubMethod = fn(Date, i32) -> Result<Date>;

pub const UNIX_EPOCH_DOW: WeekDay = WeekDay::Thursday;
const ROUNDS_UP_DAY: u32 = 16;
const ISO_YEAR_TABLE: [(DateSubMethod, i32); 8] = [
    (sub_to_date, 0), // Unreachable
    (sub_to_date, -1),
    (current_date, 0),
    (sub_to_date, 1),
    (sub_to_date, 2),
    (sub_to_date, 3),
    (sub_to_date, -3),
    (sub_to_date, -2),
];

/// Weekdays in the order of 1..=7 Sun..=Sat for formatting and calculation use
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum WeekDay {
    Sunday = 1,
    Monday = 2,
    Tuesday = 3,
    Wednesday = 4,
    Thursday = 5,
    Friday = 6,
    Saturday = 7,
}

impl From<usize> for WeekDay {
    /// Converts `usize` to `WeekDay` in the order of 1..=7 to Sunday..=Saturday
    ///
    /// # Panics
    /// Panics if `weekday` is out of range of 1..=7
    #[inline]
    fn from(weekday: usize) -> Self {
        use crate::date::WeekDay::*;
        const WEEKDAY_TABLE: [WeekDay; 7] = [
            Sunday, Monday, Tuesday, Wednesday, Thursday, Friday, Saturday,
        ];
        WEEKDAY_TABLE[weekday - 1]
    }
}

/// Months in the order of 1..=12 January..=December for formatting use
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum Month {
    January = 1,
    February = 2,
    March = 3,
    April = 4,
    May = 5,
    June = 6,
    July = 7,
    August = 8,
    September = 9,
    October = 10,
    November = 11,
    December = 12,
}

impl From<usize> for Month {
    /// Converts `usize` to `Month` in the order of 1..=12 to January..=December
    ///
    /// # Panics
    /// Panics if `month` is out of range of 1..=12
    #[inline]
    fn from(month: usize) -> Self {
        use crate::date::Month::*;
        const MONTH_TABLE: [Month; 12] = [
            January, February, March, April, May, June, July, August, September, October, November,
            December,
        ];
        MONTH_TABLE[month - 1]
    }
}

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
        if year < DATE_MIN_YEAR || year > DATE_MAX_YEAR {
            return Err(Error::DateOutOfRange);
        }

        if month < 1 || month > MONTHS_PER_YEAR {
            return Err(Error::InvalidMonth);
        }

        if day < 1 || day > 31 {
            return Err(Error::InvalidDay);
        }

        if day > days_of_month(year, month) {
            return Err(Error::InvalidDate);
        }

        Ok(unsafe { Date::from_ymd_unchecked(year, month, day) })
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

    /// Gets the days from Unix Epoch of this `Date`.
    #[inline(always)]
    pub const fn days(self) -> i32 {
        self.0
    }

    /// Creates a `Date` from the given days from Unix Epoch without checking validity.
    ///
    /// # Safety
    /// This function is unsafe because the day value is not checked for validity!
    /// Before using it, check that the value is correct.
    #[inline(always)]
    pub const unsafe fn from_days_unchecked(days: i32) -> Self {
        Date(days)
    }

    /// Creates a `Date` from the given days from Unix Epoch.
    #[inline]
    pub const fn try_from_days(days: i32) -> Result<Self> {
        if is_valid_date(days) {
            Ok(unsafe { Date::from_days_unchecked(days) })
        } else {
            Err(Error::DateOutOfRange)
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
        Ok(LazyFormat::new(fmt, self))
    }

    /// Parses `Date` from given string and format.
    #[inline]
    pub fn parse<S1: AsRef<str>, S2: AsRef<str>>(input: S1, fmt: S2) -> Result<Self> {
        let fmt = Formatter::try_new(fmt)?;
        fmt.parse(input)
    }

    /// Makes a new `Timestamp` from the current date and 00:00:00.
    #[inline(always)]
    pub(crate) const fn and_zero_time(self) -> Timestamp {
        Timestamp::new(self, Time::ZERO)
    }

    /// `Date` adds days.
    #[inline]
    pub const fn add_days(self, days: i32) -> Result<Date> {
        let result = self.days().checked_add(days);
        match result {
            Some(d) => Date::try_from_days(d),
            None => Err(Error::DateOutOfRange),
        }
    }

    #[inline]
    pub(crate) fn add_interval_ym_internal(self, interval: IntervalYM) -> Result<Date> {
        let (year, month, day) = self.extract();

        let mut new_month = month as i32 + interval.months();
        let mut new_year = year;

        if new_month > MONTHS_PER_YEAR as i32 {
            new_year += (new_month - 1) / MONTHS_PER_YEAR as i32;
            new_month = (new_month - 1) % MONTHS_PER_YEAR as i32 + 1;
        } else if new_month < 1 {
            new_year += new_month / MONTHS_PER_YEAR as i32 - 1;
            new_month = new_month % MONTHS_PER_YEAR as i32 + MONTHS_PER_YEAR as i32;
        }

        Date::try_from_ymd(new_year, new_month as u32, day)
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
        self.days() - date.days()
    }

    ///`Date` subtracts days.
    #[inline]
    pub const fn sub_days(self, days: i32) -> Result<Date> {
        let result = self.days().checked_sub(days);
        match result {
            Some(d) => Date::try_from_days(d),
            None => Err(Error::DateOutOfRange),
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

    /// Extract day of week (1..=7 Sunday..=Saturday)
    #[inline]
    pub fn day_of_week(self) -> WeekDay {
        // Add offset
        let mut date = self.days() + UNIX_EPOCH_DOW as i32 - 1;
        date %= 7;
        if date < 0 {
            date += 7;
        }
        // Change to 1..=7 (Sun..=Sat)
        WeekDay::from(date as usize + 1)
    }

    /// Get local system date
    #[inline]
    pub fn now() -> Result<Date> {
        let now = Local::now().naive_local();
        Date::try_from_ymd(now.year(), now.month(), now.day())
    }

    /// Converts date to ISO year.
    #[inline]
    fn date_to_iso_year(self) -> i32 {
        // Converts Julian date to day-of-week (0..6 == Mon..Sun)
        #[inline]
        fn week_day_of_julian(date: i32) -> i32 {
            let mut date = date;
            date %= 7;
            // Cope if division truncates towards zero, as it probably does
            if date < 0 {
                date += 7;
            }
            date
        }

        let mut year = self.year().unwrap();
        // current day
        let current_julian_day = self.days() + UNIX_EPOCH_JULIAN;
        // fourth day of current year
        let mut fourth_julian_day = date2julian(year, 1, 4);
        // offset to first day of week (Monday)
        let mut offset_to_monday = week_day_of_julian(fourth_julian_day);

        // We need the first week containing a Thursday, otherwise this day falls
        // into the previous year for purposes of counting weeks
        if current_julian_day < fourth_julian_day - offset_to_monday {
            fourth_julian_day = date2julian(year - 1, 1, 4);
            offset_to_monday = week_day_of_julian(fourth_julian_day);
            year -= 1;
        }

        // Sometimes the last few days in a year will fall into the first week of
        // the next year, so check for this
        let num_of_week = (current_julian_day - (fourth_julian_day - offset_to_monday)) / 7 + 1;
        if num_of_week >= 52 {
            fourth_julian_day = date2julian(year + 1, 1, 4);
            offset_to_monday = week_day_of_julian(fourth_julian_day);
            if current_julian_day >= fourth_julian_day - offset_to_monday {
                year += 1;
            }
        }

        year
    }
}

impl Trunc for Date {
    #[inline]
    fn trunc_century(self) -> Result<Self> {
        let mut year = self.year().unwrap();

        if year % 100 == 0 {
            year -= 1;
        }

        year = year / 100 * 100 + 1;
        Ok(unsafe { Date::from_ymd_unchecked(year, 1, 1) })
    }

    #[inline]
    fn trunc_year(self) -> Result<Self> {
        Ok(unsafe { Date::from_ymd_unchecked(self.year().unwrap(), 1, 1) })
    }

    #[inline]
    fn trunc_iso_year(self) -> Result<Self> {
        let iso_year = self.date_to_iso_year();
        let first_date = unsafe { Date::from_ymd_unchecked(iso_year, 1, 1) };
        let week_day = first_date.day_of_week() as usize;
        let (to_first_date_of_week, remain_day) = ISO_YEAR_TABLE[week_day];
        to_first_date_of_week(first_date, remain_day)
    }

    #[inline]
    fn trunc_quarter(self) -> Result<Self> {
        const QUARTER_FIRST_MONTH: [u32; 12] = [1, 1, 1, 4, 4, 4, 7, 7, 7, 10, 10, 10];

        let (year, month, _) = self.extract();
        let quarter_month = QUARTER_FIRST_MONTH[month as usize - 1];

        Ok(unsafe { Date::from_ymd_unchecked(year, quarter_month, 1) })
    }

    #[inline]
    fn trunc_month(self) -> Result<Self> {
        let (year, month, _) = self.extract();
        Ok(unsafe { Date::from_ymd_unchecked(year, month, 1) })
    }

    #[inline]
    fn trunc_week(self) -> Result<Self> {
        let trunc_day =
            self.sub_date(unsafe { Date::from_ymd_unchecked(self.year().unwrap(), 1, 1) }) % 7;
        let res_date = self.sub_days(trunc_day)?;
        Ok(res_date)
    }

    #[inline]
    fn trunc_iso_week(self) -> Result<Self> {
        const ISO_WEEK_TABLE: [(DateSubMethod, i32); 8] = [
            (sub_to_date, 0), // Unreachable
            (sub_to_date, 6),
            (current_date, 0),
            (sub_to_date, 1),
            (sub_to_date, 2),
            (sub_to_date, 3),
            (sub_to_date, 4),
            (sub_to_date, 5),
        ];

        let week_day = self.day_of_week() as usize;
        let (to_first_date_of_week, remain_day) = ISO_WEEK_TABLE[week_day];
        to_first_date_of_week(self, remain_day)
    }

    #[inline]
    fn trunc_month_start_week(self) -> Result<Self> {
        let remain_day = self.day().unwrap() % 7;
        let trunc_day = if remain_day == 0 { 6 } else { remain_day - 1 };
        let res_date = self.sub_days(trunc_day)?;
        Ok(res_date)
    }

    #[inline]
    fn trunc_day(self) -> Result<Self> {
        Ok(self)
    }

    #[inline]
    fn trunc_sunday_start_week(self) -> Result<Self> {
        let res_date = self.sub_days(self.day_of_week() as i32 - 1)?;
        Ok(res_date)
    }

    #[inline]
    fn trunc_hour(self) -> Result<Self> {
        Ok(self)
    }

    #[inline]
    fn trunc_minute(self) -> Result<Self> {
        Ok(self)
    }
}

#[inline(always)]
fn current_date(date: Date, _sub_day: i32) -> Result<Date> {
    Ok(date)
}

#[inline(always)]
fn sub_to_date(date: Date, sub_day: i32) -> Result<Date> {
    date.sub_days(sub_day)
}

impl Round for Date {
    #[inline]
    fn round_century(self) -> Result<Self> {
        let input_year = self.year().unwrap();
        if input_year > DATE_MAX_YEAR - 50 {
            return Err(Error::DateOutOfRange);
        }

        let mut century = input_year / 100;
        if input_year % 100 == 0 {
            century -= 1;
        } else if input_year % 100 > 50 {
            century += 1;
        }

        let res_year = century * 100 + 1;
        Ok(unsafe { Date::from_ymd_unchecked(res_year, 1, 1) })
    }

    #[inline]
    fn round_year(self) -> Result<Self> {
        let (mut year, month, _) = self.extract();
        if month >= 7 {
            if year == DATE_MAX_YEAR {
                return Err(Error::DateOutOfRange);
            }
            year += 1;
        }
        Ok(unsafe { Date::from_ymd_unchecked(year, 1, 1) })
    }

    #[inline]
    fn round_iso_year(self) -> Result<Self> {
        let (year, month, _) = self.extract();
        let mut date = self;
        if month >= 7 {
            if year == DATE_MAX_YEAR {
                return Err(Error::DateOutOfRange);
            }
            // Sets the month and date into the first week.
            date = unsafe { Date::from_ymd_unchecked(year + 1, 1, 4) };
        }
        date.trunc_iso_year()
    }

    #[inline]
    fn round_quarter(self) -> Result<Self> {
        const QUARTER_ROUND_MONTH: [u32; 12] = [1, 4, 4, 4, 7, 7, 7, 10, 10, 10, 1, 1];
        const QUARTER_TRUNC_MONTH: [u32; 12] = [1, 1, 4, 4, 4, 7, 7, 7, 10, 10, 10, 1];

        let (mut year, month, day) = self.extract();
        let is_round = day >= ROUNDS_UP_DAY;
        if year == DATE_MAX_YEAR && is_round {
            return Err(Error::DateOutOfRange);
        }

        let index = month as usize - 1;
        let quarter_month = if is_round {
            if month == 11 {
                year += 1;
            }
            QUARTER_ROUND_MONTH[index]
        } else {
            if month == 12 {
                year += 1;
            }
            QUARTER_TRUNC_MONTH[index]
        };

        Ok(unsafe { Date::from_ymd_unchecked(year, quarter_month, 1) })
    }

    #[inline]
    fn round_month(self) -> Result<Self> {
        let (mut year, mut month, day) = self.extract();
        if day >= ROUNDS_UP_DAY {
            if month == 12 {
                if year == DATE_MAX_YEAR {
                    return Err(Error::DateOutOfRange);
                }
                year += 1;
                month = 1;
            } else {
                month += 1;
            }
        }
        Ok(unsafe { Date::from_ymd_unchecked(year, month, 1) })
    }

    #[inline]
    fn round_week(self) -> Result<Self> {
        const WEEK_TABLE: [(DateSubMethod, i32); 8] = [
            (current_date, 0),
            (sub_to_date, 1),
            (sub_to_date, 2),
            (sub_to_date, 3),
            (sub_to_date, -3),
            (sub_to_date, -2),
            (sub_to_date, -1),
            (sub_to_date, 0), // Unreachable
        ];

        let week_day =
            self.sub_date(unsafe { Date::from_ymd_unchecked(self.year().unwrap(), 1, 1) }) % 7;
        let (to_first_date_of_week, remain_day) = WEEK_TABLE[week_day as usize];
        to_first_date_of_week(self, remain_day)
    }

    #[inline]
    fn round_iso_week(self) -> Result<Self> {
        const ISO_WEEK_TABLE: [(DateSubMethod, i32); 8] = [
            (sub_to_date, 0), // Unreachable
            (sub_to_date, -1),
            (current_date, 0),
            (sub_to_date, 1),
            (sub_to_date, 2),
            (sub_to_date, 3),
            (sub_to_date, -3),
            (sub_to_date, -2),
        ];

        let week_day = self.day_of_week() as usize;
        let (to_first_date_of_week, remain_day) = ISO_WEEK_TABLE[week_day];
        to_first_date_of_week(self, remain_day)
    }

    #[inline]
    fn round_month_start_week(self) -> Result<Self> {
        const MONTH_START_WEEK_TABLE: [(DateSubMethod, i32); 8] = [
            (sub_to_date, -1),
            (current_date, 0),
            (sub_to_date, 1),
            (sub_to_date, 2),
            (sub_to_date, 3),
            (sub_to_date, -3),
            (sub_to_date, -2),
            (sub_to_date, 0), // Unreachable
        ];

        let week_day = self.day().unwrap() % 7;
        let (to_first_date_of_week, remain_day) = MONTH_START_WEEK_TABLE[week_day as usize];
        to_first_date_of_week(self, remain_day)
    }

    #[inline]
    fn round_day(self) -> Result<Self> {
        Ok(self)
    }

    #[inline]
    fn round_sunday_start_week(self) -> Result<Self> {
        const SUNDAY_START_WEEK_TABLE: [(DateSubMethod, i32); 8] = [
            (sub_to_date, 0), // Unreachable
            (current_date, 0),
            (sub_to_date, 1),
            (sub_to_date, 2),
            (sub_to_date, 3),
            (sub_to_date, -3),
            (sub_to_date, -2),
            (sub_to_date, -1),
        ];

        let week_day = self.day_of_week() as usize;
        let (to_first_date_of_week, remain_day) = SUNDAY_START_WEEK_TABLE[week_day];
        to_first_date_of_week(self, remain_day)
    }

    #[inline]
    fn round_hour(self) -> Result<Self> {
        Ok(self)
    }

    #[inline]
    fn round_minute(self) -> Result<Self> {
        Ok(self)
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
        Some(self.and_zero_time().usecs().cmp(&other.usecs()))
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

impl DateTime for Date {
    #[inline]
    fn year(&self) -> Option<i32> {
        let (year, _, _) = self.extract();
        Some(year)
    }

    #[inline]
    fn month(&self) -> Option<i32> {
        let (_, month, _) = self.extract();
        Some(month as i32)
    }

    #[inline]
    fn day(&self) -> Option<i32> {
        let (_, _, day) = self.extract();
        Some(day as i32)
    }

    #[inline(always)]
    fn hour(&self) -> Option<i32> {
        None
    }

    #[inline(always)]
    fn minute(&self) -> Option<i32> {
        None
    }

    #[inline(always)]
    fn second(&self) -> Option<f64> {
        None
    }

    #[inline(always)]
    fn date(&self) -> Option<Date> {
        Some(*self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Local};

    #[test]
    fn test_date() {
        let date = Date::try_from_ymd(1970, 1, 1).unwrap();
        assert_eq!(date.days(), 0);
        assert_eq!(date.extract(), (1970, 1, 1));

        let date = Date::try_from_ymd(1, 1, 1).unwrap();
        assert_eq!(date.extract(), (1, 1, 1));

        let date = Date::try_from_ymd(9999, 12, 31).unwrap();
        assert_eq!(date.extract(), (9999, 12, 31));
        let date2 = Date::parse("9999-12-31", "YYYY-MM-DD").unwrap();
        assert_eq!(date2, date);

        // Out of order
        {
            // Parse

            let date = Date::try_from_ymd(9999, 12, 31).unwrap();
            let date2 = Date::parse("9999\\12-31", "yyyy\\mm-dd").unwrap();
            assert_eq!(date2, date);

            let date2 = Date::parse("9999-. 12--31", "YYYY-. MM--DD").unwrap();
            assert_eq!(date2, date);

            let date2 = Date::parse("31 9999-. 12", "dd YYYY-. MM").unwrap();
            assert_eq!(date, date2);

            let date2 = Date::parse("-12  31 -9999;", "-MM  DD -yyyy;").unwrap();
            assert_eq!(date, date2);

            // Format
            let fmt = date.format("\\YYYY\\ MM-/DD").unwrap();
            assert_eq!(format!("{}", fmt), "\\9999\\ 12-/31");

            let fmt = date.format("\\YYYY\\ MM-/DD;").unwrap();
            assert_eq!(format!("{}", fmt), "\\9999\\ 12-/31;");
        }

        // Duplicate parse
        {
            // todo All types duplication check, including parse and format
        }

        // Default
        {
            let now = Local::now().naive_local();
            let dt = Date::try_from_ymd(now.year(), now.month(), 1).unwrap();
            let date = Date::parse(" ", " ").unwrap();
            assert_eq!(date, dt);

            let date = Date::parse("", "").unwrap();
            assert_eq!(date, dt);
        }

        // Short format
        {
            let date = generate_date(1234, 8, 6);
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
        }

        // Normal
        {
            let date = generate_date(2000, 1, 1);
            let fmt = format!("{}", date.format("yyyy-MONTH-dd").unwrap());
            assert_eq!(fmt, "2000-JANUARY-01");

            let date = generate_date(2000, 1, 1);
            let fmt = format!("{}", date.format("yyyy-Mon-dd").unwrap());
            assert_eq!(fmt, "2000-Jan-01");

            let fmt = format!("{}", date.format("Day yyyy-Mon-dd").unwrap());
            assert_eq!(fmt, "Saturday 2000-Jan-01");

            let fmt = format!("{}", date.format("yyyyMMdd").unwrap());
            assert_eq!(fmt, "20000101");

            let date = generate_date(2001, 1, 2);
            assert_eq!(format!("{}", date.format("YYYYMMDD").unwrap()), "20010102");

            assert_eq!(date, Date::parse("20010102", "YYYYMMDD").unwrap());

            assert_eq!(date, Date::parse("2001012", "YYYYMMDD").unwrap());
        }

        // Day parse check
        {
            let date = generate_date(2021, 4, 22);
            let date2 = Date::parse("2021-04-22 thu", "yyyy-mm-dd dy").unwrap();
            assert_eq!(date, date2);

            assert!(Date::parse("2021-04-23 thur", "yyyy-mm-dd dy",).is_err());

            assert!(Date::parse("2021-04-27 tues", "yyyy-mm-dd dy",).is_err());
        }

        // Duplicate format
        {
            let date = generate_date(2021, 4, 25);
            assert_eq!(
                format!("{}", date.format("DAY DaY DY MM MM yyyy YYYY").unwrap()),
                "SUNDAY Sunday SUN 04 04 2021 2021"
            );
        }

        // Invalid
        {
            // Parse
            assert!(Date::parse("2021-04-22", "yyyy-mmX-dd",).is_err());
            assert!(Date::parse("2021-04-22", "yyy-mm-dd",).is_err());
            assert!(Date::parse("2021-04-32", "yyyy-mm-dd",).is_err());
            assert!(Date::parse("10000-04-30", "yyyy-mm-dd",).is_err());
            assert!(Date::parse("2021-04-22", "ABCD-mm-dd",).is_err());
            assert!(Date::parse("2021423", "yyyymmdd",).is_err());
            assert!(Date::parse("2021-04-22 11", "yyyy-mm-dd hh",).is_err());
            assert!(Date::parse("2021-04-22 11", "yyyy-mm-dd mi",).is_err());
            assert!(Date::parse("2021-04-22 11", "yyyy-mm-dd ss",).is_err());
            assert!(Date::parse("2021-04-22 11", "yyyy-mm-dd ff",).is_err());
        }
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
            Date::try_from_ymd(2, 1, 2).unwrap()
        );

        let date = Date::try_from_ymd(5000, 6, 15).unwrap();
        assert_eq!(
            date.add_days(718).unwrap(),
            Date::try_from_ymd(5002, 6, 3).unwrap()
        );
        assert_eq!(
            date.sub_days(718).unwrap(),
            Date::try_from_ymd(4998, 6, 27).unwrap()
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
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

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
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

        // Sub positive interval with carry test
        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        // Boundary test
        let date = generate_date(9999, 12, 31);
        let interval = IntervalDT::try_from_dhms(5, 4, 3, 2, 1).unwrap();
        let expect = generate_ts(9999, 12, 25, 19, 56, 57, 999999);
        assert_eq!(date.sub_interval_dt(interval).unwrap(), expect);

        let interval = IntervalDT::try_from_dhms(1, 0, 0, 0, 1).unwrap();
        assert!(date.add_interval_dt(interval).is_err());

        let interval = IntervalDT::try_from_dhms(12345, 12, 3, 5, 6).unwrap();
        assert!(date.add_interval_dt(interval).is_err());

        let date = generate_date(1, 1, 1);
        let interval = IntervalDT::try_from_dhms(5, 4, 3, 2, 1).unwrap();
        let expect = generate_ts(1, 1, 6, 4, 3, 2, 1);
        assert_eq!(date.add_interval_dt(interval).unwrap(), expect);

        let interval = IntervalDT::try_from_dhms(0, 0, 0, 0, 1).unwrap();
        assert!(date.sub_interval_dt(interval).is_err());

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

        // Boundary test
        let upper_date = generate_date(9999, 12, 31);
        let lower_date = generate_date(1, 1, 1);
        let interval = IntervalYM::try_from_ym(0, 1).unwrap();

        assert!(upper_date.add_interval_ym(interval).is_err());
        assert!(lower_date.sub_interval_ym(interval).is_err());

        // Month day overflow
        let date = generate_date(2001, 3, 31);
        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert!(date.add_interval_ym(interval).is_err());

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert!(date.add_interval_ym(interval).is_err());

        let interval = IntervalYM::try_from_ym(2, 11).unwrap();
        assert!(date.add_interval_ym(interval).is_err());

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert!(date.sub_interval_ym(-interval).is_err());

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert!(date.sub_interval_ym(-interval).is_err());

        let interval = IntervalYM::try_from_ym(2, 11).unwrap();
        assert!(date.sub_interval_ym(-interval).is_err());

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert!(date.sub_interval_ym(interval).is_err());

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert!(date.sub_interval_ym(interval).is_err());

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert!(date.sub_interval_ym(interval).is_err());

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert!(date.sub_interval_ym(interval).is_err());

        let interval = IntervalYM::try_from_ym(1, 1).unwrap();
        assert!(date.add_interval_ym(-interval).is_err());

        let interval = IntervalYM::try_from_ym(0, 11).unwrap();
        assert!(date.add_interval_ym(-interval).is_err());

        let interval = IntervalYM::try_from_ym(2, 1).unwrap();
        assert!(date.add_interval_ym(-interval).is_err());

        let date = generate_date(2000, 2, 29);
        let interval = IntervalYM::try_from_ym(2, 0).unwrap();
        assert!(date.add_interval_ym(interval).is_err());
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
            generate_ts(9999, 12, 30, 11, 25, 3, 1)
        );

        // Out of range
        assert!(Date::try_from_ymd(1, 1, 1)
            .unwrap()
            .sub_time(Time::try_from_hms(12, 34, 56, 999999).unwrap())
            .is_err());
    }

    #[test]
    fn test_date_sub_timestamp() {
        let upper_date = generate_date(9999, 12, 31);
        let lower_date = generate_date(1, 1, 1);
        let upper_ts = generate_ts(9999, 12, 31, 23, 59, 59, 999999);
        let lower_ts = generate_ts(1, 1, 1, 0, 0, 0, 0);
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
    fn test_date_sub_date() {
        let upper_date = generate_date(9999, 12, 31);
        let lower_date = generate_date(1, 1, 1);
        let date = generate_date(5000, 6, 15);

        assert_eq!(upper_date.sub_date(lower_date), 3652058);

        assert_eq!(lower_date.sub_date(upper_date), -3652058);

        assert_eq!(upper_date.sub_date(date), 1826046);
    }

    #[test]
    fn test_date_cmp_timestamp() {
        let ts = generate_ts(1970, 1, 1, 1, 1, 1, 1);
        let date = generate_date(1970, 1, 1);
        assert!(date < ts);
        let ts = generate_ts(1970, 1, 1, 0, 0, 0, 0);
        assert!(date == ts);
    }

    fn test_extract(year: i32, month: u32, day: u32) {
        let date = generate_date(year, month, day);
        assert_eq!(year, date.year().unwrap());
        assert_eq!(month as i32, date.month().unwrap());
        assert_eq!(day as i32, date.day().unwrap());

        assert!(date.hour().is_none());
        assert!(date.minute().is_none());
        assert!(date.second().is_none());
    }

    #[test]
    fn test_date_extract() {
        test_extract(1960, 12, 31);
        test_extract(1, 1, 1);
        test_extract(1969, 12, 31);
        test_extract(1969, 12, 30);
        test_extract(1970, 1, 1);
        test_extract(1999, 10, 21);
        test_extract(9999, 12, 31);
    }

    #[test]
    fn test_now() {
        let now = Local::now().naive_local();
        let dt = Date::now().unwrap();
        assert_eq!(now.year() as i32, dt.year().unwrap());
        assert_eq!(now.month() as i32, dt.month().unwrap());
        assert_eq!(now.day() as i32, dt.day().unwrap());
    }

    #[test]
    fn test_trunc() {
        let dt = generate_date(1996, 10, 24);

        assert_eq!(generate_date(1901, 1, 1), dt.trunc_century().unwrap());
        assert_eq!(generate_date(1996, 1, 1), dt.trunc_year().unwrap());

        // Test ISO Year
        // First date of range
        assert_eq!(
            generate_date(1, 1, 1),
            generate_date(1, 1, 1).trunc_iso_year().unwrap()
        );
        // First year of Julian date
        assert_eq!(
            generate_date(1583, 1, 3),
            generate_date(1583, 12, 31).trunc_iso_year().unwrap()
        );
        // Last date of range
        assert_eq!(
            generate_date(9999, 1, 4),
            generate_date(9999, 12, 31).trunc_iso_year().unwrap()
        );
        assert_eq!(generate_date(1996, 1, 1), dt.trunc_iso_year().unwrap());
        // Previous two years
        assert_eq!(
            generate_date(2019, 12, 30),
            generate_date(2021, 1, 3).trunc_iso_year().unwrap()
        );
        // Previous one year
        assert_eq!(
            generate_date(2018, 12, 31),
            generate_date(2019, 12, 29).trunc_iso_year().unwrap()
        );
        // Same year
        assert_eq!(
            generate_date(2019, 12, 30),
            generate_date(2019, 12, 31).trunc_iso_year().unwrap()
        );
        assert_eq!(
            generate_date(2018, 12, 31),
            generate_date(2018, 12, 31).trunc_iso_year().unwrap()
        );

        assert_eq!(generate_date(1996, 10, 1), dt.trunc_quarter().unwrap());
        assert_eq!(generate_date(1996, 10, 1), dt.trunc_month().unwrap());
        assert_eq!(generate_date(1996, 10, 21), dt.trunc_week().unwrap());
        assert_eq!(generate_date(1996, 10, 21), dt.trunc_iso_week().unwrap());
        assert_eq!(
            generate_date(1996, 10, 22),
            dt.trunc_month_start_week().unwrap()
        );
        assert_eq!(
            generate_date(1996, 10, 1),
            generate_date(1996, 10, 7).trunc_month_start_week().unwrap()
        );
        assert_eq!(generate_date(1996, 10, 24), dt.trunc_day().unwrap());
        assert_eq!(
            generate_date(1996, 10, 20),
            dt.trunc_sunday_start_week().unwrap()
        );
        assert_eq!(
            generate_date(2015, 4, 11),
            generate_date(2015, 4, 11).trunc_hour().unwrap()
        );
        assert_eq!(
            generate_date(2015, 4, 11),
            generate_date(2015, 4, 11).trunc_minute().unwrap()
        );
    }

    #[test]
    fn test_round_overflow() {
        let dt = generate_date(DATE_MAX_YEAR, 12, 31);
        assert!(dt.round_century().is_err());
        assert!(dt.round_year().is_err());
        assert!(dt.round_quarter().is_err());
        assert!(dt.round_month().is_err());
        assert!(dt.round_sunday_start_week().is_err());
    }

    #[test]
    fn test_round() {
        let dt = generate_date(1996, 10, 24);

        assert_eq!(generate_date(2001, 1, 1), dt.round_century().unwrap());
        assert_eq!(generate_date(1997, 1, 1), dt.round_year().unwrap());

        // Test ISO Year
        // First date of range
        assert_eq!(
            generate_date(1, 1, 1),
            generate_date(1, 1, 1).round_iso_year().unwrap()
        );
        assert_eq!(
            generate_date(1584, 1, 2),
            generate_date(1583, 12, 31).round_iso_year().unwrap()
        );
        assert_eq!(generate_date(1996, 12, 30), dt.round_iso_year().unwrap());
        // Previous two years
        assert_eq!(
            generate_date(2019, 12, 30),
            generate_date(2021, 1, 3).round_iso_year().unwrap()
        );
        // Same year
        assert_eq!(
            generate_date(2019, 12, 30),
            generate_date(2019, 12, 29).round_iso_year().unwrap()
        );
        assert_eq!(
            generate_date(2019, 12, 30),
            generate_date(2019, 12, 31).round_iso_year().unwrap()
        );
        assert_eq!(
            generate_date(2018, 12, 31),
            generate_date(2018, 12, 30).round_iso_year().unwrap()
        );
        assert_eq!(
            generate_date(2018, 12, 31),
            generate_date(2018, 12, 31).round_iso_year().unwrap()
        );
        // Next year
        assert_eq!(
            generate_date(2001, 1, 1),
            generate_date(2000, 12, 30).round_iso_year().unwrap()
        );
        assert_eq!(
            generate_date(2001, 1, 1),
            generate_date(2000, 12, 31).round_iso_year().unwrap()
        );

        assert_eq!(generate_date(1996, 10, 1), dt.round_quarter().unwrap());
        assert_eq!(
            generate_date(2022, 1, 1),
            generate_date(2021, 11, 16).round_quarter().unwrap()
        );
        assert_eq!(generate_date(1996, 11, 1), dt.round_month().unwrap());
        assert_eq!(
            generate_date(2021, 10, 15),
            generate_date(2021, 10, 13).round_week().unwrap()
        );
        assert_eq!(
            generate_date(2021, 10, 18),
            generate_date(2021, 10, 15).round_iso_week().unwrap()
        );
        assert_eq!(
            generate_date(2021, 11, 8),
            generate_date(2021, 11, 5).round_month_start_week().unwrap()
        );
        assert_eq!(
            generate_date(1996, 10, 24),
            generate_date(1996, 10, 24).round_day().unwrap()
        );
        assert_eq!(
            generate_date(1996, 10, 27),
            dt.round_sunday_start_week().unwrap()
        );
        assert_eq!(
            generate_date(2015, 3, 3),
            generate_date(2015, 3, 3).round_hour().unwrap()
        );
        assert_eq!(
            generate_date(2015, 3, 3),
            generate_date(2015, 3, 3).round_hour().unwrap()
        );
        assert_eq!(
            generate_date(2015, 3, 3),
            generate_date(2015, 3, 3).round_minute().unwrap()
        );
        assert_eq!(
            generate_date(2015, 3, 3),
            generate_date(2015, 3, 3).round_minute().unwrap()
        );
    }
}
