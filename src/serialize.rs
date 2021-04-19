//! Impl the `serde::Serialize` and `serde::Deserialize` traits.

use crate::{Date, Formatter};
use once_cell::sync::Lazy;
use serde_crate::de::Visitor;
use serde_crate::{de, ser, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

static DATE_FORMATTER: Lazy<Formatter> = Lazy::new(|| Formatter::try_new("YYYY-MM-DD").unwrap());

impl Serialize for Date {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            let mut buf = String::new();
            DATE_FORMATTER
                .format_date(*self, &mut buf)
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
                DATE_FORMATTER.parse_date(v).map_err(de::Error::custom)
            }
        }

        if deserializer.is_human_readable() {
            deserializer.deserialize_str(DateVisitor)
        } else {
            deserializer.deserialize_i32(DateVisitor)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_date() {
        let date = Date::try_from_ymd(2021, 4, 16).unwrap();
        let date_json = serde_json::to_string(&date).unwrap();
        println!("json: {}", date_json);
        let json_decode: Date = serde_json::from_str(&date_json).unwrap();
        assert_eq!(json_decode, date);

        let bin = bincode::serialize(&date).unwrap();
        println!("bin: {:?}", bin);
        let bin_decode: Date = bincode::deserialize(&bin).unwrap();
        assert_eq!(bin_decode, date);
    }
}
