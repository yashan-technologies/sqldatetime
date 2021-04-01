//! Interval implementation.

use crate::common::MONTHS_PER_YEAR;
use crate::error::{Error, Result};

const INTERVAL_MAX_YEAR: i32 = 178000000;

/// `Year-Month Interval` represents the duration of a period of time,
/// has an interval precision that includes a YEAR field or a MONTH field, or both.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct IntervalYm(i32);

impl IntervalYm {
    /// The smallest interval that can be represented by `IntervalYm`, i.e. `-178000000-00`.
    pub const MIN: Self = unsafe { IntervalYm::from_ym_unchecked(-178000000, 0) };

    /// The largest interval that can be represented by `IntervalYm`, i.e. `178000000-00`.
    pub const MAX: Self = unsafe { IntervalYm::from_ym_unchecked(178000000, 0) };

    /// The zero value of interval, i.e. `00-00`.
    pub const ZERO: Self = IntervalYm(0);

    /// Creates a `IntervalYm` from the given year and month.
    ///
    /// # Safety
    /// This function is unsafe because the values are not checked for validity!
    /// Before using it, check that the values are all correct.
    #[inline]
    pub const unsafe fn from_ym_unchecked(year: i32, month: i32) -> Self {
        let months = if year >= 0 {
            year * MONTHS_PER_YEAR as i32 + month
        } else {
            year * MONTHS_PER_YEAR as i32 - month
        };
        IntervalYm(months)
    }

    /// Creates a `IntervalYm` from the given year and month.
    #[inline]
    pub const fn try_from_ym(year: i32, month: i32) -> Result<Self> {
        if IntervalYm::is_valid(year, month) {
            Ok(unsafe { IntervalYm::from_ym_unchecked(year, month) })
        } else {
            Err(Error::OutOfRange)
        }
    }

    /// Checks if the given year and month are valid.
    #[inline]
    pub const fn is_valid(year: i32, month: i32) -> bool {
        if year > INTERVAL_MAX_YEAR || year < -INTERVAL_MAX_YEAR {
            return false;
        }

        if (year == INTERVAL_MAX_YEAR || year == -INTERVAL_MAX_YEAR) && month != 0 {
            return false;
        }

        if month < 0 || month >= MONTHS_PER_YEAR as i32 {
            return false;
        }

        true
    }

    /// Gets the value of `IntervalYm`.
    #[allow(dead_code)]
    #[inline(always)]
    pub(crate) const fn value(self) -> i32 {
        self.0
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub(crate) const unsafe fn from_value_unchecked(value: i32) -> Self {
        IntervalYm(value)
    }

    /// Extracts `(year, month)` from the interval.
    #[inline]
    pub const fn extract(self) -> (i32, i32) {
        let year = self.0 / MONTHS_PER_YEAR as i32;
        let month = if year >= 0 {
            self.0 - year * MONTHS_PER_YEAR as i32
        } else {
            -self.0 + year * MONTHS_PER_YEAR as i32
        };
        (year, month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_ym() {
        let interval = IntervalYm::try_from_ym(0, 0).unwrap();
        assert_eq!(interval.value(), 0);
        assert_eq!(interval.extract(), (0, 0));

        let interval = IntervalYm::try_from_ym(178000000, 0).unwrap();
        assert_eq!(interval.extract(), (178000000, 0));

        let interval = IntervalYm::try_from_ym(-178000000, 0).unwrap();
        assert_eq!(interval.extract(), (-178000000, 0));

        let interval = IntervalYm::try_from_ym(177999999, 11).unwrap();
        assert_eq!(interval.extract(), (177999999, 11));

        let interval = IntervalYm::try_from_ym(-177999999, 11).unwrap();
        assert_eq!(interval.extract(), (-177999999, 11));
    }
}
