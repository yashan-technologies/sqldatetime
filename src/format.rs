//! Formatting (and parsing) utilities for date and time.

use crate::common::{is_leap_year, the_day_of_year, the_month_day_of_days, DATE_MIN_YEAR};
use crate::date::{Month, WeekDay};
use crate::error::Result;
use crate::format::NameStyle::{AbbrCapital, Capital};
use crate::local::Local;
use crate::util::StrExt;
use crate::{Date, DateTime, Error, IntervalDT, IntervalYM, Time, Timestamp};
use stack_buf::StackVec;
use std::convert::TryFrom;
use std::fmt;

const MAX_FIELDS: usize = 36;

const FRACTION_FACTOR: [f64; 10] = [
    1000000.0, 100000.0, 10000.0, 1000.0, 100.0, 10.0, 1.0, 0.1, 0.01, 0.001,
];

const YEAR_MODIFIER: [u32; 4] = [10, 100, 1000, 10000];

const MONTH_TABLE: [&str; 13] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12",
];

const HOUR_TABLE: [&str; 25] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23", "24",
];

const DAY_TABLE: [&str; 32] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
];

const MINUTE_SECOND_TABLE: [&str; 61] = [
    "00", "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12", "13", "14", "15",
    "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30", "31",
    "32", "33", "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45", "46", "47",
    "48", "49", "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "60",
];

const MONTH_NAME_TABLE: [[&str; 12]; 6] = [
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

const DAY_NAME_TABLE: [[&str; 7]; 6] = [
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

const DAY_OF_WEEK_TABLE: [&str; 8] = ["0", "1", "2", "3", "4", "5", "6", "7"];

#[rustfmt::skip]
const DAY_OF_YEAR_TABLE: [&str; 367] = [
    "000",
    "001", "002", "003", "004", "005", "006", "007", "008", "009", "010",
    "011", "012", "013", "014", "015", "016", "017", "018", "019", "020",
    "021", "022", "023", "024", "025", "026", "027", "028", "029", "030",
    "031", "032", "033", "034", "035", "036", "037", "038", "039", "040",
    "041", "042", "043", "044", "045", "046", "047", "048", "049", "050",
    "051", "052", "053", "054", "055", "056", "057", "058", "059", "060",
    "061", "062", "063", "064", "065", "066", "067", "068", "069", "070",
    "071", "072", "073", "074", "075", "076", "077", "078", "079", "080",
    "081", "082", "083", "084", "085", "086", "087", "088", "089", "090",
    "091", "092", "093", "094", "095", "096", "097", "098", "099", "100",
    "101", "102", "103", "104", "105", "106", "107", "108", "109", "110",
    "111", "112", "113", "114", "115", "116", "117", "118", "119", "120",
    "121", "122", "123", "124", "125", "126", "127", "128", "129", "130",
    "131", "132", "133", "134", "135", "136", "137", "138", "139", "140",
    "141", "142", "143", "144", "145", "146", "147", "148", "149", "150",
    "151", "152", "153", "154", "155", "156", "157", "158", "159", "160",
    "161", "162", "163", "164", "165", "166", "167", "168", "169", "170",
    "171", "172", "173", "174", "175", "176", "177", "178", "179", "180",
    "181", "182", "183", "184", "185", "186", "187", "188", "189", "190",
    "191", "192", "193", "194", "195", "196", "197", "198", "199", "200",
    "201", "202", "203", "204", "205", "206", "207", "208", "209", "210",
    "211", "212", "213", "214", "215", "216", "217", "218", "219", "220",
    "221", "222", "223", "224", "225", "226", "227", "228", "229", "230",
    "231", "232", "233", "234", "235", "236", "237", "238", "239", "240",
    "241", "242", "243", "244", "245", "246", "247", "248", "249", "250",
    "251", "252", "253", "254", "255", "256", "257", "258", "259", "260",
    "261", "262", "263", "264", "265", "266", "267", "268", "269", "270",
    "271", "272", "273", "274", "275", "276", "277", "278", "279", "280",
    "281", "282", "283", "284", "285", "286", "287", "288", "289", "290",
    "291", "292", "293", "294", "295", "296", "297", "298", "299", "300",
    "301", "302", "303", "304", "305", "306", "307", "308", "309", "310",
    "311", "312", "313", "314", "315", "316", "317", "318", "319", "320",
    "321", "322", "323", "324", "325", "326", "327", "328", "329", "330",
    "331", "332", "333", "334", "335", "336", "337", "338", "339", "340",
    "341", "342", "343", "344", "345", "346", "347", "348", "349", "350",
    "351", "352", "353", "354", "355", "356", "357", "358", "359", "360",
    "361", "362", "363", "364", "365", "366",
];

#[rustfmt::skip]
const WEEK_OF_MONTH_TABLE: [&str; 32] = [
    "0",
    "1", "1", "1", "1", "1", "1", "1",
    "2", "2", "2", "2", "2", "2", "2",
    "3", "3", "3", "3", "3", "3", "3",
    "4", "4", "4", "4", "4", "4", "4",
    "5", "5", "5",
];

#[rustfmt::skip]
const WEEK_OF_YEAR_TABLE: [&str; 367] = [
    "00",
    "01", "01", "01", "01", "01", "01", "01",
    "02", "02", "02", "02", "02", "02", "02",
    "03", "03", "03", "03", "03", "03", "03",
    "04", "04", "04", "04", "04", "04", "04",
    "05", "05", "05", "05", "05", "05", "05",
    "06", "06", "06", "06", "06", "06", "06",
    "07", "07", "07", "07", "07", "07", "07",
    "08", "08", "08", "08", "08", "08", "08",
    "09", "09", "09", "09", "09", "09", "09",
    "10", "10", "10", "10", "10", "10", "10",
    "11", "11", "11", "11", "11", "11", "11",
    "12", "12", "12", "12", "12", "12", "12",
    "13", "13", "13", "13", "13", "13", "13",
    "14", "14", "14", "14", "14", "14", "14",
    "15", "15", "15", "15", "15", "15", "15",
    "16", "16", "16", "16", "16", "16", "16",
    "17", "17", "17", "17", "17", "17", "17",
    "18", "18", "18", "18", "18", "18", "18",
    "19", "19", "19", "19", "19", "19", "19",
    "20", "20", "20", "20", "20", "20", "20",
    "21", "21", "21", "21", "21", "21", "21",
    "22", "22", "22", "22", "22", "22", "22",
    "23", "23", "23", "23", "23", "23", "23",
    "24", "24", "24", "24", "24", "24", "24",
    "25", "25", "25", "25", "25", "25", "25",
    "26", "26", "26", "26", "26", "26", "26",
    "27", "27", "27", "27", "27", "27", "27",
    "28", "28", "28", "28", "28", "28", "28",
    "29", "29", "29", "29", "29", "29", "29",
    "30", "30", "30", "30", "30", "30", "30",
    "31", "31", "31", "31", "31", "31", "31",
    "32", "32", "32", "32", "32", "32", "32",
    "33", "33", "33", "33", "33", "33", "33",
    "34", "34", "34", "34", "34", "34", "34",
    "35", "35", "35", "35", "35", "35", "35",
    "36", "36", "36", "36", "36", "36", "36",
    "37", "37", "37", "37", "37", "37", "37",
    "38", "38", "38", "38", "38", "38", "38",
    "39", "39", "39", "39", "39", "39", "39",
    "40", "40", "40", "40", "40", "40", "40",
    "41", "41", "41", "41", "41", "41", "41",
    "42", "42", "42", "42", "42", "42", "42",
    "43", "43", "43", "43", "43", "43", "43",
    "44", "44", "44", "44", "44", "44", "44",
    "45", "45", "45", "45", "45", "45", "45",
    "46", "46", "46", "46", "46", "46", "46",
    "47", "47", "47", "47", "47", "47", "47",
    "48", "48", "48", "48", "48", "48", "48",
    "49", "49", "49", "49", "49", "49", "49",
    "50", "50", "50", "50", "50", "50", "50",
    "51", "51", "51", "51", "51", "51", "51",
    "52", "52", "52", "52", "52", "52", "52",
    "53", "53",
];

pub trait DateTimeFormat:
    DateTime + Into<NaiveDateTime> + Copy + TryFrom<NaiveDateTime, Error = Error>
{
    const YEAR_MAX_LENGTH: usize = 4;

    const MONTH_MAX_LENGTH: usize = 2;

    const DAY_MAX_LENGTH: usize = 2;

    const HOUR_MAX_LENGTH: usize = 2;

    const MINUTE_MAX_LENGTH: usize = 2;

    const SECOND_MAX_LENGTH: usize = 2;

    const DAY_OF_YEAR_MAX_LENGTH: usize = 3;

    const HAS_DATE: bool;
    const HAS_TIME: bool;
    const HAS_FRACTION: bool;
    const IS_INTERVAL_YM: bool;
    const IS_INTERVAL_DT: bool;
}

impl DateTimeFormat for Date {
    const HAS_DATE: bool = true;
    const HAS_TIME: bool = false;
    const HAS_FRACTION: bool = false;
    const IS_INTERVAL_YM: bool = false;
    const IS_INTERVAL_DT: bool = false;
}

impl DateTimeFormat for Time {
    const HAS_DATE: bool = false;
    const HAS_TIME: bool = true;
    const HAS_FRACTION: bool = true;
    const IS_INTERVAL_YM: bool = false;
    const IS_INTERVAL_DT: bool = false;
}

impl DateTimeFormat for Timestamp {
    const HAS_DATE: bool = true;
    const HAS_TIME: bool = true;
    const HAS_FRACTION: bool = true;
    const IS_INTERVAL_YM: bool = false;
    const IS_INTERVAL_DT: bool = false;
}

impl DateTimeFormat for IntervalYM {
    const YEAR_MAX_LENGTH: usize = 9;

    const HAS_DATE: bool = false;
    const HAS_TIME: bool = false;
    const HAS_FRACTION: bool = false;
    const IS_INTERVAL_YM: bool = true;
    const IS_INTERVAL_DT: bool = false;
}

impl DateTimeFormat for IntervalDT {
    const DAY_MAX_LENGTH: usize = 9;

    const HAS_DATE: bool = false;
    const HAS_TIME: bool = true;
    const HAS_FRACTION: bool = true;
    const IS_INTERVAL_YM: bool = false;
    const IS_INTERVAL_DT: bool = true;
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
    pub negative: bool,
}

impl NaiveDateTime {
    #[inline]
    pub const fn new() -> Self {
        NaiveDateTime {
            year: DATE_MIN_YEAR,
            month: 0,
            day: 1,
            hour: 0,
            minute: 0,
            sec: 0,
            usec: 0,
            ampm: None,
            negative: false,
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
        debug_assert!(p < 10);
        (self.usec() as f64 / FRACTION_FACTOR[p as usize]) as u32
    }

    #[inline]
    pub const fn negative(&self) -> bool {
        self.negative
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

            self.hour = hour24;
        }
    }

    #[inline]
    pub const fn month_str(&self) -> &str {
        MONTH_TABLE[self.month as usize]
    }

    #[inline]
    pub const fn hour12_str(&self) -> &str {
        HOUR_TABLE[self.hour12() as usize]
    }

    #[inline]
    pub const fn hour24_str(&self) -> &str {
        HOUR_TABLE[self.hour24() as usize]
    }

    #[inline]
    pub const fn day_str(&self) -> &str {
        DAY_TABLE[self.day as usize]
    }

    #[inline]
    pub const fn minute_str(&self) -> &str {
        MINUTE_SECOND_TABLE[self.minute as usize]
    }

    #[inline]
    pub const fn second_str(&self) -> &str {
        MINUTE_SECOND_TABLE[self.sec as usize]
    }

    #[inline]
    pub fn month_name(&self, style: NameStyle) -> &str {
        Month::from(self.month as usize).name(style)
    }

    #[inline]
    pub fn week_day_name(&self, date: Option<Date>, style: NameStyle) -> Result<&str> {
        if let Some(d) = date {
            Ok(d.day_of_week().name(style))
        } else {
            Ok(Date::try_from_ymd(self.year, self.month, self.day)?
                .day_of_week()
                .name(style))
        }
    }

    #[inline]
    pub fn day_of_week_str(&self, date: Option<Date>) -> Result<&str> {
        if let Some(d) = date {
            Ok(d.day_of_week().num_str())
        } else {
            Ok(Date::try_from_ymd(self.year, self.month, self.day)?
                .day_of_week()
                .num_str())
        }
    }

    #[inline]
    pub fn day_of_year_str(&self) -> &str {
        let days = the_day_of_year(self.year, self.month, self.day);
        DAY_OF_YEAR_TABLE[days as usize]
    }

    #[inline]
    pub const fn week_of_month_str(&self) -> &str {
        WEEK_OF_MONTH_TABLE[self.day as usize]
    }

    #[inline]
    pub const fn week_of_year_str(&self) -> &str {
        WEEK_OF_YEAR_TABLE[the_day_of_year(self.year, self.month, self.day) as usize]
    }
}

impl WeekDay {
    #[inline(always)]
    pub(crate) fn name(self, style: NameStyle) -> &'static str {
        DAY_NAME_TABLE[style as usize][self as usize - 1]
    }

    #[inline(always)]
    pub(crate) fn num_str(self) -> &'static str {
        DAY_OF_WEEK_TABLE[self as usize]
    }
}

impl Month {
    #[inline(always)]
    pub(crate) fn name(self, style: NameStyle) -> &'static str {
        MONTH_NAME_TABLE[style as usize][self as usize - 1]
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Field {
    Invalid,
    /// ' '
    Blank(u8),
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
    /// '_'
    Underline,
    /// 'T', ISO 8601 format
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
    /// 'D'
    DayOfWeek,
    /// 'DDD'
    DayOfYear,
    /// 'W'
    WeekOfMonth,
    /// 'WW'
    WeekOfYear,
}

#[derive(Debug)]
pub enum AmPm {
    Am,
    Pm,
}

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Copy, Clone)]
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
    fn punctuation_count(&mut self, expect: u8) -> u8 {
        self.remain().map_or(0, |rem| {
            rem.iter().take_while(|&y| y.eq(&expect)).count() as u8
        })
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
                    b' ' => {
                        let len = self.punctuation_count(b' ');
                        self.advance(len as usize);
                        Field::Blank(len + 1)
                    }
                    b'-' => Field::Hyphen,
                    b':' => Field::Colon,
                    b'/' => Field::Slash,
                    b'\\' => Field::Backslash,
                    b',' => Field::Comma,
                    b'.' => Field::Dot,
                    b';' => Field::Semicolon,
                    b'_' => Field::Underline,
                    b'A' | b'a' => {
                        self.back(1);
                        self.parse_am()
                    }
                    b'D' | b'd' => match self.peek() {
                        Some(ch) => match ch {
                            b'D' | b'd' => {
                                self.advance(1);
                                match self.peek() {
                                    Some(ch) => match ch {
                                        b'D' | b'd' => {
                                            self.advance(1);
                                            Field::DayOfYear
                                        }
                                        _ => Field::Day,
                                    },
                                    None => Field::Day,
                                }
                            }
                            b'a' | b'A' | b'Y' | b'y' => {
                                self.back(1);
                                self.parse_day_name()
                            }
                            _ => Field::DayOfWeek,
                        },
                        None => Field::DayOfWeek,
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
                    b'W' | b'w' => match self.peek() {
                        Some(ch) => match ch {
                            b'W' | b'w' => {
                                self.advance(1);
                                Field::WeekOfYear
                            }
                            _ => Field::WeekOfMonth,
                        },
                        None => Field::WeekOfMonth,
                    },
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
    format_exact: bool,
}

impl Formatter {
    /// Creates a new `Formatter` from given format string.
    #[inline]
    pub fn try_new<S: AsRef<str>>(fmt: S) -> Result<Self> {
        let parser = FormatParser::new(fmt.as_ref().as_bytes());

        let mut fields = StackVec::new();

        for field in parser {
            if let Field::Invalid = field {
                return Err(Error::InvalidFormat(
                    "date format not recognized".try_to_string()?,
                ));
            }

            if fields.is_full() {
                return Err(Error::InvalidFormat(
                    "date format is too long for internal buffer".try_to_string()?,
                ));
            }

            fields.push(field);
        }

        Ok(Formatter {
            fields,
            format_exact: false,
        })
    }

    /// Formats datetime types
    #[inline]
    pub fn format<W: fmt::Write, T: DateTimeFormat>(&self, datetime: T, mut w: W) -> Result<()> {
        let dt = datetime.into();
        if dt.negative() {
            // negative interval
            w.write_char('-')?;
        } else if T::IS_INTERVAL_YM || T::IS_INTERVAL_DT {
            w.write_char('+')?;
        }

        let mut is_iso_format = false;

        let mut iter = self.fields.iter();
        while let Some(field) = iter.next() {
            match field {
                Field::Invalid => unreachable!(),
                Field::Blank(n) => {
                    for _ in 0..*n {
                        w.write_char(' ')?
                    }
                }
                Field::Hyphen => w.write_char('-')?,
                Field::Colon => w.write_char(':')?,
                Field::Slash => w.write_char('/')?,
                Field::Backslash => w.write_char('\\')?,
                Field::Comma => w.write_char(',')?,
                Field::Dot => {
                    if is_iso_format {
                        // when the next field is fraction, write the dot along with the fraction
                        let s = iter.as_slice();
                        if !matches!(s.first(), Some(&Field::Fraction(_))) {
                            w.write_char('.')?
                        }
                    } else {
                        w.write_char('.')?
                    }
                }
                Field::Semicolon => w.write_char(';')?,
                Field::Underline => w.write_char('_')?,
                Field::T => {
                    if !T::HAS_TIME {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                    is_iso_format = true;
                    if T::HAS_DATE {
                        w.write_char('T')?
                    }
                }
                Field::Year(n) => {
                    let year = if T::HAS_DATE {
                        dt.year() % (YEAR_MODIFIER[*n as usize - 1] as i32)
                    } else if T::IS_INTERVAL_YM {
                        dt.year()
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    };
                    write_u32(&mut w, year as u32, *n as usize)?;
                }
                Field::Month => {
                    if T::HAS_DATE || T::IS_INTERVAL_YM {
                        w.write_str(dt.month_str())?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Day => {
                    if T::HAS_DATE {
                        w.write_str(dt.day_str())?
                    } else if T::IS_INTERVAL_DT {
                        if dt.day() < 32 {
                            w.write_str(dt.day_str())?
                        } else {
                            write!(w, "{}", dt.day())?
                        }
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Hour24 => {
                    if T::HAS_TIME {
                        w.write_str(dt.hour24_str())?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Hour12 => {
                    if T::HAS_TIME && !T::IS_INTERVAL_DT {
                        w.write_str(dt.hour12_str())?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Minute => {
                    if T::HAS_TIME {
                        w.write_str(dt.minute_str())?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Second => {
                    if T::HAS_TIME {
                        w.write_str(dt.second_str())?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Fraction(p) => {
                    if T::HAS_FRACTION {
                        if is_iso_format {
                            if dt.usec() != 0 {
                                w.write_char('.')?;
                                let p = p.unwrap_or(6);
                                write_u32(&mut w, dt.fraction(p), p as usize)?;
                            }
                        } else {
                            let p = p.unwrap_or(6);
                            write_u32(&mut w, dt.fraction(p), p as usize)?;
                        }
                    } else if !is_iso_format {
                        // ignore fraction in ISO format when the datetime type has no fraction
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::AmPm(am_pm) => {
                    if T::HAS_TIME && !T::IS_INTERVAL_DT {
                        w.write_str(am_pm.format(dt.hour24()))?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::MonthName(style) => {
                    if T::HAS_DATE {
                        w.write_str(dt.month_name(*style))?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::DayName(style) => {
                    if T::HAS_DATE {
                        w.write_str(dt.week_day_name(datetime.date(), *style)?)?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::DayOfWeek => {
                    if T::HAS_DATE {
                        w.write_str(dt.day_of_week_str(datetime.date())?)?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::DayOfYear => {
                    if T::HAS_DATE {
                        w.write_str(dt.day_of_year_str())?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::WeekOfMonth => {
                    if T::HAS_DATE {
                        w.write_str(dt.week_of_month_str())?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::WeekOfYear => {
                    if T::HAS_DATE {
                        w.write_str(dt.week_of_year_str())?
                    } else {
                        return Err(Error::FormatError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Parses datetime types
    #[inline]
    pub fn parse<S: AsRef<str>, T: DateTimeFormat>(&self, input: S) -> Result<T> {
        let result = match self.format_exact {
            true => self.parse_internal::<S, T, true>(input),
            false => self.parse_internal::<S, T, false>(input),
        };
        if T::IS_INTERVAL_YM || T::IS_INTERVAL_DT {
            match result {
                Ok(_) => result,
                Err(e) => match e {
                    Error::ParseError(_) => Err(Error::ParseError(
                        "the interval is invalid".try_to_string()?,
                    )),
                    _ => Err(e),
                },
            }
        } else {
            result
        }
    }

    #[inline]
    fn parse_internal<S: AsRef<str>, T: DateTimeFormat, const FX: bool>(
        &self,
        input: S,
    ) -> Result<T> {
        const COMPATIBLE_SEPARATOR: [u8; 8] = [b'.', b':', b'-', b'/', b'\\', b',', b';', b'_'];

        let mut s = input.as_ref().as_bytes();
        let mut dt = NaiveDateTime::new();
        let mut need_time_fields = false;

        macro_rules! expect_number {
            ($max_len: expr) => {{
                let (neg, n, rem) = parse_number(s, $max_len)?;
                s = rem;
                (n, neg)
            }};
        }

        macro_rules! expect_time_field_with_tolerance {
            ($max_len: expr, $default: expr) => {{
                if s.is_empty() {
                    if need_time_fields {
                        return Err(Error::ParseError("time field is missing".try_to_string()?));
                    } else {
                        ($default, $default < 0)
                    }
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
        // If hour is not set, then is_hour24_set is None.
        // If it is set in hour24, then Some(true), else if in hour12, then Some(false)
        let mut is_hour24_set: Option<bool> = None;
        let mut is_min_set = false;
        let mut is_sec_set = false;
        let mut is_fraction_set = false;
        let mut is_iso_format = false;

        let mut dow: Option<WeekDay> = None;
        let mut doy: Option<u32> = None;
        let mut now: Option<Local> = None;
        let mut get_now = || {
            if now.is_none() {
                now = Some(Local::now());
            }
            now.unwrap()
        };

        for field in self.fields.iter() {
            if !FX {
                s = eat_whitespaces(s);
            }
            match field {
                // todo ignore the absence of symbols; Format exact
                Field::Invalid => unreachable!(),
                Field::Blank(_) => {}
                Field::Hyphen
                | Field::Colon
                | Field::Slash
                | Field::Backslash
                | Field::Comma
                | Field::Dot
                | Field::Semicolon
                | Field::Underline => match s.first() {
                    Some(ch) if COMPATIBLE_SEPARATOR.contains(ch) => {
                        s = &s[1..];
                    }
                    None => continue,
                    Some(ch) => {
                        if need_time_fields && *ch == b'Z' {
                            // ignore trailing 'Z' in ISO 8601 format
                            let rem = eat_whitespaces(&s[1..]);
                            if rem.is_empty() {
                                continue;
                            }
                        }
                        return Err(Error::ParseError(try_format!(
                            "the input {} is inconsistent with the format",
                            input.as_ref()
                        )?));
                    }
                },
                Field::T => {
                    if !T::HAS_TIME {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }

                    is_iso_format = true;

                    if expect_char(s, b'T') {
                        s = &s[1..];
                        need_time_fields = true;
                    } else if !s.is_empty() || !T::HAS_DATE {
                        // missing 'T' is allowed; When we don't have date, we *must* have time
                        need_time_fields = true;
                    }
                }
                Field::Year(n) => {
                    if T::HAS_DATE || T::IS_INTERVAL_YM {
                        if is_year_set {
                            return Err(Error::ParseError(
                                "format code (year) appears twice".try_to_string()?,
                            ));
                        }
                        let len = if T::IS_INTERVAL_YM {
                            T::YEAR_MAX_LENGTH
                        } else {
                            *n as usize
                        };
                        let (negative, year, rem) = parse_year(s, len, &mut get_now)?;
                        if T::HAS_DATE && (negative || year == 0) {
                            return Err(Error::ParseError(
                                "(full) year must be between 1 and 9999".try_to_string()?,
                            ));
                        }
                        dt.negative = negative;
                        dt.year = year;
                        s = rem;
                        is_year_set = true;
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Month => {
                    if T::HAS_DATE || T::IS_INTERVAL_YM {
                        if is_month_set {
                            return Err(Error::ParseError(
                                "format code (month) appears twice".try_to_string()?,
                            ));
                        }

                        let month = match parse_number(s, T::MONTH_MAX_LENGTH) {
                            Ok((negative, month, rem)) => {
                                if negative {
                                    return Err(Error::ParseError(
                                        "not a valid month".try_to_string()?,
                                    ));
                                }
                                s = rem;
                                month as u32
                            }
                            Err(_) => {
                                let (month, rem) = parse_month_name(s)?;
                                s = rem;
                                month as u32
                            }
                        };

                        dt.month = month;
                        is_month_set = true;
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Day => {
                    if T::HAS_DATE || T::IS_INTERVAL_DT {
                        if is_day_set {
                            return Err(Error::ParseError(
                                "format code (day) appears twice".try_to_string()?,
                            ));
                        }
                        let (day, negative) = expect_number!(T::DAY_MAX_LENGTH);
                        if T::HAS_DATE && negative {
                            return Err(Error::ParseError(
                                "day of month must be between 1 and last day of month"
                                    .try_to_string()?,
                            ));
                        }
                        dt.day = day.unsigned_abs();
                        dt.negative = negative;
                        is_day_set = true;
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Hour24 => {
                    if T::HAS_TIME {
                        if is_hour24_set.is_some() {
                            return Err(Error::ParseError(
                                "format code (hour) appears twice".try_to_string()?,
                            ));
                        }
                        if dt.ampm.is_some() {
                            return Err(Error::ParseError(
                                "'HH24' precludes use of meridian indicator".try_to_string()?,
                            ));
                        }
                        let (hour, negative) = if T::IS_INTERVAL_DT {
                            expect_number!(T::HOUR_MAX_LENGTH)
                        } else {
                            expect_time_field_with_tolerance!(T::HOUR_MAX_LENGTH, 0)
                        };
                        if negative {
                            return Err(Error::ParseError(
                                "hour must be between 0 and 23".try_to_string()?,
                            ));
                        }
                        dt.hour = hour as u32;
                        is_hour24_set = Some(true);
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Hour12 => {
                    if T::HAS_TIME && !T::IS_INTERVAL_DT {
                        if is_hour24_set.is_some() {
                            return Err(Error::ParseError(
                                "format code (hour) appears twice".try_to_string()?,
                            ));
                        }

                        let hour = if s.is_empty() {
                            0
                        } else {
                            let (hour, _) = expect_number!(T::HOUR_MAX_LENGTH);
                            if !(1..=12).contains(&hour) {
                                return Err(Error::ParseError(
                                    "hour must be between 1 and 12".try_to_string()?,
                                ));
                            }
                            hour
                        };

                        dt.hour = hour as u32;
                        dt.adjust_hour12();
                        is_hour24_set = Some(false);
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Minute => {
                    if T::HAS_TIME {
                        if is_min_set {
                            return Err(Error::ParseError(
                                "format code (minute) appears twice".try_to_string()?,
                            ));
                        }
                        let (minute, negative) = if T::IS_INTERVAL_DT {
                            expect_number!(T::MINUTE_MAX_LENGTH)
                        } else {
                            expect_time_field_with_tolerance!(T::MINUTE_MAX_LENGTH, 0)
                        };
                        if negative {
                            return Err(Error::ParseError(
                                "minutes must be between 0 and 59".try_to_string()?,
                            ));
                        }
                        dt.minute = minute as u32;
                        is_min_set = true;
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Second => {
                    if T::HAS_TIME {
                        if is_sec_set {
                            return Err(Error::ParseError(
                                "format code (second) appears twice".try_to_string()?,
                            ));
                        }
                        let (sec, negative) = if T::IS_INTERVAL_DT {
                            expect_number!(T::SECOND_MAX_LENGTH)
                        } else {
                            expect_time_field_with_tolerance!(T::SECOND_MAX_LENGTH, 0)
                        };
                        if negative {
                            return Err(Error::ParseError(
                                "seconds must be between 0 and 59".try_to_string()?,
                            ));
                        }
                        dt.sec = sec as u32;
                        is_sec_set = true;
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::Fraction(p) => {
                    if T::HAS_FRACTION || is_iso_format {
                        if is_fraction_set {
                            return Err(Error::ParseError(
                                "format code (fraction) appears twice".try_to_string()?,
                            ));
                        }
                        // When parsing, if FF is given, the default precision is 9
                        (dt.usec, s) = parse_fraction::<T>(s, p.unwrap_or(9) as usize)?;
                        is_fraction_set = true;
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::AmPm(style) => {
                    if T::HAS_TIME && !T::IS_INTERVAL_DT {
                        if dt.ampm.is_some() {
                            return Err(Error::ParseError(
                                "format code (am/pm) appears twice".try_to_string()?,
                            ));
                        }
                        if let Some(true) = is_hour24_set {
                            return Err(Error::ParseError(
                                "'HH24' precludes use of meridian indicator".try_to_string()?,
                            ));
                        }
                        let (am_pm, rem) = parse_ampm(s, style)?;
                        s = rem;

                        dt.ampm = am_pm;
                        if dt.ampm.is_some() {
                            dt.adjust_hour12();
                        }
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::MonthName(_) => {
                    if T::HAS_DATE {
                        if is_month_set {
                            return Err(Error::ParseError(
                                "format code (month) appears twice".try_to_string()?,
                            ));
                        }
                        let (month, rem) = parse_month_name(s)?;
                        s = rem;

                        dt.month = month as u32;
                        is_month_set = true;
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::DayName(style) => {
                    if T::HAS_DATE {
                        if dow.is_some() {
                            return Err(Error::ParseError(
                                "format code (day of week) appears twice".try_to_string()?,
                            ));
                        }
                        let (d, rem) = parse_week_day_name(s, *style)?;
                        s = rem;

                        dow = Some(d);
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::DayOfWeek => {
                    if T::HAS_DATE {
                        if dow.is_some() {
                            return Err(Error::ParseError(
                                "format code (day of week) appears twice".try_to_string()?,
                            ));
                        }
                        let (d, rem) = parse_week_day_number(s)?;
                        s = rem;

                        dow = Some(d);
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::DayOfYear => {
                    if T::HAS_DATE {
                        if doy.is_some() {
                            return Err(Error::ParseError(
                                "format code (day of year) appears twice".try_to_string()?,
                            ));
                        }
                        let (days, negative) = expect_number!(T::DAY_OF_YEAR_MAX_LENGTH);
                        if negative {
                            return Err(Error::ParseError(
                                "day of year must be between 1 and 365 (366 for leap year)"
                                    .try_to_string()?,
                            ));
                        }
                        doy = Some(days as u32);
                    } else {
                        return Err(Error::ParseError(
                            "date format not recognized".try_to_string()?,
                        ));
                    }
                }
                Field::WeekOfMonth => {
                    return Err(Error::ParseError(
                        "format code (week of month) cannot appear in date input format"
                            .try_to_string()?,
                    ))
                }
                Field::WeekOfYear => {
                    return Err(Error::ParseError(
                        "format code (week of year) cannot appear in date input format"
                            .try_to_string()?,
                    ))
                }
            }
        }

        if !FX {
            s = eat_whitespaces(s);
        }

        if need_time_fields && s.first() == Some(&b'Z') {
            // ignore trailing 'Z' in ISO 8601 format
            s = eat_whitespaces(&s[1..]);
        }

        if !s.is_empty() {
            return Err(Error::ParseError(
                "format picture ends before converting entire input string".try_to_string()?,
            ));
        }

        if T::HAS_DATE {
            match (is_year_set, is_month_set) {
                (true, true) => {}
                (true, false) => {
                    let datetime_now = get_now();
                    dt.month = datetime_now.month();
                }
                (false, false) => {
                    let datetime_now = get_now();
                    dt.year = datetime_now.year();
                    dt.month = datetime_now.month();
                }
                (false, true) => {
                    let datetime_now = get_now();
                    dt.year = datetime_now.year();
                }
            }
        }

        if let Some(d) = doy {
            let is_leap_year = is_leap_year(dt.year);
            if d == 0 || (!is_leap_year && d > 365) || (is_leap_year && d > 366) {
                return Err(Error::ParseError(
                    "day of year must be between 1 and 365 (366 for leap year)".try_to_string()?,
                ));
            }
            let (month, day) = the_month_day_of_days(d, is_leap_year);
            match (is_month_set, is_day_set) {
                (true, true) => {
                    if month != dt.month || day != dt.day {
                        return Err(Error::ParseError(
                            "day of year conflicts with Julian date".try_to_string()?,
                        ));
                    }
                }
                (true, false) => {
                    if month != dt.month {
                        return Err(Error::ParseError(
                            "day of year conflicts with Julian date".try_to_string()?,
                        ));
                    }
                    dt.day = day;
                }
                (false, true) => {
                    if day != dt.day {
                        return Err(Error::ParseError(
                            "day of year conflicts with Julian date".try_to_string()?,
                        ));
                    }
                    dt.month = month;
                }
                (false, false) => {
                    dt.month = month;
                    dt.day = day;
                }
            }
        }

        // Check if parsed day of week conflicts with the date
        if let Some(d) = dow {
            let date = Date::try_from(&dt)?;
            if date.day_of_week() != d {
                return Err(Error::ParseError(
                    "day of week conflicts with Julian date".try_to_string()?,
                ));
            }
        }

        T::try_from(dt)
    }
}

fn write_u32<W: fmt::Write>(mut w: W, value: u32, width: usize) -> Result<()> {
    debug_assert!(width < 11 && width > 0);
    let mut buf: [u8; 11] = [b'0'; 11];
    let mut index: usize = 10;

    let mut val = value;
    while val >= 10 {
        let v = val % 10;
        val /= 10;

        buf[index] = v as u8 + b'0';
        index -= 1;
    }

    buf[index] = val as u8 + b'0';

    let len = 11 - index;
    if width > len {
        index -= width - len;
    }
    let s = unsafe { std::str::from_utf8_unchecked(&buf[index..11]) };

    w.write_str(s)?;
    Ok(())
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
        None => {
            return Err(Error::ParseError(
                "the input is inconsistent with the format".try_to_string()?,
            ))
        }
    };

    let (digits, s) = eat_digits(s, max_len);
    if digits.is_empty() {
        return Err(Error::ParseError(
            "a non-numeric character was found where a numeric was expected".try_to_string()?,
        ));
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
fn eat_whitespaces(s: &[u8]) -> &[u8] {
    let i = s.iter().take_while(|&i| i.is_ascii_whitespace()).count();
    &s[i..]
}

#[inline]
fn parse_year<'a, T: FnMut() -> Local>(
    input: &'a [u8],
    max_len: usize,
    get_now: &mut T,
) -> Result<(bool, i32, &'a [u8])> {
    // todo do not allow sign element 's' before y/yy/yyy in the format string
    match max_len {
        2 => {
            let input_len = input.len();
            let (negative, year, rem) = parse_number(input, 4)?;
            if input_len - rem.len() > 2 {
                Ok((negative, year, rem))
            } else {
                let now = get_now();
                let current_year = now.year();
                let result_year = current_year - current_year % 100 + year;
                Ok((negative, result_year, rem))
            }
        }
        1 | 3 => {
            let (negative, year, rem) = parse_number(input, max_len)?;
            let now = get_now();
            let current_year = now.year();
            let result_year =
                current_year - current_year % YEAR_MODIFIER[max_len - 1] as i32 + year;
            Ok((negative, result_year, rem))
        }
        _ => parse_number(input, max_len),
    }
}

#[inline]
fn parse_ampm<'a>(s: &'a [u8], style: &'a AmPmStyle) -> Result<(Option<AmPm>, &'a [u8])> {
    if s.is_empty() {
        return Ok((None, s));
    }
    match style {
        AmPmStyle::LowerDot | AmPmStyle::UpperDot => {
            if CaseInsensitive::starts_with(s, b"A.M.") {
                Ok((Some(AmPm::Am), &s[4..]))
            } else if CaseInsensitive::starts_with(s, b"P.M.") {
                Ok((Some(AmPm::Pm), &s[4..]))
            } else {
                Err(Error::ParseError(
                    "AM/A.M. or PM/P.M. required".try_to_string()?,
                ))
            }
        }
        AmPmStyle::Upper | AmPmStyle::Lower => {
            if CaseInsensitive::starts_with(s, b"AM") {
                Ok((Some(AmPm::Am), &s[2..]))
            } else if CaseInsensitive::starts_with(s, b"PM") {
                Ok((Some(AmPm::Pm), &s[2..]))
            } else {
                Err(Error::ParseError(
                    "AM/A.M. or PM/P.M. required".try_to_string()?,
                ))
            }
        }
    }
}

#[inline]
fn parse_fraction<T: DateTimeFormat>(s: &[u8], max_len: usize) -> Result<(u32, &[u8])> {
    match s.first() {
        Some(ch) => {
            if *ch == b'-' {
                return Err(Error::ParseError(
                    "the fractional seconds must be between 0 and 999999".try_to_string()?,
                ));
            }
        }
        None => {
            return Ok((0, s));
        }
    }

    let (digits, s) = eat_digits(s, max_len);
    let int = digits
        .iter()
        .fold(0, |int, &i| int * 10 + (i - b'0') as i32);
    Ok((
        // Truncate when we want time, but do not want fractions.  Matches oracle::Date.
        match T::HAS_FRACTION {
            false => 0u32,
            _ => (int as f64 * FRACTION_FACTOR[digits.len()]).round() as u32,
        },
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

    Err(Error::ParseError("not a valid month".try_to_string()?))
}

#[inline]
fn parse_week_day_name(s: &[u8], style: NameStyle) -> Result<(WeekDay, &[u8])> {
    match style {
        Capital | NameStyle::Lower | NameStyle::Upper => {
            for (index, day) in DAY_NAME_TABLE[Capital as usize].iter().enumerate() {
                if CaseInsensitive::starts_with(s, day.as_bytes()) {
                    return Ok((WeekDay::from(index + 1), &s[day.len()..]));
                }
            }
        }
        AbbrCapital | NameStyle::AbbrLower | NameStyle::AbbrUpper => {
            for (index, day) in DAY_NAME_TABLE[AbbrCapital as usize].iter().enumerate() {
                if CaseInsensitive::starts_with(s, day.as_bytes()) {
                    return Ok((WeekDay::from(index + 1), &s[day.len()..]));
                }
            }
        }
    }

    Err(Error::ParseError(
        "not a valid day of the week".try_to_string()?,
    ))
}

#[inline]
fn parse_week_day_number(s: &[u8]) -> Result<(WeekDay, &[u8])> {
    if s.is_empty() {
        return Err(Error::ParseError(
            "not a valid day of the week".try_to_string()?,
        ));
    }

    let num = s[0] - b'0';
    if (1..=7).contains(&num) {
        return Ok((WeekDay::from(num as usize), &s[1..]));
    }

    Err(Error::ParseError(
        "not a valid day of the week".try_to_string()?,
    ))
}

pub struct LazyFormat<T: DateTimeFormat> {
    fmt: Formatter,
    dt: T,
}

impl<T: DateTimeFormat> LazyFormat<T> {
    #[inline]
    pub fn new(fmt: Formatter, dt: T) -> Self {
        LazyFormat { fmt, dt }
    }
}

impl<T: DateTimeFormat> fmt::Display for LazyFormat<T> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.fmt.format(self.dt, f).map_err(|_| fmt::Error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::format::AmPmStyle::{Lower as AmLower, LowerDot, Upper as AmUpper, UpperDot};
    use crate::format::Field::{AmPm, Blank, DayName, DayOfWeek, DayOfYear, MonthName};
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
        assert_eq!(parser.next(), Some(Field::Blank(1)));
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
            b"MONTH  Month    month MON mon Mon DAY day Day DY Dy dy AM am A.M. a.m. D d DDD",
        );

        let expect = [
            MonthName(Upper),
            Blank(2),
            MonthName(Capital),
            Blank(4),
            MonthName(Lower),
            Blank(1),
            MonthName(AbbrUpper),
            Blank(1),
            MonthName(AbbrLower),
            Blank(1),
            MonthName(AbbrCapital),
            Blank(1),
            DayName(Upper),
            Blank(1),
            DayName(Lower),
            Blank(1),
            DayName(Capital),
            Blank(1),
            DayName(AbbrUpper),
            Blank(1),
            DayName(AbbrCapital),
            Blank(1),
            DayName(AbbrLower),
            Blank(1),
            AmPm(AmUpper),
            Blank(1),
            AmPm(AmLower),
            Blank(1),
            AmPm(UpperDot),
            Blank(1),
            AmPm(LowerDot),
            Blank(1),
            DayOfWeek,
            Blank(1),
            DayOfWeek,
            Blank(1),
            DayOfYear,
        ];
        for e in expect.iter() {
            assert_eq!(e, &parser.next().unwrap())
        }
        assert_eq!(None, parser.next())
    }

    #[test]
    fn test_formatter() {
        assert!(Formatter::try_new(
            "                                                             "
        )
        .is_ok());
    }

    #[test]
    fn generate_array() {
        let mut res = String::new();
        res.push('[');
        for i in 0..10000 {
            res.push('"');
            if i < 10 {
                res.push('0');
            }
            res.push_str(&i.to_string());
            res.push('"');
            res.push_str(", ")
        }
        res.push(']');
        println!("{}", res);
    }

    #[test]
    fn test_write_u32() {
        fn assert(val: u32, expected: &str, width: usize) {
            let mut s = String::with_capacity(10);
            write_u32(&mut s, val, width).unwrap();
            assert_eq!(expected, &s);
        }

        assert(0, "000000000", 9);
        assert(0, "00000", 5);
        assert(0, "00", 2);
        assert(1, "000000001", 9);
        assert(1, "1", 1);
        assert(1, "01", 2);
        assert(1, "001", 3);
        assert(23457, "23457", 3);
        assert(10, "000000010", 9);
        assert(11, "0000000011", 10);
        assert(101, "0000000101", 10);
        assert(10101, "000010101", 9);
        assert(123456789, "0123456789", 10);
        assert(123456789, "123456789", 9);
        assert(u32::MAX, "4294967295", 10);
    }

    #[test]
    fn test_week_of_year_table() {
        for i in 1..=366 {
            let week = (i + 6) / 7;
            let week_str = if week < 10 {
                format!("0{}", week)
            } else {
                week.to_string()
            };
            assert_eq!(week_str.as_str(), WEEK_OF_YEAR_TABLE[i as usize]);
        }
    }

    #[test]
    fn test_day_of_year_table() {
        for day in 1..=366 {
            let day_str = if day < 10 {
                format!("00{}", day)
            } else if day < 100 {
                format!("0{}", day)
            } else {
                day.to_string()
            };
            assert_eq!(day_str.as_str(), DAY_OF_YEAR_TABLE[day as usize]);
        }
    }

    #[test]
    fn test_parse_fmt_compatible() {
        let fmts = [
            "yyyy.mm.dd.hh24.mi.ss.ff",
            "yyyy:mm:dd:hh24:mi:ss:ff",
            "yyyy-mm-dd-hh24-mi-ss-ff",
            "yyyy/mm/dd/hh24/mi/ss/ff",
            "yyyy\\mm\\dd\\hh24\\mi\\ss\\ff",
            "yyyy,mm,dd,hh24,mi,ss,ff",
            "yyyy;mm;dd;hh24;mi;ss;ff",
            "yyyy_mm_dd_hh24_mi_ss_ff",
        ];

        for fmt in fmts {
            // Normal punctuations
            assert_eq!(
                Timestamp::new(
                    Date::try_from_ymd(2022, 6, 18).unwrap(),
                    Time::try_from_hms(3, 4, 5, 6).unwrap(),
                ),
                Timestamp::parse("2022.06:18-03/04\\05,000006", fmt).unwrap(),
            );
            // All of the same punctuations
            assert_eq!(
                Timestamp::new(
                    Date::try_from_ymd(2022, 6, 18).unwrap(),
                    Time::try_from_hms(3, 4, 5, 6).unwrap(),
                ),
                Timestamp::parse("2022-06-18-03-04-05-000006", fmt).unwrap()
            );
            assert_eq!(
                Timestamp::new(
                    Date::try_from_ymd(2022, 6, 18).unwrap(),
                    Time::try_from_hms(3, 4, 5, 6).unwrap(),
                ),
                Timestamp::parse("2022.06.18.03.04.05.000006", fmt).unwrap()
            );
            assert_eq!(
                Timestamp::new(
                    Date::try_from_ymd(2022, 6, 18).unwrap(),
                    Time::try_from_hms(3, 4, 5, 6).unwrap(),
                ),
                Timestamp::parse("2022:06:18:03:04:05:000006", fmt).unwrap()
            );
            assert_eq!(
                Timestamp::new(
                    Date::try_from_ymd(2022, 6, 18).unwrap(),
                    Time::try_from_hms(3, 4, 5, 6).unwrap(),
                ),
                Timestamp::parse("2022_06_18_03_04_05_000006", fmt).unwrap()
            );
        }

        // Test different numbers or kinds of punctuations
        assert!(
            Timestamp::parse("2022-06-18 03:04:05.000006", "yyyy-mm-dd hh24:mi:ss..ff").is_err()
        );
        assert!(
            Timestamp::parse("2022-06-18 03:04:05..000006", "yyyy-mm-dd hh24:mi:ss..ff").is_ok()
        );
        assert!(
            Timestamp::parse("2022-06-18 03:04:05.:000006", "yyyy-mm-dd hh24:mi:ss..ff").is_ok()
        );
        assert!(
            Timestamp::parse("2022-06-18 03:04:05.:000006", "yyyy-mm-dd hh24:mi:ss__ff").is_ok()
        );

        // Ignore all Blank
        assert!(Timestamp::parse(
            "2022-06-18:::03:04:05.000006",
            "yyyy-mm-dd   :::hh24:mi:ss.ff"
        )
        .is_ok());
        assert!(Timestamp::parse(
            "2022-06-18 :::03:04:05.000006",
            "yyyy-mm-dd   :::hh24:mi:ss.ff"
        )
        .is_ok());
        assert!(Timestamp::parse(
            "2022-06-18 : : : 03:04:05.000006",
            "yyyy-mm-dd   :::hh24:mi:ss.ff"
        )
        .is_ok());
    }

    #[test]
    fn test_date_year_parse() {
        let fmt = Formatter::try_new("YYYY").unwrap();
        assert!(match fmt.parse::<_, Date>("0") {
            Ok(_) => false,
            Err(Error::ParseError(s)) => {
                s == "(full) year must be between 1 and 9999"
            }
            Err(_) => false,
        });

        let fmt = Formatter::try_new("Y").unwrap();
        assert!(fmt.parse::<_, Date>("0").is_ok());

        let fmt = Formatter::try_new("YYY").unwrap();
        assert!(fmt.parse::<_, Date>("000").is_ok());

        let fmt = Formatter::try_new("YY").unwrap();
        assert!(fmt.parse::<_, Date>("00").is_ok());

        let fmt = Formatter::try_new("YY").unwrap();
        assert!(match fmt.parse::<_, Date>("0000") {
            Ok(_) => false,
            Err(Error::ParseError(s)) => {
                s == "(full) year must be between 1 and 9999"
            }
            Err(_) => false,
        });

        let fmt = Formatter::try_new("YYYY-MM-DD HH24:MI:SS.FF6").unwrap();
        assert!(match fmt.parse::<_, Timestamp>("0") {
            Ok(_) => false,
            Err(Error::ParseError(s)) => {
                s == "(full) year must be between 1 and 9999"
            }
            Err(_) => false,
        });
    }
}
