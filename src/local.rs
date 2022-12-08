use libc::{time_t, tm};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone)]
pub struct Local {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
    usec: u32,
}

impl Local {
    #[inline]
    pub fn now() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before Unix epoch");

        let res = local_time(now.as_secs() as time_t);
        Local {
            year: res.tm_year + 1900,
            month: (res.tm_mon + 1) as u32,
            day: res.tm_mday as u32,
            hour: res.tm_hour as u32,
            minute: res.tm_min as u32,
            second: res.tm_sec as u32,
            usec: now.subsec_micros(),
        }
    }

    #[inline]
    pub fn year(&self) -> i32 {
        self.year
    }

    #[inline]
    pub fn month(&self) -> u32 {
        self.month
    }

    #[inline]
    pub fn day(&self) -> u32 {
        self.day
    }

    #[inline]
    pub fn hour(&self) -> u32 {
        self.hour
    }

    #[inline]
    pub fn minute(&self) -> u32 {
        self.minute
    }

    #[inline]
    pub fn second(&self) -> u32 {
        self.second
    }

    #[inline]
    pub fn usec(&self) -> u32 {
        self.usec
    }
}

#[cfg(unix)]
fn local_time(time: time_t) -> tm {
    use libc::{c_char, localtime_r};

    let mut res = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
        tm_gmtoff: 0,
        tm_zone: std::ptr::null::<c_char>(),
    };
    unsafe {
        localtime_r(&time, &mut res);
    }
    res
}

#[cfg(windows)]
fn local_time(time: time_t) -> tm {
    use libc::localtime_s;

    let mut res = tm {
        tm_sec: 0,
        tm_min: 0,
        tm_hour: 0,
        tm_mday: 0,
        tm_mon: 0,
        tm_year: 0,
        tm_wday: 0,
        tm_yday: 0,
        tm_isdst: 0,
    };
    unsafe {
        localtime_s(&mut res, &time);
    }
    res
}
