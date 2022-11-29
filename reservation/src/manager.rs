use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, types::Uuid, PgPool, Row};

use crate::{ReservationId, ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        rsvp.validate()?;

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan().into();

        let status = abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);
        // generate a insert sql for the reservation
        let id:Uuid= sqlx::query(
          "INSERT INTO rsvp.reservations (user_id,resource_id,timespan,note,status) VALUES ($1,$2,$3,$4,$5::rsvp.reservation_status) RETURNING id")
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?
        .get(0);

        rsvp.id = id.to_string();

        Ok(rsvp)
    }

    /// change reservation status (if current status is pending, change it to confirmed)
    async fn change_status(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        // 将 id 转换为 uuid
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        // if current status is pending,change it to confirmed,otherwise d nothing.
        let rsvp:abi::Reservation=sqlx::query_as(
          "UPDATE rsvp.reservations SET status ='confirmed' WHERE id= $1 AND status= 'pending' RETURNING *"
        ).bind(id).fetch_one(&self.pool).await?;

        Ok(rsvp)
    }
    /// update note
    async fn update_note(
        &self,
        id: ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        // update the note of the reservation
        let id = Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp =
            sqlx::query_as("UPDATE rsvp.reservations SET note = $1 WHERE id =$2 RETURNING *")
                .bind(note)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(rsvp)
    }
    /// delete reservation
    async fn delete(&self, id: ReservationId) -> Result<(), abi::Error> {
        let uuid =
            Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        sqlx::query("DELETE FROM rsvp.reservations WHERE id= $1")
            .bind(uuid)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    /// get reservation by id
    async fn get(&self, id: ReservationId) -> Result<abi::Reservation, abi::Error> {
        let uuid =
            Uuid::parse_str(&id).map_err(|_| abi::Error::InvalidReservationId(id.clone()))?;
        let rsvp = sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id= $1")
            .bind(uuid)
            .fetch_one(&self.pool)
            .await?;

        Ok(rsvp)
    }
    /// query reservations
    async fn query(
        &self,
        _query: abi::ReservationRequest,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        todo!()
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use abi::{Reservation, ReservationConflict, ReservationConflictInfo, ReservationWindow};

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let (rsvp, _manager) = make_chalanzi_reservation(migrate_pool.clone()).await;
        assert!(!rsvp.id.is_empty());
    }

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn reserve_conflict_reservation_should_reject() {
        let (_rsvp, manager) = make_chalanzi_reservation(migrate_pool.clone()).await;
        let rsvp2 = abi::Reservation::new_pending(
            "wanerId",
            "ocean-view-room-713",
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "hello.",
        );

        let err = manager.reserve(rsvp2).await.unwrap_err();

        let info = ReservationConflictInfo::Parsed(ReservationConflict {
            new: ReservationWindow {
                rid: "ocean-view-room-713".to_string(),
                start: "2022-12-26T15:00:00-0700".parse().unwrap(),
                end: "2022-12-30T12:00:00-0700".parse().unwrap(),
            },
            old: ReservationWindow {
                rid: "ocean-view-room-713".to_string(),
                start: "2022-12-25T15:00:00-0700".parse().unwrap(),
                end: "2022-12-28T12:00:00-0700".parse().unwrap(),
            },
        });

        assert_eq!(err, abi::Error::ConflictReservation(info));
    }

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn reserve_should_reject_if_id_is_not_empty() {
        let manager = ReservationManager::new(migrate_pool.clone());
        let rsvp1 = abi::Reservation::new_pending(
            "chalanziId",
            "ocean-view-room-713",
            "2022-12-25T15:00:00-0700".parse().unwrap(),
            "2022-12-28T12:00:00-0700".parse().unwrap(),
            "hello.",
        );
        let rsvp2 = abi::Reservation::new_pending(
            "wanerId",
            "ocean-view-room-713",
            "2022-12-26T15:00:00-0700".parse().unwrap(),
            "2022-12-30T12:00:00-0700".parse().unwrap(),
            "hello.",
        );
        let _rsvp1 = manager.reserve(rsvp1).await.unwrap();
        let err = manager.reserve(rsvp2).await.unwrap_err();
        println!("{:?}", err);
        if let abi::Error::ConflictReservation(abi::ReservationConflictInfo::Parsed(info)) = err {
            assert_eq!(info.old.rid, "ocean-view-room-713");
            assert_eq!(info.old.start.to_rfc3339(), "2022-12-25T22:00:00+00:00");
            assert_eq!(info.old.end.to_rfc3339(), "2022-12-28T19:00:00+00:00");
        } else {
            panic!("expect conflict reservation error");
        }
    }

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn reserve_change_status_should_work() {
        let (rsvp, manager) = make_waner_reservation(migrate_pool.clone()).await;
        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        assert_eq!(rsvp.status, abi::ReservationStatus::Confirmed as i32)
    }

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn reserve_change_not_pending_should_do_nothing() {
        let (rsvp, manager) = make_waner_reservation(migrate_pool.clone()).await;
        let rsvp = manager.change_status(rsvp.id).await.unwrap();

        // change status again should nothing
        let ret = manager.change_status(rsvp.id).await.unwrap_err();
        assert_eq!(ret, abi::Error::RowNotFound);
    }

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn update_note_should_work() {
        let (rsvp, manager) = make_waner_reservation(migrate_pool.clone()).await;
        let rsvp = manager
            .update_note(rsvp.id, "hello world".to_string())
            .await
            .unwrap();
        assert_eq!(rsvp.note, "hello world".to_string())
    }

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn get_reservation_should_work() {
        let (rsvp, manager) = make_waner_reservation(migrate_pool.clone()).await;
        let rsvp1 = manager.get(rsvp.id.clone()).await.unwrap();
        assert_eq!(rsvp1, rsvp)
    }
    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn delete_reservation_should_work() {
        let (rsvp, manager) = make_waner_reservation(migrate_pool.clone()).await;
        manager.delete(rsvp.id.clone()).await.unwrap();
        let rsvp1 = manager.get(rsvp.id.clone()).await.unwrap_err();
        assert_eq!(rsvp1, abi::Error::RowNotFound);
    }

    async fn make_chalanzi_reservation(pool: PgPool) -> (Reservation, ReservationManager) {
        make_reservation(
            pool,
            "chalanziId",
            "ocean-view-room-713",
            "2022-12-25T15:00:00-0700",
            "2022-12-28T12:00:00-0700",
            "我将与下午3点到达，请帮忙预约",
        )
        .await
    }

    async fn make_waner_reservation(pool: PgPool) -> (Reservation, ReservationManager) {
        make_reservation(
            pool,
            "wanerId",
            "ixia-test-1",
            "2023-01-25T15:00:00-0700",
            "2023-02-25T12:00:00-0700",
            "我需要预订xyz一个月",
        )
        .await
    }

    async fn make_reservation(
        pool: PgPool,
        uid: &str,
        rid: &str,
        start: &str,
        end: &str,
        note: &str,
    ) -> (Reservation, ReservationManager) {
        let manager = ReservationManager::new(pool.clone());
        let rsvp = abi::Reservation::new_pending(
            uid,
            rid,
            start.parse().unwrap(),
            end.parse().unwrap(),
            note,
        );

        (manager.reserve(rsvp).await.unwrap(), manager)
    }
}
