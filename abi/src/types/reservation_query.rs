use chrono::{DateTime, Utc};
use sqlx::postgres::types::PgRange;

use crate::{get_timestamp, validate_range, ReservationQuery, Validate};

impl ReservationQuery {
    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        get_timestamp(self.start.as_ref(), self.end.as_ref())
    }
}

impl Validate for ReservationQuery {
    fn validate(&self) -> Result<(), crate::Error> {
        validate_range(self.start.as_ref(), self.end.as_ref())?;
        Ok(())
    }
}
