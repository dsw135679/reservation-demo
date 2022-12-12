mod reservation;
mod reservation_filter;
mod reservation_query;
mod reservation_status;

use std::ops::Bound;

use chrono::{DateTime, Utc};
use prost_types::Timestamp;
pub use reservation::*;
pub use reservation_query::*;
pub use reservation_status::*;
use sqlx::postgres::types::PgRange;

use crate::{convert_to_utc_time, Error};

pub fn validate_range(start: Option<&Timestamp>, end: Option<&Timestamp>) -> Result<(), Error> {
    if start.is_none() || end.is_none() {
        return Err(Error::InvalidTime);
    }

    let start = start.as_ref().unwrap();
    let end = end.as_ref().unwrap();

    if start.seconds >= end.seconds {
        return Err(Error::InvalidTime);
    }

    Ok(())
}

pub fn get_timestamp(start: Option<&Timestamp>, end: Option<&Timestamp>) -> PgRange<DateTime<Utc>> {
    let start = convert_to_utc_time(start.as_ref().unwrap());
    let end = convert_to_utc_time(end.as_ref().unwrap());
    PgRange {
        start: Bound::Included(start),
        end: Bound::Excluded(end),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_range_should_allow_range() {
        let start = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 2,
            nanos: 0,
        };

        assert!(validate_range(Some(&start), Some(&end)).is_ok());
    }

    #[test]
    fn validate_range_should_reject_invalid_range() {
        let end = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        let start = Timestamp {
            seconds: 2,
            nanos: 0,
        };

        assert!(validate_range(Some(&start), Some(&end)).is_err());
    }

    #[test]
    fn get_timestamp_should_work_for_valid_start_end() {
        let start = Timestamp {
            seconds: 1,
            nanos: 0,
        };
        let end = Timestamp {
            seconds: 2,
            nanos: 0,
        };

        let range = get_timestamp(Some(&start), Some(&end));
        assert_eq!(range.start, Bound::Included(convert_to_utc_time(&start)));
        assert_eq!(range.end, Bound::Excluded(convert_to_utc_time(&end)));
    }
}
