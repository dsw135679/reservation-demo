mod error;
mod pb;
mod types;
mod utils;

pub use error::{Error, ReservationConflict, ReservationConflictInfo, ReservationWindow};
pub use pb::*;
pub use types::*;
pub use utils::*;

pub trait Validate {
    fn validate(&self) -> Result<(), Error>;
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
