mod manager;

use async_trait::async_trait;
use sqlx::PgPool;

#[derive(Debug)]
pub struct ReservationManager {
    pool: PgPool,
}

/// reservation trait
#[async_trait]
pub trait Rsvp {
    /// make a reservation
    async fn reserve(&self, rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error>;
    /// change reservation status (if current status is pending, change it to confirmed)
    async fn change_status(&self, id: abi::ReservationId) -> Result<abi::Reservation, abi::Error>;
    /// update note
    async fn update_note(
        &self,
        id: abi::ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error>;
    /// delete reservation
    async fn delete(&self, id: abi::ReservationId) -> Result<(), abi::Error>;
    /// get reservation by id
    async fn get(&self, id: abi::ReservationId) -> Result<abi::Reservation, abi::Error>;
    /// query reservations
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error>;
    /// query reservations order by id
    async fn filter(
        &self,
        query: abi::ReservationFilter,
    ) -> Result<(abi::FilterPager, Vec<abi::Reservation>), abi::Error>;
}
