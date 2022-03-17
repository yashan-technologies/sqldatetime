//! Utilities

use crate::error::Result;
use std::fmt;

pub trait StrExt {
    fn try_to_string(&self) -> Result<String>;
}

impl StrExt for str {
    #[inline]
    fn try_to_string(&self) -> Result<String> {
        let mut s = String::new();
        s.try_reserve(self.len())?;
        s.push_str(self);
        Ok(s)
    }
}

#[repr(transparent)]
struct StrBuf(String);

impl fmt::Write for StrBuf {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.0.try_reserve(s.len()).map_err(|_| fmt::Error)?;
        self.0.push_str(s);
        Ok(())
    }
}

#[inline]
pub fn try_format(args: fmt::Arguments<'_>) -> Result<String> {
    use fmt::Write;

    let mut output = StrBuf(String::new());
    output.write_fmt(args)?;
    Ok(output.0)
}

macro_rules! try_format {
    ($($arg:tt)*) => {{
        let res = $crate::util::try_format(format_args!($($arg)*));
        res
    }}
}
