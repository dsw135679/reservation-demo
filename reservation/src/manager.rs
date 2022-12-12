use abi::{Normalizer, ToSql, Validator};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{postgres::types::PgRange, PgPool, Row};

use crate::{ReservationManager, Rsvp};

#[async_trait]
impl Rsvp for ReservationManager {
    async fn reserve(&self, mut rsvp: abi::Reservation) -> Result<abi::Reservation, abi::Error> {
        rsvp.validate()?;

        let timespan: PgRange<DateTime<Utc>> = rsvp.get_timespan();

        let status = abi::ReservationStatus::from_i32(rsvp.status)
            .unwrap_or(abi::ReservationStatus::Pending);
        // generate a insert sql for the reservation
        let id= sqlx::query(
          "INSERT INTO rsvp.reservations (user_id,resource_id,timespan,note,status) VALUES ($1,$2,$3,$4,$5::rsvp.reservation_status) RETURNING id")
        .bind(rsvp.user_id.clone())
        .bind(rsvp.resource_id.clone())
        .bind(timespan)
        .bind(rsvp.note.clone())
        .bind(status.to_string())
        .fetch_one(&self.pool)
        .await?
        .get(0);

        rsvp.id = id;

        Ok(rsvp)
    }

    /// change reservation status (if current status is pending, change it to confirmed)
    async fn change_status(&self, id: abi::ReservationId) -> Result<abi::Reservation, abi::Error> {
        id.validate()?;
        // if current status is pending,change it to confirmed,otherwise d nothing.
        let rsvp:abi::Reservation=sqlx::query_as(
          "UPDATE rsvp.reservations SET status ='confirmed' WHERE id= $1 AND status= 'pending' RETURNING *"
        ).bind(id).fetch_one(&self.pool).await?;

        Ok(rsvp)
    }
    /// update note
    async fn update_note(
        &self,
        id: abi::ReservationId,
        note: String,
    ) -> Result<abi::Reservation, abi::Error> {
        // update the note of the reservation
        id.validate()?;
        let rsvp =
            sqlx::query_as("UPDATE rsvp.reservations SET note = $1 WHERE id = $2 RETURNING *")
                .bind(note)
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(rsvp)
    }
    /// delete reservation
    async fn delete(&self, id: abi::ReservationId) -> Result<(), abi::Error> {
        id.validate()?;
        sqlx::query("DELETE FROM rsvp.reservations WHERE id= $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
    /// get reservation by id
    async fn get(&self, id: abi::ReservationId) -> Result<abi::Reservation, abi::Error> {
        id.validate()?;
        let rsvp = sqlx::query_as("SELECT * FROM rsvp.reservations WHERE id= $1")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(rsvp)
    }
    /// query reservations
    async fn query(
        &self,
        query: abi::ReservationQuery,
    ) -> Result<Vec<abi::Reservation>, abi::Error> {
        let sql = query.to_sql();
        let rsvps = sqlx::query_as(&sql).fetch_all(&self.pool).await?;
        Ok(rsvps)
    }

    async fn filter(
        &self,
        mut filter: abi::ReservationFilter,
    ) -> Result<(abi::FilterPager, Vec<abi::Reservation>), abi::Error> {
        // filter reservations by user_id,resource_id,status,and order by id
        filter.normalize()?;
        let sql = filter.to_sql();

        let rsvps: Vec<abi::Reservation> = sqlx::query_as(&sql).fetch_all(&self.pool).await?;
        let mut rsvps = rsvps.into_iter().collect();
        let pager = filter.get_pager(&mut rsvps);
        Ok((pager, rsvps.into_iter().collect()))
    }
}

impl ReservationManager {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[cfg(test)]
mod tests {
    use abi::{
        Reservation, ReservationConflict, ReservationConflictInfo, ReservationFilterBuilder,
        ReservationQueryBuilder, ReservationWindow,
    };
    use prost_types::Timestamp;

    use super::*;

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn reserve_should_work_for_valid_window() {
        let (rsvp, _manager) = make_chalanzi_reservation(migrate_pool.clone()).await;
        assert!(rsvp.id != 0);
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
        let rsvp1 = manager.get(rsvp.id).await.unwrap();
        assert_eq!(rsvp1, rsvp)
    }

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn delete_reservation_should_work() {
        let (rsvp, manager) = make_waner_reservation(migrate_pool.clone()).await;
        manager.delete(rsvp.id).await.unwrap();
        let rsvp1 = manager.get(rsvp.id).await.unwrap_err();
        assert_eq!(rsvp1, abi::Error::RowNotFound);
    }

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn query_reservations_should_work() {
        let (rsvp, manager) = make_chalanzi_reservation(migrate_pool.clone()).await;

        let query = ReservationQueryBuilder::default()
            .user_id("chalanziId")
            .resource_id("")
            .start("2022-12-01T15:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2022-12-28T12:00:00-0700".parse::<Timestamp>().unwrap())
            .status(abi::ReservationStatus::Pending as i32)
            .build()
            .unwrap();

        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0], rsvp);

        // if window is not in range,should return empty
        let query = ReservationQueryBuilder::default()
            .user_id("chalanziId")
            .resource_id("")
            .start("2023-12-01T15:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2023-12-28T12:00:00-0700".parse::<Timestamp>().unwrap())
            .status(abi::ReservationStatus::Pending as i32)
            .build()
            .unwrap();

        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 0);

        // if status is not correct,should return empty
        let query = ReservationQueryBuilder::default()
            .user_id("chalanziId")
            .resource_id("")
            .start("2022-12-01T15:00:00-0700".parse::<Timestamp>().unwrap())
            .end("2022-12-28T12:00:00-0700".parse::<Timestamp>().unwrap())
            .status(abi::ReservationStatus::Confirmed as i32)
            .build()
            .unwrap();

        let rsvps = manager.query(query.clone()).await.unwrap();
        assert_eq!(rsvps.len(), 0);

        // change state to Confirmed, query should get result
        let rsvp = manager.change_status(rsvp.id).await.unwrap();
        let rsvps = manager.query(query).await.unwrap();
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0], rsvp)
    }

    #[sqlx_database_tester::test(pool(variable = "migrate_pool", migrations = "../migrations"))]
    async fn filter_reservations_should_work() {
        let (rsvp, manager) = make_chalanzi_reservation(migrate_pool.clone()).await;
        let filter = ReservationFilterBuilder::default()
            .user_id("chalanziId")
            .status(abi::ReservationStatus::Pending as i32)
            .build()
            .unwrap();

        let (pager, rsvps) = manager.filter(filter).await.unwrap();
        assert_eq!(pager.prev, None);
        assert_eq!(pager.next, None);
        assert_eq!(rsvps.len(), 1);
        assert_eq!(rsvps[0], rsvp);
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
