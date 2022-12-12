use chrono::{DateTime, NaiveDateTime, Utc};
use prost_types::Timestamp;

pub fn convert_to_utc_time(ts: &Timestamp) -> DateTime<Utc> {
    DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(ts.seconds, ts.nanos as _)
            .expect("invalid or out-of-range datetime"),
        Utc,
    )
}

pub fn convert_to_timestamp(dt: &DateTime<Utc>) -> Timestamp {
    Timestamp {
        seconds: dt.timestamp(),
        nanos: dt.timestamp_subsec_nanos() as _,
    }
}

pub fn str_to_option(s: &str) -> Option<&str> {
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}
