use std::ops::Bound;

use chrono::{DateTime, FixedOffset, Utc};
use sqlx::{
    postgres::{types::PgRange, PgRow},
    FromRow, Row,
};

use crate::{
    convert_to_timestamp, get_timestamp, pager::Id, validate_range, Error, Reservation,
    ReservationStatus, RsvpStatus, Validator,
};

impl Reservation {
    pub fn new_pending(
        uid: impl Into<String>,
        rid: impl Into<String>,
        start: DateTime<FixedOffset>,
        end: DateTime<FixedOffset>,
        note: impl Into<String>,
    ) -> Self {
        Self {
            id: 0,
            user_id: uid.into(),
            status: ReservationStatus::Pending as i32,
            resource_id: rid.into(),
            start: Some(convert_to_timestamp(&start.with_timezone(&Utc))),
            end: Some(convert_to_timestamp(&end.with_timezone(&Utc))),
            note: note.into(),
        }
    }

    pub fn get_timespan(&self) -> PgRange<DateTime<Utc>> {
        get_timestamp(self.start.as_ref(), self.end.as_ref())
    }
}

impl Id for Reservation {
    fn id(&self) -> i64 {
        self.id
    }
}

impl Validator for Reservation {
    fn validate(&self) -> Result<(), Error> {
        if self.user_id.is_empty() {
            return Err(Error::InvalidUserId(self.user_id.clone()));
        }

        if self.resource_id.is_empty() {
            return Err(Error::InvalidResourceId(self.resource_id.clone()));
        }

        validate_range(self.start.as_ref(), self.end.as_ref())?;

        Ok(())
    }
}

impl FromRow<'_, PgRow> for Reservation {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let range: PgRange<DateTime<Utc>> = row.get("timespan");
        let range: NaiveRange<DateTime<Utc>> = range.into();
        assert!(range.start.is_some());
        assert!(range.end.is_some());
        let start = range.start.unwrap();
        let end = range.end.unwrap();
        let status: RsvpStatus = row.get("status");
        Ok(Self {
            id: row.get("id"),
            user_id: row.get("user_id"),
            resource_id: row.get("resource_id"),
            start: Some(convert_to_timestamp(&start)),
            end: Some(convert_to_timestamp(&end)),
            note: row.get("note"),
            status: ReservationStatus::from(status) as i32,
        })
    }
}

struct NaiveRange<T> {
    start: Option<T>,
    end: Option<T>,
}

impl<T> From<PgRange<T>> for NaiveRange<T> {
    fn from(range: PgRange<T>) -> Self {
        let f = |b: Bound<T>| match b {
            Bound::Included(v) => Some(v),
            Bound::Excluded(v) => Some(v),
            Bound::Unbounded => None,
        };

        let start = f(range.start);
        let end = f(range.end);

        Self { start, end }
    }
}
