// Copyright 2021 CoD Technologies Corp.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! sqldatetime benchmark

#[cfg(feature = "oracle")]
use crate::oracle_bench::*;
use sqldatetime::{
    Date, DateTime, Formatter, IntervalDT, IntervalYM, Round, Time, Timestamp, Trunc,
};
use stack_buf::StackVec;

use bencher::{benchmark_group, benchmark_main, black_box, Bencher};

trait FromStr {
    fn from_str(input: &str) -> Self;
}

impl FromStr for Timestamp {
    fn from_str(input: &str) -> Self {
        Timestamp::parse(input, "yyyy-mm-dd hh24:mi:ss.ff").unwrap()
    }
}

impl FromStr for Date {
    fn from_str(input: &str) -> Self {
        Date::parse(input, "yyyy-mm-dd").unwrap()
    }
}

impl FromStr for Time {
    fn from_str(input: &str) -> Self {
        Time::parse(input, "hh24:mi:ss.ff").unwrap()
    }
}

impl FromStr for IntervalDT {
    fn from_str(input: &str) -> Self {
        IntervalDT::parse(input, "dd hh24:mi:ss.ff").unwrap()
    }
}

impl FromStr for IntervalYM {
    fn from_str(input: &str) -> Self {
        IntervalYM::parse(input, "yy-mm").unwrap()
    }
}

fn timestamp_parse_format(bench: &mut Bencher) {
    bench.iter(|| Formatter::try_new(black_box("yyyy-mm-dd hh24:mi:ss.ff")).unwrap());
}

fn timestamp_parse(bench: &mut Bencher) {
    let fmt = Formatter::try_new(black_box("yyyy-mm-dd hh24:mi:ss.ff")).unwrap();
    bench.iter(|| {
        let _n = fmt
            .parse::<&str, Timestamp>(black_box("2000-1-1 10:10:10.123456"))
            .unwrap();
    })
}

fn timestamp_parse_format_without_date(bench: &mut Bencher) {
    bench.iter(|| Formatter::try_new(black_box("hh24:mi:ss.ff")).unwrap());
}

fn timestamp_parse_without_date(bench: &mut Bencher) {
    let fmt = Formatter::try_new(black_box("hh24:mi:ss.ff")).unwrap();
    bench.iter(|| {
        let _n = fmt
            .parse::<&str, Timestamp>(black_box("10:10:10.123456"))
            .unwrap();
    })
}

fn timestamp_format(bench: &mut Bencher) {
    let ts = Timestamp::from_str("2021-8-16 12:12:34.234566");
    let fmt = Formatter::try_new("yyyy-mm-dd hh24:mi:ss.ff").unwrap();
    bench.iter(|| {
        let mut s = StackVec::<u8, 100>::new();
        fmt.format(black_box(ts), &mut s).unwrap();
    })
}

fn timestamp_format_dow(bench: &mut Bencher) {
    let ts = Timestamp::from_str("2021-8-16 12:12:34.234566");
    let fmt = Formatter::try_new("yyyy-mm-dd hh24:mi:ss.ff day").unwrap();
    bench.iter(|| {
        let mut s = StackVec::<u8, 100>::new();
        fmt.format(black_box(ts), &mut s).unwrap();
    })
}

fn timestamp_sub_interval_dt_100_times(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");
    let ds = IntervalDT::from_str("12345 10:12:34.23456");
    bench.iter(|| {
        for _ in 0..100 {
            let x = ts.sub_interval_dt(ds).unwrap();
            black_box(x);
        }
    })
}

fn timestamp_sub_interval_ym(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");
    let ym = IntervalYM::from_str("123-11");
    bench.iter(|| {
        let _ = black_box(black_box(ts).sub_interval_ym(black_box(ym)).unwrap());
    })
}

fn timestamp_sub_time_100_times(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");
    let tm = Time::from_str("12:12:12.123456");
    bench.iter(|| {
        for _ in 0..100 {
            let _ = black_box(black_box(ts).sub_time(black_box(tm)).unwrap());
        }
    })
}

fn timestamp_sub_days(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");
    bench.iter(|| {
        let _ = black_box(black_box(ts).sub_days(black_box(92832.1273468)).unwrap());
    })
}

fn timestamp_extract_100_times(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");

    bench.iter(|| {
        for _ in 0..100 {
            let _ = black_box(black_box(ts).extract());
        }
    })
}

fn timestamp_extract_year(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");

    bench.iter(|| {
        let _ = black_box(black_box(ts).year());
    })
}

fn timestamp_extract_month(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");

    bench.iter(|| {
        let _ = black_box(black_box(ts).month());
    })
}

fn timestamp_extract_day(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");

    bench.iter(|| {
        let _ = black_box(black_box(ts).day());
    })
}

fn timestamp_extract_hour(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");

    bench.iter(|| {
        let _ = black_box(black_box(ts).hour());
    })
}

fn timestamp_extract_minute(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");

    bench.iter(|| {
        let _ = black_box(black_box(ts).minute());
    })
}

fn timestamp_extract_second(bench: &mut Bencher) {
    let ts = Timestamp::from_str("5000-6-6 12:45:22.123456");

    bench.iter(|| {
        let _ = black_box(black_box(ts).second());
    })
}

fn timestamp_sub_timestamp_100_times(bench: &mut Bencher) {
    let ts1 = Timestamp::from_str("5000-6-6 12:45:22.123456");
    let ts2 = Timestamp::from_str("9999-6-6 23:45:22.189039");
    bench.iter(|| {
        for _ in 0..100 {
            let _ = black_box(black_box(ts1).sub_timestamp(black_box(ts2)));
        }
    })
}

fn timestamp_last_day_of_month(bench: &mut Bencher) {
    let ts = Timestamp::from_str("2021-12-28 9:57:22.123456");
    bench.iter(|| {
        let _ = black_box(black_box(ts).last_day_of_month());
    })
}

fn date_parse_format(bench: &mut Bencher) {
    bench.iter(|| Formatter::try_new(black_box("yyyy-mm-dd")).unwrap());
}

fn date_parse(bench: &mut Bencher) {
    let fmt = Formatter::try_new(black_box("yyyy-mm-dd")).unwrap();
    bench.iter(|| {
        let _n = fmt.parse::<&str, Date>(black_box("2000-1-1")).unwrap();
    })
}

fn date_format(bench: &mut Bencher) {
    let ts = Date::from_str("2021-8-16");
    let fmt = Formatter::try_new("yyyy-mm-dd").unwrap();
    bench.iter(|| {
        let mut s = StackVec::<u8, 100>::new();
        fmt.format(black_box(ts), &mut s).unwrap();
    })
}

fn date_sub_days_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5000-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let _ = black_box(black_box(dt).sub_days(black_box(92832)).unwrap());
        }
    })
}

fn date_add_months(bench: &mut Bencher) {
    let dt = Date::from_str("5000-6-6");
    bench.iter(|| {
        let _ = black_box(black_box(dt).add_months(black_box(683)).unwrap());
    })
}

fn date_add_months2(bench: &mut Bencher) {
    let dt = Date::from_str("5000-6-6");
    bench.iter(|| {
        let _ = black_box(black_box(dt).add_months2(black_box(683)).unwrap());
    })
}

fn date_sub_interval_ym(bench: &mut Bencher) {
    let dt = Date::from_str("5000-6-6");
    let ym = IntervalYM::from_str("123-11");
    bench.iter(|| {
        let _ = black_box(black_box(dt).sub_interval_ym(black_box(ym)).unwrap());
    })
}

fn date_sub_interval_dt_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5000-6-6");
    let ds = IntervalDT::from_str("123 11:12:34.2345");
    bench.iter(|| {
        for _ in 0..100 {
            let _ = black_box(black_box(dt).sub_interval_dt(black_box(ds)).unwrap());
        }
    })
}

fn date_sub_date_100_times(bench: &mut Bencher) {
    let dt1 = Date::from_str("2903-4-5");
    let dt2 = Date::from_str("1919-1-1");
    bench.iter(|| {
        for _ in 0..100 {
            let _ = black_box(black_box(dt1).sub_date(black_box(dt2)));
        }
    })
}

fn date_add_time_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("2903-4-5");
    let tm = Time::from_str("10:12:13.2345");
    bench.iter(|| {
        for _ in 0..100 {
            let _ = black_box(black_box(dt).add_time(black_box(tm)));
        }
    })
}

fn date_extract(bench: &mut Bencher) {
    let dt = Date::from_str("2903-4-5");
    bench.iter(|| {
        let _ = black_box(black_box(dt).extract());
    })
}

fn date_dow(bench: &mut Bencher) {
    let dt = Date::from_str("2903-4-5");
    bench.iter(|| {
        let _ = black_box(black_box(dt).day_of_week());
    })
}

pub fn date_trunc_century_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_century().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_year_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_year().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_iso_year_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_iso_year().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_quarter_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_quarter().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_month_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_month().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_week_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_week().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_iso_week_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_iso_week().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_month_start_week_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_month_start_week().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_day_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_day().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_sunday_start_week_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_sunday_start_week().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_hour_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_hour().unwrap();
            black_box(x);
        }
    })
}

pub fn date_trunc_minute_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.trunc_minute().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_century_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_century().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_year_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_year().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_iso_year_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_iso_year().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_quarter_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_quarter().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_month_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_month().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_week_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_week().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_iso_week_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_iso_week().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_month_start_week_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_month_start_week().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_day_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_day().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_sunday_start_week_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_sunday_start_week().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_hour_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_hour().unwrap();
            black_box(x);
        }
    })
}

pub fn date_round_minute_100_times(bench: &mut Bencher) {
    let dt = Date::from_str("5555-6-6");
    bench.iter(|| {
        for _ in 0..100 {
            let x = dt.round_minute().unwrap();
            black_box(x);
        }
    })
}

fn date_last_day_of_month(bench: &mut Bencher) {
    let dt = Date::from_str("2021-12-28");
    bench.iter(|| {
        let _ = black_box(black_box(dt).last_day_of_month());
    })
}

fn time_parse_format(bench: &mut Bencher) {
    bench.iter(|| {
        Formatter::try_new(black_box("hh24:mi:ss.ff")).unwrap();
    })
}

fn time_parse(bench: &mut Bencher) {
    let fmt = Formatter::try_new(black_box("hh24:mi:ss.ff")).unwrap();
    bench.iter(|| {
        let _n = fmt
            .parse::<&str, Time>(black_box("10:10:10.123456"))
            .unwrap();
    })
}

fn time_format(bench: &mut Bencher) {
    let tm = Time::from_str("12:12:34.234566");
    let fmt = Formatter::try_new("hh24:mi:ss.ff").unwrap();
    bench.iter(|| {
        let mut s = StackVec::<u8, 100>::new();
        fmt.format(black_box(tm), &mut s).unwrap();
    })
}

fn time_sub_interval_dt(bench: &mut Bencher) {
    let tm = Time::from_str("10:10:10.123456");
    let ds = IntervalDT::from_str("12345 10:10:10.12344");
    bench.iter(|| {
        let _ = black_box(black_box(tm).sub_interval_dt(black_box(ds)));
    })
}

fn time_mul_f64(bench: &mut Bencher) {
    let tm = Time::from_str("10:10:10.123456");
    bench.iter(|| {
        let _ = black_box(black_box(tm).mul_f64(black_box(64543.2345)));
    })
}

fn time_div_f64(bench: &mut Bencher) {
    let tm = Time::from_str("10:10:10.123456");
    bench.iter(|| {
        let _ = black_box(black_box(tm).div_f64(black_box(64543.2345)));
    })
}

fn interval_dt_parse_format(bench: &mut Bencher) {
    bench.iter(|| {
        Formatter::try_new(black_box("DD hh24:mi:ss.ff")).unwrap();
    })
}

fn interval_dt_parse(bench: &mut Bencher) {
    let fmt = Formatter::try_new(black_box("DD hh24:mi:ss.ff")).unwrap();
    bench.iter(|| {
        let _n = fmt
            .parse::<&str, IntervalDT>(black_box("12345 10:10:10.123456"))
            .unwrap();
    })
}

fn interval_dt_format(bench: &mut Bencher) {
    let ds = IntervalDT::from_str("12345 10:10:10.123456");
    let fmt = Formatter::try_new("dd hh24:mi:ss.ff").unwrap();
    bench.iter(|| {
        let mut s = StackVec::<u8, 100>::new();
        fmt.format(black_box(ds), &mut s).unwrap();
    })
}

fn interval_dt_sub_time(bench: &mut Bencher) {
    let tm = Time::from_str("10:10:10.123456");
    let ds = IntervalDT::from_str("12345 10:10:10.12344");
    bench.iter(|| {
        let _ = black_box(black_box(ds).sub_time(black_box(tm)));
    })
}

fn interval_dt_sub_interval_dt(bench: &mut Bencher) {
    let ds1 = IntervalDT::from_str("12345 10:10:10.12344");
    let ds2 = IntervalDT::from_str("84757 13:15:17.88874");
    bench.iter(|| {
        let _ = black_box(black_box(ds1).sub_interval_dt(black_box(ds2)));
    })
}

fn interval_dt_mul_f64(bench: &mut Bencher) {
    let ds = IntervalDT::from_str("1928 10:10:10.123456");
    bench.iter(|| {
        let _ = black_box(black_box(ds).mul_f64(black_box(64543.2345)));
    })
}

fn interval_dt_div_f64(bench: &mut Bencher) {
    let ds = IntervalDT::from_str("1928 10:10:10.123456");
    bench.iter(|| {
        let _ = black_box(black_box(ds).div_f64(black_box(64543.2345)));
    })
}

fn interval_ym_parse_format(bench: &mut Bencher) {
    bench.iter(|| {
        Formatter::try_new(black_box("YY-MM")).unwrap();
    })
}

fn interval_ym_parse(bench: &mut Bencher) {
    let fmt = Formatter::try_new(black_box("YY-MM")).unwrap();
    bench.iter(|| {
        let _n = fmt
            .parse::<&str, IntervalYM>(black_box("12345-10"))
            .unwrap();
    })
}

fn interval_ym_format(bench: &mut Bencher) {
    let ym = IntervalYM::from_str("12345-10");
    let fmt = Formatter::try_new("yy-mm").unwrap();
    bench.iter(|| {
        let mut s = StackVec::<u8, 100>::new();
        fmt.format(black_box(ym), &mut s).unwrap();
    })
}

fn interval_ym_mul_f64(bench: &mut Bencher) {
    let ym = IntervalYM::from_str("1928-10");
    bench.iter(|| {
        let _ = black_box(black_box(ym).mul_f64(black_box(64543.2345)));
    })
}

fn interval_ym_div_f64(bench: &mut Bencher) {
    let ym = IntervalYM::from_str("1928-10");
    bench.iter(|| {
        let _ = black_box(black_box(ym).div_f64(black_box(64543.2345)));
    })
}

fn interval_ym_sub_interval_ym(bench: &mut Bencher) {
    let ym1 = IntervalYM::from_str("12345-10");
    let ym2 = IntervalYM::from_str("84757-11");
    bench.iter(|| {
        let _ = black_box(black_box(ym1).sub_interval_ym(black_box(ym2)));
    })
}

#[cfg(feature = "oracle")]
mod oracle_bench {

    use super::*;
    use sqldatetime::{OracleDate, Round, Trunc};

    impl FromStr for OracleDate {
        fn from_str(input: &str) -> Self {
            OracleDate::parse(input, "yyyy-mm-dd hh24:mi:ss").unwrap()
        }
    }

    pub fn oracle_date_parse_format(bench: &mut Bencher) {
        bench.iter(|| Formatter::try_new(black_box("yyyy-mm-dd hh24:mi:ss")).unwrap());
    }

    pub fn oracle_date_parse(bench: &mut Bencher) {
        let fmt = black_box(Formatter::try_new(black_box("yyyy-mm-dd hh24:mi:ss")).unwrap());
        bench.iter(|| {
            let _n = fmt
                .parse::<&str, OracleDate>(black_box("1919-1-1 10:10:10"))
                .unwrap();
        })
    }

    pub fn oracle_date_parse_without_date(bench: &mut Bencher) {
        let fmt = black_box(Formatter::try_new(black_box("hh24:mi:ss")).unwrap());
        bench.iter(|| {
            let _n = fmt
                .parse::<&str, OracleDate>(black_box("10:10:10"))
                .unwrap();
        })
    }

    pub fn oracle_date_format(bench: &mut Bencher) {
        let od = OracleDate::from_str("2020-1-1 12:12:34");
        let fmt = black_box(Formatter::try_new(black_box("yyyy-mm-dd hh24:mi:ss")).unwrap());
        bench.iter(|| {
            let mut s = StackVec::<u8, 100>::new();
            fmt.format(black_box(od), &mut s).unwrap();
        })
    }

    pub fn oracle_date_format_dow(bench: &mut Bencher) {
        let od = OracleDate::from_str("2021-8-18 12:12:34");
        let fmt = black_box(Formatter::try_new(black_box("yyyy-mm-dd hh24:mi:ss day")).unwrap());
        bench.iter(|| {
            let mut s = StackVec::<u8, 100>::new();
            fmt.format(black_box(od), &mut s).unwrap();
        })
    }

    pub fn oracle_date_sub_interval_ym(bench: &mut Bencher) {
        let od = black_box(OracleDate::from_str("5000-6-6 12:45:22"));
        let ym = IntervalYM::from_str("123-11");
        bench.iter(|| {
            let _ = od.sub_interval_ym(ym).unwrap();
        })
    }

    pub fn oracle_date_sub_oracle_date_100_times(bench: &mut Bencher) {
        let od1 = OracleDate::from_str("5000-6-6 12:45:22");
        let od2 = OracleDate::from_str("9999-6-6 23:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let _x = black_box(od1).sub_date(black_box(od2));
            }
        })
    }

    pub fn oracle_date_sub_interval_dt_100_times(bench: &mut Bencher) {
        let od = OracleDate::from_str("5000-6-6 12:45:22");
        let ds = IntervalDT::from_str("12345 10:12:34.23456");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.sub_interval_dt(ds).unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_century_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_century().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_year_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_year().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_iso_year_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_iso_year().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_quarter_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_quarter().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_month_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_month().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_week_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_week().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_iso_week_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_iso_week().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_month_start_week_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_month_start_week().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_day_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_day().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_sunday_start_week_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_sunday_start_week().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_hour_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_hour().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_trunc_minute_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.trunc_minute().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_century_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_century().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_year_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_year().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_iso_year_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_iso_year().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_quarter_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-16 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_quarter().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_month_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_month().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_week_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_week().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_iso_week_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_iso_week().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_month_start_week_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_month_start_week().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_day_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_day().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_sunday_start_week_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_sunday_start_week().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_hour_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_hour().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_round_minute_100_times(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od.round_minute().unwrap();
                black_box(x);
            }
        })
    }

    pub fn oracle_date_last_day_of_month(bench: &mut Bencher) {
        let od: OracleDate = OracleDate::from_str("2021-12-28 9:57:22");
        bench.iter(|| {
            let _ = black_box(black_box(od).last_day_of_month());
        })
    }

    pub fn oracle_date_months_between_100_times(bench: &mut Bencher) {
        let od1: OracleDate = OracleDate::from_str("5555-6-6 12:45:22");
        let od2: OracleDate = OracleDate::from_str("2021-12-28 9:57:22");
        bench.iter(|| {
            for _ in 0..100 {
                let x = od1.months_between(od2);
                black_box(x);
            }
        })
    }
}

#[cfg(feature = "oracle")]
benchmark_group!(
    oracle_datetime_benches,
    oracle_date_parse_format,
    oracle_date_parse,
    oracle_date_parse_without_date,
    oracle_date_format,
    oracle_date_format_dow,
    oracle_date_sub_interval_dt_100_times,
    oracle_date_sub_interval_ym,
    oracle_date_sub_oracle_date_100_times,
    oracle_date_trunc_century_100_times,
    oracle_date_trunc_year_100_times,
    oracle_date_trunc_iso_year_100_times,
    oracle_date_trunc_quarter_100_times,
    oracle_date_trunc_month_100_times,
    oracle_date_trunc_week_100_times,
    oracle_date_trunc_iso_week_100_times,
    oracle_date_trunc_month_start_week_100_times,
    oracle_date_trunc_day_100_times,
    oracle_date_trunc_sunday_start_week_100_times,
    oracle_date_trunc_hour_100_times,
    oracle_date_trunc_minute_100_times,
    oracle_date_round_century_100_times,
    oracle_date_round_year_100_times,
    oracle_date_round_iso_year_100_times,
    oracle_date_round_quarter_100_times,
    oracle_date_round_month_100_times,
    oracle_date_round_week_100_times,
    oracle_date_round_iso_week_100_times,
    oracle_date_round_month_start_week_100_times,
    oracle_date_round_day_100_times,
    oracle_date_round_sunday_start_week_100_times,
    oracle_date_round_hour_100_times,
    oracle_date_round_minute_100_times,
    oracle_date_last_day_of_month,
    oracle_date_months_between_100_times
);

benchmark_group!(
    datetime_benches,
    timestamp_parse_format,
    timestamp_parse,
    timestamp_parse_format_without_date,
    timestamp_parse_without_date,
    timestamp_format,
    timestamp_format_dow,
    timestamp_sub_interval_dt_100_times,
    timestamp_sub_interval_ym,
    timestamp_sub_time_100_times,
    timestamp_sub_days,
    timestamp_sub_timestamp_100_times,
    timestamp_extract_100_times,
    timestamp_extract_year,
    timestamp_extract_month,
    timestamp_extract_day,
    timestamp_extract_hour,
    timestamp_extract_minute,
    timestamp_extract_second,
    timestamp_last_day_of_month,
    date_parse_format,
    date_parse,
    date_format,
    date_sub_days_100_times,
    date_add_months,
    date_add_months2,
    date_sub_interval_ym,
    date_sub_interval_dt_100_times,
    date_sub_date_100_times,
    date_extract,
    date_dow,
    date_add_time_100_times,
    date_trunc_century_100_times,
    date_trunc_year_100_times,
    date_trunc_iso_year_100_times,
    date_trunc_quarter_100_times,
    date_trunc_month_100_times,
    date_trunc_week_100_times,
    date_trunc_iso_week_100_times,
    date_trunc_month_start_week_100_times,
    date_trunc_day_100_times,
    date_trunc_sunday_start_week_100_times,
    date_trunc_hour_100_times,
    date_trunc_minute_100_times,
    date_round_century_100_times,
    date_round_year_100_times,
    date_round_iso_year_100_times,
    date_round_quarter_100_times,
    date_round_month_100_times,
    date_round_week_100_times,
    date_round_iso_week_100_times,
    date_round_month_start_week_100_times,
    date_round_day_100_times,
    date_round_sunday_start_week_100_times,
    date_round_hour_100_times,
    date_round_minute_100_times,
    date_last_day_of_month,
    time_parse_format,
    time_parse,
    time_format,
    time_sub_interval_dt,
    time_mul_f64,
    time_div_f64,
    interval_dt_parse_format,
    interval_dt_parse,
    interval_dt_format,
    interval_dt_mul_f64,
    interval_dt_div_f64,
    interval_dt_sub_time,
    interval_dt_sub_interval_dt,
    interval_ym_parse_format,
    interval_ym_parse,
    interval_ym_format,
    interval_ym_mul_f64,
    interval_ym_div_f64,
    interval_ym_sub_interval_ym,
);

#[cfg(feature = "oracle")]
benchmark_main!(datetime_benches, oracle_datetime_benches);

#[cfg(not(feature = "oracle"))]
benchmark_main!(datetime_benches);
