mod conflict;
pub use conflict::{ReservationConflict, ReservationConflictInfo, ReservationWindow};

use sqlx::postgres::PgDatabaseError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Database error")]
    DbError(sqlx::Error),
    #[error("No reservation found by the given condition")]
    RowNotFound,
    #[error("Invalid start or end time for the reservation")]
    InvalidTime,

    #[error("Conflict reservation")]
    ConflictReservation(ReservationConflictInfo),
    #[error("Invalid user id: {0}")]
    InvalidUserId(String),
    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),
    #[error("Invalid reservation id: {0}")]
    InvalidReservationId(i64),
    #[error("invalid header (expected {expected:?}, found {found:?})")]
    InvalidHeader { expected: String, found: String },
    #[error("unknown data store error")]
    Unknown,
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            // TODO this is not a good way to compare DB error.
            (Self::DbError(_), Self::DbError(_)) => true,
            (Self::InvalidTime, Self::InvalidTime) => true,
            (Self::ConflictReservation(v1), Self::ConflictReservation(v2)) => v1 == v2,
            (Self::RowNotFound, Self::RowNotFound) => true,
            (Self::InvalidReservationId(v1), Self::InvalidReservationId(v2)) => v1 == v2,
            (Self::InvalidUserId(v1), Self::InvalidUserId(v2)) => v1 == v2,
            (Self::InvalidResourceId(v1), Self::InvalidResourceId(v2)) => v1 == v2,
            (Self::Unknown, Self::Unknown) => true,
            _ => false,
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match e {
            sqlx::Error::Database(e) => {
                let err: &PgDatabaseError = e.downcast_ref();
                match (err.code(), err.schema(), err.table()) {
                    ("23P01", Some("rsvp"), Some("reservations")) => {
                        Error::ConflictReservation(err.detail().unwrap().parse().unwrap())
                    }
                    _ => Error::DbError(sqlx::Error::Database(e)),
                }
            }
            sqlx::Error::RowNotFound => Error::RowNotFound,
            _ => Error::DbError(e),
        }
    }
}
