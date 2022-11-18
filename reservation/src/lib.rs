mod error;

pub use error::ReservationError;

pub type ReservationId = String;
pub type UserId = String;
pub type ResourceId = String;

pub trait Rsvp {
    /// make a reservation
    fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, ReservationError>;
    /// change reservation status (if current status is pending, change it to confirmed)
    fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError>;
    /// update note
    fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, ReservationError>;
    /// delete reservation
    fn delete(&self, id: ReservationId) -> Result<(), ReservationError>;
    /// get reservation by id
    fn get(&self, id: ReservationId) -> Result<abi::Reservation, ReservationError>;
    /// query reservations
    fn query(&self, query: abi::QueryRequest) -> Result<Vec<abi::Reservation>, ReservationError>;
}
