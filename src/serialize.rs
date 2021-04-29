//! Impl the `serde::Serialize` and `serde::Deserialize` traits.

use crate::{Date, Formatter, IntervalDT, IntervalYM, Time, Timestamp};
use once_cell::sync::Lazy;
use serde_crate::de::Visitor;
use serde_crate::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

static DATE_FORMATTER: Lazy<Formatter> = Lazy::new(|| Formatter::try_new("YYYY-MM-DD").unwrap());
static TIMESTAMP_FORMATTER: Lazy<Formatter> =
    Lazy::new(|| Formatter::try_new("YYYY-MM-DD HH24:MI:SS.FF6").unwrap());
static TIME_FORMATTER: Lazy<Formatter> =
    Lazy::new(|| Formatter::try_new("HH24:MI:SS.FF6").unwrap());
static INTERVAL_YM_FORMATTER: Lazy<Formatter> =
    Lazy::new(|| Formatter::try_new("YYYY-MM").unwrap());
static INTERVAL_DT_FORMATTER: Lazy<Formatter> =
    Lazy::new(|| Formatter::try_new("DD HH24:MI:SS.FF6").unwrap());

impl Serialize for Date {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut buf = String::new();
            DATE_FORMATTER
                .format(*self, &mut buf)
                .map_err(ser::Error::custom)?;
            serializer.serialize_str(&buf)
        } else {
            serializer.serialize_i32(self.value())
        }
    }
}

impl<'de> Deserialize<'de> for Date {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DateVisitor;

        impl<'de> Visitor<'de> for DateVisitor {
            type Value = Date;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a Date")
            }

            #[inline]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(unsafe { Date::from_value_unchecked(v) })
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                DATE_FORMATTER.parse(v).map_err(de::Error::custom)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(DateVisitor)
        } else {
            deserializer.deserialize_i32(DateVisitor)
        }
    }
}

impl Serialize for Timestamp {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut buf = String::new();
            TIMESTAMP_FORMATTER
                .format(*self, &mut buf)
                .map_err(ser::Error::custom)?;
            serializer.serialize_str(&buf)
        } else {
            serializer.serialize_i64(self.value())
        }
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimestampVisitor;

        impl<'de> Visitor<'de> for TimestampVisitor {
            type Value = Timestamp;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a Timestamp")
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(unsafe { Timestamp::from_value_unchecked(v) })
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                TIMESTAMP_FORMATTER.parse(v).map_err(de::Error::custom)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(TimestampVisitor)
        } else {
            deserializer.deserialize_i64(TimestampVisitor)
        }
    }
}

impl Serialize for Time {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut buf = String::new();
            TIME_FORMATTER
                .format(*self, &mut buf)
                .map_err(ser::Error::custom)?;
            serializer.serialize_str(&buf)
        } else {
            serializer.serialize_i64(self.value())
        }
    }
}

impl<'de> Deserialize<'de> for Time {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TimeVisitor;

        impl<'de> Visitor<'de> for TimeVisitor {
            type Value = Time;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a Time")
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(unsafe { Time::from_value_unchecked(v) })
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                TIME_FORMATTER.parse(v).map_err(de::Error::custom)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(TimeVisitor)
        } else {
            deserializer.deserialize_i64(TimeVisitor)
        }
    }
}

impl Serialize for IntervalYM {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut buf = String::new();
            INTERVAL_YM_FORMATTER
                .format(*self, &mut buf)
                .map_err(ser::Error::custom)?;
            serializer.serialize_str(&buf)
        } else {
            serializer.serialize_i32(self.value())
        }
    }
}

impl<'de> Deserialize<'de> for IntervalYM {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IntervalVisitor;

        impl<'de> Visitor<'de> for IntervalVisitor {
            type Value = IntervalYM;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a IntervalYM")
            }

            #[inline]
            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(unsafe { IntervalYM::from_value_unchecked(v) })
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                INTERVAL_YM_FORMATTER.parse(v).map_err(de::Error::custom)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(IntervalVisitor)
        } else {
            deserializer.deserialize_i32(IntervalVisitor)
        }
    }
}

impl Serialize for IntervalDT {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut buf = String::new();
            INTERVAL_DT_FORMATTER
                .format(*self, &mut buf)
                .map_err(ser::Error::custom)?;
            serializer.serialize_str(&buf)
        } else {
            serializer.serialize_i64(self.value())
        }
    }
}

impl<'de> Deserialize<'de> for IntervalDT {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct IntervalVisitor;

        impl<'de> Visitor<'de> for IntervalVisitor {
            type Value = IntervalDT;

            #[inline]
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a IntervalDT")
            }

            #[inline]
            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(unsafe { IntervalDT::from_value_unchecked(v) })
            }

            #[inline]
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                INTERVAL_DT_FORMATTER.parse(v).map_err(de::Error::custom)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(IntervalVisitor)
        } else {
            deserializer.deserialize_i64(IntervalVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_date(year: i32, mon: u32, day: u32) {
        let date = Date::try_from_ymd(year, mon, day).unwrap();
        let date_json = serde_json::to_string(&date).unwrap();
        let json_decode: Date = serde_json::from_str(&date_json).unwrap();
        assert_eq!(
            date_json,
            format!("\"{}\"", date.format("YYYY-MM-DD").unwrap())
        );
        assert_eq!(json_decode, date);

        let bin = bincode::serialize(&date).unwrap();
        let bin_decode: Date = bincode::deserialize(&bin).unwrap();
        assert_eq!(bin_decode, date);
    }

    #[test]
    fn test_serde_date() {
        test_date(1, 1, 1);
        test_date(1234, 12, 31);
        test_date(1969, 12, 30);
        test_date(1969, 12, 31);
        test_date(1970, 1, 1);
        test_date(2000, 1, 1);
        test_date(9999, 12, 31);
    }

    fn test_timestamp(year: i32, mon: u32, day: u32, hour: u32, min: u32, sec: u32, usec: u32) {
        let date = Date::try_from_ymd(year, mon, day).unwrap();
        let time = Time::try_from_hms(hour, min, sec, usec).unwrap();
        let timestamp = Timestamp::new(date, time);
        let ts_json = serde_json::to_string(&timestamp).unwrap();
        assert_eq!(
            ts_json,
            format!(
                "\"{}\"",
                timestamp.format("YYYY-MM-DD HH24:MI:SS.FF6").unwrap()
            )
        );
        let json_decode: Timestamp = serde_json::from_str(&ts_json).unwrap();
        assert_eq!(json_decode, timestamp);

        let bin = bincode::serialize(&timestamp).unwrap();
        let bin_decode: Timestamp = bincode::deserialize(&bin).unwrap();
        assert_eq!(bin_decode, timestamp);
    }

    #[test]
    fn test_serde_timestamp() {
        test_timestamp(0001, 1, 1, 0, 0, 0, 0);
        test_timestamp(0001, 1, 1, 1, 1, 1, 1);
        test_timestamp(1969, 12, 30, 23, 30, 30, 30);
        test_timestamp(1969, 12, 31, 23, 59, 59, 999999);
        test_timestamp(1970, 1, 1, 0, 0, 0, 0);
        test_timestamp(1970, 10, 1, 23, 30, 0, 0);
        test_timestamp(9999, 12, 31, 23, 59, 59, 999999);
    }

    fn test_time(hour: u32, min: u32, sec: u32, usec: u32) {
        let time = Time::try_from_hms(hour, min, sec, usec).unwrap();
        let time_json = serde_json::to_string(&time).unwrap();

        let json_decode: Time = serde_json::from_str(&time_json).unwrap();
        assert_eq!(
            time_json,
            format!("\"{}\"", time.format("hh24:mi:ss.ff6").unwrap())
        );
        assert_eq!(json_decode, time);

        let bin = bincode::serialize(&time).unwrap();
        let bin_decode: Time = bincode::deserialize(&bin).unwrap();
        assert_eq!(bin_decode, time);
    }

    #[test]
    fn test_serde_time() {
        test_time(0, 0, 0, 0);
        test_time(1, 1, 1, 1);
        test_time(23, 59, 59, 999999);
    }

    fn test_interval_ym(negate: bool, year: u32, mon: u32) {
        let interval = if negate {
            IntervalYM::try_from_ym(year, mon).unwrap().negate()
        } else {
            IntervalYM::try_from_ym(year, mon).unwrap()
        };
        let interval_json = serde_json::to_string(&interval).unwrap();
        let json_decode: IntervalYM = serde_json::from_str(&interval_json).unwrap();
        assert_eq!(
            interval_json,
            format!("\"{}\"", interval.format("YYYY-MM").unwrap())
        );
        assert_eq!(json_decode, interval);

        let bin = bincode::serialize(&interval).unwrap();
        let bin_decode: IntervalYM = bincode::deserialize(&bin).unwrap();
        assert_eq!(bin_decode, interval);
    }

    #[test]
    fn test_serde_interval_ym() {
        test_interval_ym(false, 0000, 0);
        test_interval_ym(false, 0000, 1);
        test_interval_ym(false, 0001, 1);
        test_interval_ym(false, 178000000, 0);
        test_interval_ym(true, 0000, 1);
        test_interval_ym(true, 0001, 1);
        test_interval_ym(true, 178000000, 0);
    }

    fn test_interval_dt(negate: bool, day: u32, hour: u32, min: u32, sec: u32, usec: u32) {
        let interval = if negate {
            IntervalDT::try_from_dhms(day, hour, min, sec, usec)
                .unwrap()
                .negate()
        } else {
            IntervalDT::try_from_dhms(day, hour, min, sec, usec).unwrap()
        };
        let interval_json = serde_json::to_string(&interval).unwrap();
        let json_decode: IntervalDT = serde_json::from_str(&interval_json).unwrap();
        assert_eq!(
            interval_json,
            format!("\"{}\"", interval.format("DD hh24:mi:ss.ff6").unwrap())
        );
        assert_eq!(json_decode, interval);

        let bin = bincode::serialize(&interval).unwrap();
        let bin_decode: IntervalDT = bincode::deserialize(&bin).unwrap();
        assert_eq!(bin_decode, interval);
    }

    #[test]
    fn test_serde_interval_dt() {
        test_interval_dt(false, 0, 0, 0, 0, 0);
        test_interval_dt(false, 0, 0, 0, 0, 1);
        test_interval_dt(false, 1, 1, 1, 1, 1);
        test_interval_dt(false, 100000000, 0, 0, 0, 0);
        test_interval_dt(true, 0, 0, 0, 0, 1);
        test_interval_dt(true, 1, 1, 1, 1, 1);
        test_interval_dt(true, 100000000, 0, 0, 0, 0);
    }
}
