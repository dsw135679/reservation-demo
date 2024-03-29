mod service;

use std::pin::Pin;

use abi::Reservation;
use futures::Stream;
use reservation::ReservationManager;
use tonic::Status;

pub use service::*;

pub struct RsvpService {
    pub manager: ReservationManager,
}

type ReservationStream = Pin<Box<dyn Stream<Item = Result<Reservation, Status>> + Send>>;
