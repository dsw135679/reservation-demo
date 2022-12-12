mod error;
mod pager;
mod pb;
mod types;
mod utils;

pub use error::{Error, ReservationConflict, ReservationConflictInfo, ReservationWindow};
pub use pb::*;
pub use types::*;
pub use utils::*;

pub type ReservationId = i64;
pub type UserId = String;
pub type ResourceId = String;

pub trait Validator {
    fn validate(&self) -> Result<(), Error>;
}

/// validate and normalize the data structure
pub trait Normalizer: Validator {
    /// caller should call normalize to make sure the data structure is ready to use
    fn normalize(&mut self) -> Result<(), Error> {
        self.validate()?;
        self.do_normalize();
        Ok(())
    }

    /// user shall implement do_normalize() to normalize the data structure
    fn do_normalize(&mut self);
}

pub trait ToSql {
    fn to_sql(&self) -> String;
}

/// database equivalent of the "reservation_status" enum
/// 数据库 reservation_status 枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "reservation_status", rename_all = "lowercase")]
pub enum RsvpStatus {
    Unknown,
    Pending,
    Confirmed,
    Blocked,
}

impl Validator for ReservationId {
    fn validate(&self) -> Result<(), Error> {
        if *self <= 0 {
            return Err(Error::InvalidReservationId(*self));
        }

        Ok(())
    }
}

// impl Validate for String {
//     fn validate(&self) -> Result<(), Error> {
//         if self.is_empty() {
//             return Err(Error::InvalidUserId(self));
//         }

//         Ok(())
//     }
// }
