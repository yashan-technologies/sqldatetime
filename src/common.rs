//! Common structures, constants and functions.

pub const MONTHS_PER_YEAR: u32 = 12;
pub const HOURS_PER_DAY: u32 = 24;
pub const MINUTES_PER_HOUR: u32 = 60;
pub const SECONDS_PER_MINUTE: u32 = 60;

pub const USECONDS_MAX: u32 = 999_999;
pub const USECONDS_PER_DAY: i64 = 86_400_000_000;
pub const USECONDS_PER_HOUR: i64 = 3_600_000_000;
pub const USECONDS_PER_MINUTE: i64 = 60_000_000;
pub const USECONDS_PER_SECOND: i64 = 1_000_000;

pub const DATE_MIN_YEAR: i32 = 1;
pub const DATE_MAX_YEAR: i32 = 9999;

pub const UNIX_EPOCH_JULIAN: i32 = date2julian(1970, 1, 1);

pub const DATE_MIN_JULIAN: i32 = date2julian(DATE_MIN_YEAR, 1, 1);
pub const DATE_MAX_JULIAN: i32 = date2julian(DATE_MAX_YEAR, 12, 31);

pub const TIMESTAMP_MIN: i64 = (DATE_MIN_JULIAN - UNIX_EPOCH_JULIAN) as i64 * USECONDS_PER_DAY;
pub const TIMESTAMP_MAX: i64 =
    (date2julian(10000, 1, 1) - UNIX_EPOCH_JULIAN) as i64 * USECONDS_PER_DAY - 1;

/// Calendar date to Julian day conversion.
/// Julian date is commonly used in astronomical applications,
/// since it is numerically accurate and computationally simple.
/// The algorithms here will accurately convert between Julian day
/// and calendar date for all non-negative Julian days
/// (i.e. from Nov 24, -4713 on).
#[inline]
pub const fn date2julian(year: i32, month: u32, day: u32) -> i32 {
    let (y, m) = if month > 2 {
        (year + 4800, month + 1)
    } else {
        (year + 4799, month + 13)
    };

    let century = y / 100;

    let mut julian = y * 365 - 32167;
    julian += y / 4 - century + century / 4;
    julian += 7834 * m as i32 / 256 + day as i32;

    julian
}

/// Julian day to Calendar date conversion.
#[inline]
pub const fn julian2date(julian_day: i32) -> (i32, u32, u32) {
    let mut julian = julian_day as u32 + 32044;
    let mut quad = julian / 146097;
    let extra = (julian - quad * 146097) * 4 + 3;
    julian += 60 + quad * 3 + extra / 146097;
    quad = julian / 1461;
    julian -= quad * 1461;

    let mut y: i32 = (julian * 4 / 1461) as i32;
    julian = if y != 0 {
        (julian + 305) % 365 + 123
    } else {
        (julian + 306) % 366 + 123
    };
    y += (quad * 4) as i32;
    let year = y - 4800;
    quad = julian * 2141 / 65_536;

    let day = julian - 7834 * quad / 256;
    let month = (quad + 10) % MONTHS_PER_YEAR as u32 + 1;

    (year, month, day)
}

#[inline(always)]
pub const fn is_valid_date(date: i32) -> bool {
    date >= (DATE_MIN_JULIAN - UNIX_EPOCH_JULIAN) && date <= (DATE_MAX_JULIAN - UNIX_EPOCH_JULIAN)
}

#[inline(always)]
pub const fn is_valid_timestamp(timestamp: i64) -> bool {
    timestamp >= TIMESTAMP_MIN && timestamp <= TIMESTAMP_MAX
}

#[inline(always)]
pub const fn is_valid_time(time: i64) -> bool {
    time >= 0 && time < USECONDS_PER_DAY
}

#[inline(always)]
const fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && ((year % 100) != 0 || (year % 400) == 0)
}

#[inline(always)]
pub const fn days_of_month(year: i32, month: u32) -> u32 {
    const DAY_TABLE: [[u32; 12]; 2] = [
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    ];

    DAY_TABLE[is_leap_year(year) as usize][month as usize - 1]
}

#[inline(always)]
pub const fn the_day_of_year(year: i32, month: u32, day: u32) -> u32 {
    const SUM_OF_DAYS_TABLE: [[u32; 12]; 2] = [
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334],
        [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335],
    ];

    SUM_OF_DAYS_TABLE[is_leap_year(year) as usize][month as usize - 1] + day
}
