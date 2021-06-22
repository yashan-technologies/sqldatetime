//! Formatting (and parsing) utilities for date and time.

use crate::common::DATE_MIN_YEAR;
use crate::date::{Month, WeekDay};
use crate::error::Result;
use crate::format::NameStyle::{AbbrCapital, Capital};
use crate::{Date, DateTime, Error, IntervalDT, IntervalYM, Time, Timestamp};
use stack_buf::StackVec;
use std::convert::TryFrom;
use std::fmt;

const MAX_FIELDS: usize = 32;

const FRACTION_FACTOR: [f64; 10] = [
    1000000.0, 100000.0, 10000.0, 1000.0, 100.0, 10.0, 1.0, 0.1, 0.01, 0.001,
];

pub const MONTH_NAME_TABLE: [[&str; 12]; 6] = [
    [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ],
    [
        "january",
        "february",
        "march",
        "april",
        "may",
        "june",
        "july",
        "august",
        "september",
        "october",
        "november",
        "december",
    ],
    [
        "JANUARY",
        "FEBRUARY",
        "MARCH",
        "APRIL",
        "MAY",
        "JUNE",
        "JULY",
        "AUGUST",
        "SEPTEMBER",
        "OCTOBER",
        "NOVEMBER",
        "DECEMBER",
    ],
    [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ],
    [
        "jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec",
    ],
    [
        "JAN", "FEB", "MAR", "APR", "MAY", "JUN", "JUL", "AUG", "SEP", "OCT", "NOV", "DEC",
    ],
];

pub const DAY_NAME_TABLE: [[&str; 7]; 6] = [
    [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ],
    [
        "sunday",
        "monday",
        "tuesday",
        "wednesday",
        "thursday",
        "friday",
        "saturday",
    ],
    [
        "SUNDAY",
        "MONDAY",
        "TUESDAY",
        "WEDNESDAY",
        "THURSDAY",
        "FRIDAY",
        "SATURDAY",
    ],
    ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
    ["sun", "mon", "tue", "wed", "thu", "fri", "sat"],
    ["SUN", "MON", "TUE", "WED", "THU", "FRI", "SAT"],
];

pub trait DateTimeFormat {
    const YEAR_MAX_LENGTH: usize = 4;

    const MONTH_MAX_LENGTH: usize = 2;

    const DAY_MAX_LENGTH: usize = 2;

    const HOUR_MAX_LENGTH: usize = 2;

    const MINUTE_MAX_LENGTH: usize = 2;

    const SECOND_MAX_LENGTH: usize = 2;
}

impl DateTimeFormat for Date {}

impl DateTimeFormat for Time {}

impl DateTimeFormat for Timestamp {}

impl DateTimeFormat for IntervalYM {
    const YEAR_MAX_LENGTH: usize = 9;
}

impl DateTimeFormat for IntervalDT {
    const DAY_MAX_LENGTH: usize = 9;
}

#[derive(Debug)]
pub struct NaiveDateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub sec: u32,
    pub usec: u32,

    // for Timestamp parsing
    pub ampm: Option<AmPm>,

    // for interval
    pub is_interval: bool,
    pub negate: bool,

    // for date and timestamp
    pub date: Option<Date>,
}

impl NaiveDateTime {
    #[inline]
    pub const fn new() -> Self {
        NaiveDateTime {
            year: DATE_MIN_YEAR,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            sec: 0,
            usec: 0,
            ampm: None,
            is_interval: false,
            negate: false,
            date: None,
        }
    }

    #[inline]
    pub const fn year(&self) -> i32 {
        self.year
    }

    #[inline]
    pub const fn month(&self) -> u32 {
        self.month
    }

    #[inline]
    pub const fn day(&self) -> u32 {
        self.day
    }

    #[inline]
    pub const fn hour24(&self) -> u32 {
        self.hour
    }

    #[inline]
    pub const fn hour12(&self) -> u32 {
        match self.hour {
            0 => 12,
            1..=12 => self.hour,
            _ => self.hour - 12,
        }
    }

    #[inline]
    pub const fn minute(&self) -> u32 {
        self.minute
    }

    #[inline]
    pub const fn sec(&self) -> u32 {
        self.sec
    }

    #[inline]
    pub const fn usec(&self) -> u32 {
        self.usec
    }

    #[inline]
    pub fn fraction(&self, p: u8) -> u32 {
        assert!(p < 10);
        (self.usec() as f64 / FRACTION_FACTOR[p as usize]) as u32
    }

    #[inline]
    pub const fn is_interval(&self) -> bool {
        self.is_interval
    }

    #[inline]
    pub const fn negate(&self) -> bool {
        self.negate
    }

    #[inline]
    pub fn adjust_hour12(&mut self) {
        if let Some(ampm) = &self.ampm {
            let hour24 = match ampm {
                AmPm::Am => {
                    if self.hour == 12 {
                        0
                    } else {
                        self.hour
                    }
                }
                AmPm::Pm => {
                    if self.hour == 12 {
                        12
                    } else {
                        self.hour + 12
                    }
                }
            };

            self.hour = hour24 as u32;
        }
    }

    #[inline]
    pub fn month_name(&self, style: NameStyle) -> &str {
        Month::from(self.month as usize).name(style)
    }

    #[inline]
    pub fn week_day_name(&self, style: NameStyle) -> Result<&str> {
        if let Some(d) = self.date {
            Ok(d.day_of_week().name(style))
        } else {
            Ok(Date::try_from_ymd(self.year, self.month, self.day)?
                .day_of_week()
                .name(style))
        }
    }
}

impl WeekDay {
    #[inline(always)]
    pub(crate) fn name(self, style: NameStyle) -> &'static str {
        DAY_NAME_TABLE[style as usize][self as usize - 1]
    }
}

impl Month {
    #[inline(always)]
    pub(crate) fn name(self, style: NameStyle) -> &'static str {
        MONTH_NAME_TABLE[style as usize][self as usize - 1]
    }
}

#[derive(Debug, PartialEq)]
pub enum Field {
    Invalid,
    /// ' '
    Blank,
    /// '-'
    Hyphen,
    /// ':'
    Colon,
    /// '/'
    Slash,
    /// '\\'
    Backslash,
    /// ','
    Comma,
    /// '.'
    Dot,
    /// ';'
    Semicolon,
    /// 'T'
    T,
    /// 'YYYY'
    Year(u8),
    /// 'MM'
    Month,
    /// 'DD'
    Day,
    /// 'DAY'
    DayName(NameStyle),
    /// 'MONTH'
    MonthName(NameStyle),
    /// 'HH24'.
    Hour24,
    /// 'HH', 'HH12'
    Hour12,
    /// 'MI'
    Minute,
    /// 'SS'
    Second,
    /// 'FF[1..9]'
    Fraction(Option<u8>),
    /// 'AM', 'A.M.', 'PM', 'P.M.'
    AmPm(AmPmStyle),
}

#[derive(Debug)]
pub enum AmPm {
    Am,
    Pm,
}

#[derive(Debug, PartialEq)]
pub enum AmPmStyle {
    Upper,
    Lower,
    UpperDot,
    LowerDot,
}

impl AmPmStyle {
    #[inline]
    const fn am(&self) -> &str {
        match self {
            AmPmStyle::Upper => "AM",
            AmPmStyle::Lower => "am",
            AmPmStyle::UpperDot => "A.M.",
            AmPmStyle::LowerDot => "a.m.",
        }
    }

    #[inline]
    const fn pm(&self) -> &str {
        match self {
            AmPmStyle::Upper => "PM",
            AmPmStyle::Lower => "pm",
            AmPmStyle::UpperDot => "P.M.",
            AmPmStyle::LowerDot => "p.m.",
        }
    }

    #[inline]
    const fn format(&self, hour: u32) -> &str {
        match hour {
            0..=11 => self.am(),
            _ => self.pm(),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum NameStyle {
    Capital = 0,
    Lower = 1,
    Upper = 2,
    AbbrCapital = 3,
    AbbrLower = 4,
    AbbrUpper = 5,
}

trait CaseInsensitive {
    fn starts_with(&self, needle: &Self) -> bool;
}

impl CaseInsensitive for [u8] {
    #[inline]
    fn starts_with(&self, needle: &Self) -> bool {
        let n = needle.len();
        self.len() >= n && needle.eq_ignore_ascii_case(&self[..n])
    }
}

pub struct FormatParser<'a> {
    input: &'a [u8],
    pos: usize,
}

impl<'a> FormatParser<'a> {
    #[inline]
    pub const fn new(input: &'a [u8]) -> Self {
        FormatParser { input, pos: 0 }
    }

    #[inline]
    fn pop(&mut self) -> Option<u8> {
        if self.pos < self.input.len() {
            let val = Some(self.input[self.pos]);
            self.pos += 1;
            val
        } else {
            None
        }
    }

    #[inline]
    fn peek(&self) -> Option<u8> {
        self.input.get(self.pos).copied()
    }

    #[inline]
    fn advance(&mut self, step: usize) {
        self.pos += step;
    }

    #[inline]
    fn back(&mut self, step: usize) {
        self.pos -= step;
    }

    #[inline]
    fn remain(&self) -> Option<&[u8]> {
        if self.pos < self.input.len() {
            Some(&self.input[self.pos..])
        } else {
            None
        }
    }

    #[inline]
    fn parse_year(&mut self) -> Field {
        let remain = match self.remain() {
            Some(rem) => rem,
            None => return Field::Invalid,
        };

        let len = remain
            .iter()
            .take(4)
            .take_while(|&y| y.eq_ignore_ascii_case(&b'y'))
            .count();

        if len > 0 {
            self.advance(len);
            Field::Year(len as u8)
        } else {
            Field::Invalid
        }
    }

    #[inline]
    fn parse_hour(&mut self) -> Field {
        let ch = match self.pop() {
            Some(ch) => ch,
            None => return Field::Invalid,
        };

        if ch != b'H' && ch != b'h' {
            return Field::Invalid;
        }

        match self.remain() {
            Some(rem) => {
                if rem.starts_with(b"24") {
                    self.advance(2);
                    Field::Hour24
                } else if rem.starts_with(b"12") {
                    self.advance(2);
                    Field::Hour12
                } else {
                    Field::Hour12
                }
            }
            None => Field::Hour12,
        }
    }

    #[inline]
    fn parse_second(&mut self) -> Field {
        match self.pop() {
            Some(b'S') | Some(b's') => Field::Second,
            _ => Field::Invalid,
        }
    }

    #[inline]
    fn parse_fraction(&mut self) -> Field {
        match self.pop() {
            Some(b'F') | Some(b'f') => match self.peek() {
                Some(ch) if ch.is_ascii_digit() => {
                    self.advance(1);
                    let p = ch - b'0';
                    if (1..=9).contains(&p) {
                        Field::Fraction(Some(p))
                    } else {
                        Field::Invalid
                    }
                }
                _ => Field::Fraction(None),
            },
            _ => Field::Invalid,
        }
    }

    #[inline]
    fn parse_am(&mut self) -> Field {
        let remain = match self.remain() {
            Some(rem) => rem,
            None => return Field::Invalid,
        };

        if remain.len() >= 4 {
            let rem = &remain[0..4];
            match rem {
                b"A.M." | b"A.m." | b"a.M." => {
                    self.advance(4);
                    return Field::AmPm(AmPmStyle::UpperDot);
                }
                b"a.m." => {
                    self.advance(4);
                    return Field::AmPm(AmPmStyle::LowerDot);
                }
                _ => {}
            };
        }

        if remain.len() >= 2 {
            let rem = &remain[0..2];
            return match rem {
                b"AM" | b"Am" | b"aM" => {
                    self.advance(2);
                    Field::AmPm(AmPmStyle::Upper)
                }
                b"am" => {
                    self.advance(2);
                    Field::AmPm(AmPmStyle::Lower)
                }
                _ => Field::Invalid,
            };
        }

        Field::Invalid
    }

    #[inline]
    fn parse_month_name(&mut self) -> Field {
        let remain = match self.remain() {
            Some(rem) => rem,
            None => return Field::Invalid,
        };

        if CaseInsensitive::starts_with(remain, b"month") {
            return match &remain[0..2] {
                b"MO" => {
                    self.advance(5);
                    Field::MonthName(NameStyle::Upper)
                }
                b"Mo" => {
                    self.advance(5);
                    Field::MonthName(NameStyle::Capital)
                }
                _ => {
                    self.advance(5);
                    Field::MonthName(NameStyle::Lower)
                }
            };
        }

        if CaseInsensitive::starts_with(remain, b"mon") {
            return match &remain[0..2] {
                b"MO" => {
                    self.advance(3);
                    Field::MonthName(NameStyle::AbbrUpper)
                }
                b"Mo" => {
                    self.advance(3);
                    Field::MonthName(NameStyle::AbbrCapital)
                }
                _ => {
                    self.advance(3);
                    Field::MonthName(NameStyle::AbbrLower)
                }
            };
        }
        Field::Invalid
    }

    #[inline]
    fn parse_day_name(&mut self) -> Field {
        let remain = match self.remain() {
            Some(rem) => rem,
            None => return Field::Invalid,
        };

        if CaseInsensitive::starts_with(remain, b"day") {
            return match &remain[0..2] {
                b"DA" => {
                    self.advance(3);
                    Field::DayName(NameStyle::Upper)
                }
                b"Da" => {
                    self.advance(3);
                    Field::DayName(NameStyle::Capital)
                }
                _ => {
                    self.advance(3);
                    Field::DayName(NameStyle::Lower)
                }
            };
        } else if remain.len() >= 2 {
            return match &remain[0..2] {
                b"DY" => {
                    self.advance(2);
                    Field::DayName(NameStyle::AbbrUpper)
                }
                b"Dy" => {
                    self.advance(2);
                    Field::DayName(NameStyle::AbbrCapital)
                }
                _ => {
                    self.advance(2);
                    Field::DayName(NameStyle::AbbrLower)
                }
            };
        }

        Field::Invalid
    }

    #[inline]
    fn parse_pm(&mut self) -> Field {
        let remain = match self.remain() {
            Some(rem) => rem,
            None => return Field::Invalid,
        };

        if remain.len() >= 4 {
            let rem = &remain[0..4];
            match rem {
                b"P.M." | b"P.m." | b"p.M." => {
                    self.advance(4);
                    return Field::AmPm(AmPmStyle::UpperDot);
                }
                b"p.m." => {
                    self.advance(4);
                    return Field::AmPm(AmPmStyle::LowerDot);
                }
                _ => {}
            };
        }

        if remain.len() >= 2 {
            let rem = &remain[0..2];
            return match rem {
                b"PM" | b"Pm" | b"pM" => {
                    self.advance(2);
                    Field::AmPm(AmPmStyle::Upper)
                }
                b"pm" => {
                    self.advance(2);
                    Field::AmPm(AmPmStyle::Lower)
                }
                _ => Field::Invalid,
            };
        }

        Field::Invalid
    }

    fn next(&mut self) -> Option<Field> {
        match self.pop() {
            Some(char) => {
                let field = match char {
                    b' ' => Field::Blank,
                    b'-' => Field::Hyphen,
                    b':' => Field::Colon,
                    b'/' => Field::Slash,
                    b'\\' => Field::Backslash,
                    b',' => Field::Comma,
                    b'.' => Field::Dot,
                    b';' => Field::Semicolon,
                    b'A' | b'a' => {
                        self.back(1);
                        self.parse_am()
                    }
                    b'D' | b'd' => match self.peek() {
                        Some(ch) => match ch {
                            b'D' | b'd' => {
                                self.advance(1);
                                Field::Day
                            }
                            b'a' | b'A' | b'Y' | b'y' => {
                                self.back(1);
                                self.parse_day_name()
                            }
                            _ => Field::Invalid,
                        },
                        None => Field::Invalid,
                    },
                    b'F' | b'f' => self.parse_fraction(),
                    b'H' | b'h' => self.parse_hour(),
                    b'M' | b'm' => match self.peek() {
                        Some(ch) => match ch {
                            b'I' | b'i' => {
                                self.advance(1);
                                Field::Minute
                            }
                            b'M' | b'm' => {
                                self.advance(1);
                                Field::Month
                            }
                            b'O' | b'o' => {
                                self.back(1);
                                self.parse_month_name()
                            }
                            _ => Field::Invalid,
                        },
                        None => Field::Invalid,
                    },
                    b'P' | b'p' => {
                        self.back(1);
                        self.parse_pm()
                    }
                    b'S' | b's' => self.parse_second(),
                    b'T' => Field::T,
                    b'Y' | b'y' => {
                        self.back(1);
                        self.parse_year()
                    }
                    _ => Field::Invalid,
                };
                Some(field)
            }
            None => None,
        }
    }
}

impl<'a> Iterator for FormatParser<'a> {
    type Item = Field;

    #[inline(always)]
    fn next(&mut self) -> Option<Field> {
        self.next()
    }
}

/// Date/Time formatter.
#[derive(Debug)]
pub struct Formatter {
    fields: StackVec<Field, MAX_FIELDS>,
}

impl Formatter {
    /// Creates a new `Formatter` from given format string.
    #[inline]
    pub fn try_new<S: AsRef<str>>(fmt: S) -> Result<Self> {
        let parser = FormatParser::new(fmt.as_ref().as_bytes());

        let mut fields = StackVec::new();

        for field in parser {
            if let Field::Invalid = field {
                return Err(Error::InvalidFormat(fmt.as_ref().to_string()));
            }

            if fields.is_full() {
                return Err(Error::InvalidFormat(format!(
                    "format `{}` is too long",
                    fmt.as_ref()
                )));
            }

            fields.push(field);
        }

        Ok(Formatter { fields })
    }

    /// Formats datetime types
    #[inline]
    pub fn format<W: fmt::Write, T: Into<NaiveDateTime>>(
        &self,
        datetime: T,
        mut w: W,
    ) -> Result<()> {
        let dt = datetime.into();
        if dt.negate() {
            // negate interval
            w.write_char('-')?;
        }

        for field in self.fields.iter() {
            match field {
                Field::Invalid => unreachable!(),
                Field::Blank => w.write_char(' ')?,
                Field::Hyphen => w.write_char('-')?,
                Field::Colon => w.write_char(':')?,
                Field::Slash => w.write_char('/')?,
                Field::Backslash => w.write_char('\\')?,
                Field::Comma => w.write_char(',')?,
                Field::Dot => w.write_char('.')?,
                Field::Semicolon => w.write_char(';')?,
                Field::T => w.write_char('T')?,
                Field::Year(n) => write!(w, "{:<0width$}", dt.year(), width = *n as usize)?,
                Field::Month => write!(w, "{:02}", dt.month())?,
                Field::Day => write!(w, "{:02}", dt.day())?,
                Field::Hour24 => write!(w, "{:02}", dt.hour24())?,
                Field::Hour12 => {
                    let hour = if dt.is_interval() {
                        dt.hour24()
                    } else {
                        dt.hour12()
                    };
                    write!(w, "{:02}", hour)?
                }
                Field::Minute => write!(w, "{:02}", dt.minute())?,
                Field::Second => write!(w, "{:02}", dt.sec())?,
                Field::Fraction(p) => {
                    let p = p.unwrap_or(6);
                    write!(w, "{:<0width$}", dt.fraction(p), width = p as usize)?
                }
                Field::AmPm(am_pm) => write!(w, "{}", am_pm.format(dt.hour24()))?,
                Field::MonthName(style) => write!(w, "{}", dt.month_name(*style))?,
                Field::DayName(style) => write!(w, "{}", dt.week_day_name(*style)?)?,
            }
        }

        Ok(())
    }

    /// Parses datetime types
    #[inline]
    pub fn parse<
        S: AsRef<str>,
        T: TryFrom<NaiveDateTime, Error = Error> + DateTime + DateTimeFormat,
    >(
        &self,
        input: S,
    ) -> Result<T> {
        let mut s = input.as_ref().as_bytes();

        let mut dt = NaiveDateTime::new();

        macro_rules! expect_char {
            ($ch: expr) => {{
                if expect_char(s, $ch) {
                    s = &s[1..];
                } else {
                    return Err(Error::ParseError(format!(
                        "the input is inconsistent with the format: {}",
                        input.as_ref()
                    )));
                }
            }};
        }

        macro_rules! expect_char_with_tolerence {
            ($ch: expr) => {{
                if expect_char(s, $ch) {
                    s = &s[1..];
                } else {
                    continue;
                }
            }};
        }

        macro_rules! expect_number {
            ($max_len: expr) => {{
                let (neg, n, rem) = parse_number(s, $max_len)?;
                s = rem;
                (n, neg)
            }};
        }

        macro_rules! expect_number_with_tolerance {
            ($max_len: expr, $default: expr) => {{
                if s.is_empty() {
                    ($default, $default < 0)
                } else {
                    let (neg, n, rem) = parse_number(s, $max_len)?;
                    s = rem;
                    (n, neg)
                }
            }};
        }

        let mut is_year_set = false;
        let mut is_month_set = false;
        let mut is_day_set = false;
        let mut is_hour_set = false;
        let mut is_min_set = false;
        let mut is_sec_set = false;
        let mut is_fraction_set = false;

        let mut dow: Option<WeekDay> = None;

        for field in self.fields.iter() {
            match field {
                // todo ignore the absence of symbols; Format exact
                Field::Invalid => unreachable!(),
                Field::Blank => expect_char_with_tolerence!(b' '),
                Field::Hyphen => expect_char!(b'-'),
                Field::Colon => expect_char_with_tolerence!(b':'),
                Field::Slash => expect_char!(b'/'),
                Field::Backslash => expect_char!(b'\\'),
                Field::Comma => expect_char!(b','),
                Field::Dot => expect_char_with_tolerence!(b'.'),
                Field::Semicolon => expect_char!(b';'),
                Field::T => expect_char!(b'T'),
                Field::Year(n) => {
                    if is_year_set {
                        return Err(Error::ParseError(
                            "format code (year) appears twice".to_string(),
                        ));
                    }
                    let len = if T::YEAR_MAX_LENGTH > 4 {
                        T::YEAR_MAX_LENGTH
                    } else {
                        *n as usize
                    };
                    let (year, negate) = expect_number!(len);
                    dt.year = year;
                    dt.negate = negate;
                    is_year_set = true;
                }
                Field::Month => {
                    if is_month_set {
                        return Err(Error::ParseError(
                            "format code (month) appears twice".to_string(),
                        ));
                    }
                    let (month, _) = expect_number!(T::MONTH_MAX_LENGTH);
                    dt.month = month as u32;
                    is_month_set = true;
                }
                Field::Day => {
                    if is_day_set {
                        return Err(Error::ParseError(
                            "format code (day) appears twice".to_string(),
                        ));
                    }
                    let (day, negate) = expect_number!(T::DAY_MAX_LENGTH);
                    dt.day = day.abs() as u32;
                    dt.negate = negate;
                    is_day_set = true;
                }
                Field::Hour24 => {
                    // todo hh24 excludes hh12 and amp
                    if is_hour_set {
                        return Err(Error::ParseError(
                            "format code (hour) appears twice".to_string(),
                        ));
                    }

                    let (hour, _) = expect_number_with_tolerance!(T::HOUR_MAX_LENGTH, 0);
                    dt.hour = hour as u32;
                    is_hour_set = true;
                }
                Field::Hour12 => {
                    if is_hour_set {
                        return Err(Error::ParseError(
                            "format code (hour) appears twice".to_string(),
                        ));
                    }
                    let (hour, _) = expect_number_with_tolerance!(T::HOUR_MAX_LENGTH, 0);
                    dt.hour = hour as u32;
                    dt.adjust_hour12();
                    is_hour_set = true;
                }
                Field::Minute => {
                    if is_min_set {
                        return Err(Error::ParseError(
                            "format code (minute) appears twice".to_string(),
                        ));
                    }
                    let (minute, _) = expect_number_with_tolerance!(T::MINUTE_MAX_LENGTH, 0);
                    dt.minute = minute as u32;
                    is_min_set = true;
                }
                Field::Second => {
                    if is_sec_set {
                        return Err(Error::ParseError(
                            "format code (second) appears twice".to_string(),
                        ));
                    }
                    let (sec, _) = expect_number_with_tolerance!(T::SECOND_MAX_LENGTH, 0);
                    dt.sec = sec as u32;
                    is_sec_set = true;
                }
                Field::Fraction(p) => {
                    if is_fraction_set {
                        return Err(Error::ParseError(
                            "format code (fraction) appears twice".to_string(),
                        ));
                    }
                    // When parsing, if FF is given, the default precision is 9
                    let (usec, rem) = parse_fraction(s, p.unwrap_or(9) as usize)?;
                    s = rem;
                    dt.usec = usec;
                    is_fraction_set = true;
                }
                Field::AmPm(_) => {
                    if dt.ampm.is_some() {
                        return Err(Error::ParseError(
                            "format code (am/pm) appears twice".to_string(),
                        ));
                    }
                    let (am_pm, rem) = parse_ampm(s)?;
                    s = rem;

                    dt.ampm = Some(am_pm);
                    dt.adjust_hour12();
                }
                Field::MonthName(_) => {
                    if is_month_set {
                        return Err(Error::ParseError(
                            "format code (month) appears twice".to_string(),
                        ));
                    }
                    let (month, rem) = parse_month_name(s)?;
                    s = rem;

                    dt.month = month as u32;
                    is_month_set = true;
                }
                Field::DayName(_) => {
                    if dow.is_some() {
                        return Err(Error::ParseError(
                            "format code (day of week) appears twice".to_string(),
                        ));
                    }
                    let (d, rem) = parse_week_day_name(s)?;
                    s = rem;

                    dow = Some(d);
                }
            }
        }

        if !s.is_empty() {
            return Err(Error::ParseError(
                "format picture ends before converting entire input string".to_string(),
            ));
        }

        // Check if parsed day of week conflicts with the date
        if let Some(d) = dow {
            let date = Date::try_from(&dt)?;
            if date.day_of_week() != d {
                return Err(Error::ParseError(
                    "day of week conflicts with Julian date".to_string(),
                ));
            }
        }

        T::try_from(dt)
    }
}

#[inline]
fn expect_char(s: &[u8], expected: u8) -> bool {
    matches!(s.first(), Some(ch) if *ch == expected)
}

#[inline]
fn parse_number(input: &[u8], max_len: usize) -> Result<(bool, i32, &[u8])> {
    let (negative, s) = match input.first() {
        Some(ch) => match ch {
            b'+' => (false, &input[1..]),
            b'-' => (true, &input[1..]),
            _ => (false, input),
        },
        None => return Err(Error::ParseError("invalid number".to_string())),
    };

    let (digits, s) = eat_digits(s, max_len);
    if digits.is_empty() || digits.len() > 9 {
        return Err(Error::ParseError("invalid number".to_string()));
    }

    let int = digits
        .iter()
        .fold(0, |int, &i| int * 10 + (i - b'0') as i32);

    let int = if negative { -int } else { int };

    Ok((negative, int, s))
}

#[inline]
fn eat_digits(s: &[u8], max_len: usize) -> (&[u8], &[u8]) {
    let i = s
        .iter()
        .take(max_len)
        .take_while(|&i| i.is_ascii_digit())
        .count();
    (&s[..i], &s[i..])
}

#[inline]
fn parse_ampm(s: &[u8]) -> Result<(AmPm, &[u8])> {
    if CaseInsensitive::starts_with(s, b"AM") {
        Ok((AmPm::Am, &s[2..]))
    } else if CaseInsensitive::starts_with(s, b"PM") {
        Ok((AmPm::Pm, &s[2..]))
    } else if CaseInsensitive::starts_with(s, b"A.M.") {
        Ok((AmPm::Am, &s[4..]))
    } else if CaseInsensitive::starts_with(s, b"P.M.") {
        Ok((AmPm::Pm, &s[4..]))
    } else {
        Err(Error::ParseError("AM/PM is missing".to_string()))
    }
}

#[inline]
fn parse_fraction(s: &[u8], max_len: usize) -> Result<(u32, &[u8])> {
    if s.is_empty() {
        return Ok((0, s));
    }
    let (digits, s) = eat_digits(s, max_len);
    let int = digits
        .iter()
        .fold(0, |int, &i| int * 10 + (i - b'0') as i32);
    Ok((
        (int as f64 * FRACTION_FACTOR[digits.len()]).round() as u32,
        s,
    ))
}

#[inline]
fn parse_month_name(s: &[u8]) -> Result<(Month, &[u8])> {
    for (index, mon) in MONTH_NAME_TABLE[Capital as usize].iter().enumerate() {
        if CaseInsensitive::starts_with(s, mon.as_bytes()) {
            return Ok((Month::from(index + 1), &s[mon.len()..]));
        }
    }

    for (index, mon) in MONTH_NAME_TABLE[AbbrCapital as usize].iter().enumerate() {
        if CaseInsensitive::starts_with(s, mon.as_bytes()) {
            return Ok((Month::from(index + 1), &s[mon.len()..]));
        }
    }

    Err(Error::ParseError("month is missing".to_string()))
}

#[inline]
fn parse_week_day_name(s: &[u8]) -> Result<(WeekDay, &[u8])> {
    for (index, day) in DAY_NAME_TABLE[Capital as usize].iter().enumerate() {
        if CaseInsensitive::starts_with(s, day.as_bytes()) {
            return Ok((WeekDay::from(index + 1), &s[day.len()..]));
        }
    }

    for (index, day) in DAY_NAME_TABLE[AbbrCapital as usize].iter().enumerate() {
        if CaseInsensitive::starts_with(s, day.as_bytes()) {
            return Ok((WeekDay::from(index + 1), &s[day.len()..]));
        }
    }

    Err(Error::ParseError("week day is missing".to_string()))
}

pub struct LazyFormat<T: Into<NaiveDateTime>> {
    fmt: Formatter,
    dt: T,
}

impl<T: Into<NaiveDateTime>> LazyFormat<T> {
    #[inline]
    pub fn new(fmt: Formatter, dt: T) -> Self {
        LazyFormat { fmt, dt }
    }
}

impl<T: Into<NaiveDateTime> + Copy> fmt::Display for LazyFormat<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt.format(self.dt, f).map_err(|_| fmt::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::AmPmStyle::{Lower as AmLower, LowerDot, Upper as AmUpper, UpperDot};
    use crate::format::Field::{AmPm, Blank, DayName, MonthName};
    use crate::format::NameStyle::{AbbrCapital, AbbrLower, AbbrUpper, Capital, Lower, Upper};

    #[test]
    fn test_format_parser() {
        let mut parser = FormatParser::new(b"yyyyyy-mm-dd hh24:mi:ss.ff9");
        assert_eq!(parser.next(), Some(Field::Year(4)));
        assert_eq!(parser.next(), Some(Field::Year(2)));
        assert_eq!(parser.next(), Some(Field::Hyphen));
        assert_eq!(parser.next(), Some(Field::Month));
        assert_eq!(parser.next(), Some(Field::Hyphen));
        assert_eq!(parser.next(), Some(Field::Day));
        assert_eq!(parser.next(), Some(Field::Blank));
        assert_eq!(parser.next(), Some(Field::Hour24));
        assert_eq!(parser.next(), Some(Field::Colon));
        assert_eq!(parser.next(), Some(Field::Minute));
        assert_eq!(parser.next(), Some(Field::Colon));
        assert_eq!(parser.next(), Some(Field::Second));
        assert_eq!(parser.next(), Some(Field::Dot));
        assert_eq!(parser.next(), Some(Field::Fraction(Some(9))));
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_format_parser_param() {
        let mut parser = FormatParser::new(
            b"MONTH Month month MON mon Mon DAY day Day DY Dy dy AM am A.M. a.m.",
        );

        let expect = [
            MonthName(Upper),
            Blank,
            MonthName(Capital),
            Blank,
            MonthName(Lower),
            Blank,
            MonthName(AbbrUpper),
            Blank,
            MonthName(AbbrLower),
            Blank,
            MonthName(AbbrCapital),
            Blank,
            DayName(Upper),
            Blank,
            DayName(Lower),
            Blank,
            DayName(Capital),
            Blank,
            DayName(AbbrUpper),
            Blank,
            DayName(AbbrCapital),
            Blank,
            DayName(AbbrLower),
            Blank,
            AmPm(AmUpper),
            Blank,
            AmPm(AmLower),
            Blank,
            AmPm(UpperDot),
            Blank,
            AmPm(LowerDot),
        ];
        for e in expect.iter() {
            assert_eq!(e, &parser.next().unwrap())
        }
        assert_eq!(None, parser.next())
    }
}
