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

    #[error("Failed to read configuration file")]
    ConfigReadError,
    #[error("Failed to parse configuration file")]
    ConfigParseError,

    #[error("Conflict reservation")]
    ConflictReservation(ReservationConflictInfo),
    #[error("Invalid user id: {0}")]
    InvalidUserId(String),
    #[error("Invalid resource id: {0}")]
    InvalidResourceId(String),
    #[error("Invalid reservation id: {0}")]
    InvalidReservationId(i64),
    #[error("Invalid header (expected {0}, found {1})")]
    InvalidHeader(String, String),
    #[error("Invalid page size: {0}")]
    InvalidPageSize(i64),
    #[error("Invalid cursor: {0}")]
    InvalidCursor(i64),
    #[error("Invalid reservation status: {0}")]
    InvalidStatus(i32),
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

impl From<crate::Error> for tonic::Status {
    fn from(e: crate::Error) -> Self {
        match e {
            Error::DbError(_) | Error::ConfigReadError | Error::ConfigParseError => {
                tonic::Status::internal(e.to_string())
            }
            Error::InvalidTime
            | Error::InvalidReservationId(_)
            | Error::InvalidUserId(_)
            | Error::InvalidResourceId(_)
            | Error::InvalidPageSize(_)
            | Error::InvalidCursor(_)
            | Error::InvalidStatus(_) => tonic::Status::invalid_argument(e.to_string()),
            Error::ConflictReservation(info) => {
                tonic::Status::failed_precondition(format!("Conflict reservation: {:?}", info))
            }
            Error::RowNotFound => {
                tonic::Status::not_found("No reservation found by the given condition")
            }
            Error::InvalidHeader(expected, found) => tonic::Status::invalid_argument(format!(
                "Invalid header (expected {:?}, found {:?})",
                expected, found
            )),
            Error::Unknown => tonic::Status::unknown("unknown error"),
        }
    }
}
