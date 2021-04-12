//! Formatting (and parsing) utilities for date and time.

use crate::common::DATE_MIN_YEAR;
use crate::error::Result;
use crate::{Date, Error, IntervalDT, IntervalYM, Time, Timestamp};
use stack_buf::StackVec;
use std::fmt;

const MAX_FIELDS: usize = 32;

#[derive(Debug)]
pub struct NaiveDateTime {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub sec: u32,
    pub usec: u32,

    // for interval
    pub is_interval: bool,
    pub negate: bool,
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
            is_interval: false,
            negate: false,
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
    pub fn fraction(&self, frac: u8) -> u32 {
        const FACTOR: [f64; 10] = [
            1000000.0, 100000.0, 10000.0, 1000.0, 100.0, 10.0, 1.0, 0.1, 0.01, 0.001,
        ];

        assert!(frac < 10);

        (self.usec() as f64 / FACTOR[frac as usize]) as u32
    }

    #[inline]
    pub const fn is_interval(&self) -> bool {
        self.is_interval
    }

    #[inline]
    pub const fn negate(&self) -> bool {
        self.negate
    }
}

#[derive(Debug, PartialEq, Clone)]
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
    Year,
    /// 'MM'
    Month,
    /// 'DD'
    Day,
    /// 'HH24'.
    Hour24,
    /// 'HH', 'HH12'
    Hour12,
    /// 'MI'
    Minute,
    /// 'SS'
    Second,
    /// 'FF[1..9]'
    Fraction(u8),
    /// 'AM', 'A.M.', 'PM', 'P.M.'
    AmPm(AmPm),
}

#[derive(Debug, PartialEq, Clone)]
pub enum AmPm {
    Upper,
    Lower,
    UpperDot,
    LowerDot,
}

impl AmPm {
    #[inline]
    const fn am(&self) -> &str {
        match self {
            AmPm::Upper => "AM",
            AmPm::Lower => "am",
            AmPm::UpperDot => "A.M.",
            AmPm::LowerDot => "a.m.",
        }
    }

    #[inline]
    const fn pm(&self) -> &str {
        match self {
            AmPm::Upper => "PM",
            AmPm::Lower => "pm",
            AmPm::UpperDot => "P.M.",
            AmPm::LowerDot => "p.m.",
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

        if CaseInsensitive::starts_with(remain, b"YYY") {
            self.advance(3);
            Field::Year
        } else {
            Field::Invalid
        }
    }

    #[inline]
    fn parse_day(&mut self) -> Field {
        match self.pop() {
            Some(b'D') | Some(b'd') => Field::Day,
            _ => Field::Invalid,
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
                    let frac = ch - b'0';
                    if (1..=9).contains(&frac) {
                        Field::Fraction(frac)
                    } else {
                        Field::Invalid
                    }
                }
                _ => Field::Fraction(6),
            },
            _ => Field::Invalid,
        }
    }

    #[inline]
    fn parse_am(&mut self) -> Field {
        self.back(1);

        let remain = match self.remain() {
            Some(rem) => rem,
            None => return Field::Invalid,
        };

        if remain.len() >= 4 {
            let rem = &remain[0..4];
            return match rem {
                b"A.M." | b"A.m." | b"a.M." => {
                    self.advance(4);
                    Field::AmPm(AmPm::UpperDot)
                }
                b"a.m." => {
                    self.advance(4);
                    Field::AmPm(AmPm::LowerDot)
                }
                _ => Field::Invalid,
            };
        }

        if remain.len() >= 2 {
            let rem = &remain[0..2];
            return match rem {
                b"AM" | b"Am" | b"aM" => {
                    self.advance(2);
                    Field::AmPm(AmPm::Upper)
                }
                b"am" => {
                    self.advance(2);
                    Field::AmPm(AmPm::Lower)
                }
                _ => Field::Invalid,
            };
        }

        Field::Invalid
    }

    #[inline]
    fn parse_pm(&mut self) -> Field {
        self.back(1);

        let remain = match self.remain() {
            Some(rem) => rem,
            None => return Field::Invalid,
        };

        if remain.len() >= 4 {
            let rem = &remain[0..4];
            return match rem {
                b"P.M." | b"P.m." | b"p.M." => {
                    self.advance(4);
                    Field::AmPm(AmPm::UpperDot)
                }
                b"p.m." => {
                    self.advance(4);
                    Field::AmPm(AmPm::LowerDot)
                }
                _ => Field::Invalid,
            };
        }

        if remain.len() >= 2 {
            let rem = &remain[0..2];
            return match rem {
                b"PM" | b"Pm" | b"pM" => {
                    self.advance(2);
                    Field::AmPm(AmPm::Upper)
                }
                b"pm" => {
                    self.advance(2);
                    Field::AmPm(AmPm::Lower)
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
                    b'A' | b'a' => self.parse_am(),
                    b'D' | b'd' => self.parse_day(),
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
                            _ => Field::Invalid,
                        },
                        None => Field::Invalid,
                    },
                    b'P' | b'p' => self.parse_pm(),
                    b'S' | b's' => self.parse_second(),
                    b'T' => Field::T,
                    b'Y' | b'y' => self.parse_year(),
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

pub struct Formatter {
    fields: StackVec<Field, MAX_FIELDS>,
}

impl Formatter {
    #[inline]
    pub fn parse<S: AsRef<str>>(fmt: S) -> Result<Self> {
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

    #[inline]
    pub fn format_date<W: fmt::Write>(&self, date: Date, w: W) -> Result<()> {
        let dt = date.into();
        self.internal_format(&dt, w)
    }

    #[inline]
    pub fn format_time<W: fmt::Write>(&self, time: Time, w: W) -> Result<()> {
        let dt = time.into();
        self.internal_format(&dt, w)
    }

    #[inline]
    pub fn format_timestamp<W: fmt::Write>(&self, ts: Timestamp, w: W) -> Result<()> {
        let dt = ts.into();
        self.internal_format(&dt, w)
    }

    #[inline]
    pub fn format_interval_ym<W: fmt::Write>(&self, interval: IntervalYM, w: W) -> Result<()> {
        let dt = interval.into();
        self.internal_format(&dt, w)
    }

    #[inline]
    pub fn format_interval_dt<W: fmt::Write>(&self, interval: IntervalDT, w: W) -> Result<()> {
        let dt = interval.into();
        self.internal_format(&dt, w)
    }

    #[inline]
    fn internal_format<W: fmt::Write>(&self, dt: &NaiveDateTime, mut w: W) -> Result<()> {
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
                Field::Year => write!(w, "{:04}", dt.year())?,
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
                Field::Fraction(frac) => {
                    write!(w, "{:<0width$}", dt.fraction(*frac), width = *frac as usize)?
                }
                Field::AmPm(am_pm) => write!(w, "{}", am_pm.format(dt.hour24()))?,
            }
        }

        Ok(())
    }
}

pub struct LazyFormat {
    fmt: Formatter,
    dt: NaiveDateTime,
}

impl LazyFormat {
    #[inline]
    pub const fn new(fmt: Formatter, dt: NaiveDateTime) -> Self {
        LazyFormat { fmt, dt }
    }
}

impl fmt::Display for LazyFormat {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt
            .internal_format(&self.dt, f)
            .map_err(|_| fmt::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_parser() {
        let mut parser = FormatParser::new(b"yyyy-mm-dd hh24:mi:ss.ff9");
        assert_eq!(parser.next(), Some(Field::Year));
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
        assert_eq!(parser.next(), Some(Field::Fraction(9)));
        assert_eq!(parser.next(), None);
    }
}
