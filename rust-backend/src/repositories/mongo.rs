//! `MongoDB`-backed implementations of the repository traits, used for the
//! `production`/`preprod` environments (see `configuration::StorageBackend`).
//!
//! Each repository is a thin persistence wrapper: reads deserialize a
//! document into the domain type; mutations load the domain aggregate, call
//! the *same* domain method the mock repository calls (`Race::process_lap`,
//! `Race::add_participant`, ...), then serialize and save it back. This keeps
//! all game/business logic on the domain structs (ADR 0001) and guarantees
//! behavioral parity with the `Mock*` repositories — enforced by the shared
//! conformance suite in `tests/repository_conformance.rs`.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mongodb::bson::{doc, Document};
use mongodb::options::{IndexOptions, ReplaceOptions};
use mongodb::{Collection, Database, IndexModel};
use uuid::Uuid;

use super::{
    PlayerRepository, RaceRepository, RepositoryError, RepositoryResult, SessionRepository,
};
use crate::domain::{Car, LapAction, LapResult, Pilot, Player, Race, RaceStatus, TeamName};
use crate::services::car_validation::ValidatedCarData;
use crate::services::session::Session;

/// Convert a `mongodb` driver error into the repository's error type. Since
/// the mock repositories never surface a "database" error variant, driver
/// errors are treated as a validation failure describing what went wrong —
/// there is no better-fitting `RepositoryError` variant, and this keeps
/// callers (which only match on `NotFound`/`Conflict`/`Validation`) working
/// unchanged.
fn map_mongo_err(err: mongodb::error::Error) -> RepositoryError {
    RepositoryError::Validation(format!("Database error: {err}"))
}

fn map_bson_ser_err(err: mongodb::bson::ser::Error) -> RepositoryError {
    RepositoryError::Validation(format!("Serialization error: {err}"))
}

fn map_bson_de_err(err: mongodb::bson::de::Error) -> RepositoryError {
    RepositoryError::Validation(format!("Deserialization error: {err}"))
}

/// `MongoDB`-backed `PlayerRepository`. Documents are keyed by `uuid` (stored
/// as a string, matching `Player`'s `uuid_as_string` serde helper).
#[derive(Clone)]
pub struct MongoPlayerRepository {
    collection: Collection<Document>,
}

impl MongoPlayerRepository {
    /// Connect to the `players` collection and ensure a unique index on `uuid`.
    pub async fn new(database: &Database) -> Result<Self, mongodb::error::Error> {
        let collection = database.collection::<Document>("players");
        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "uuid": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
                None,
            )
            .await?;
        Ok(Self { collection })
    }
}

#[async_trait]
impl PlayerRepository for MongoPlayerRepository {
    async fn create(&self, player: &Player) -> RepositoryResult<Player> {
        let doc = mongodb::bson::to_document(player).map_err(map_bson_ser_err)?;
        match self.collection.insert_one(doc, None).await {
            Ok(_) => Ok(player.clone()),
            Err(e) if is_duplicate_key_error(&e) => Err(RepositoryError::Conflict(
                "Player with this email already exists".to_string(),
            )),
            Err(e) => Err(map_mongo_err(e)),
        }
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Player>> {
        let mut cursor = self
            .collection
            .find(None, None)
            .await
            .map_err(map_mongo_err)?;
        let mut players = Vec::new();
        while cursor.advance().await.map_err(map_mongo_err)? {
            let doc = cursor.deserialize_current().map_err(map_mongo_err)?;
            players.push(mongodb::bson::from_document(doc).map_err(map_bson_de_err)?);
        }
        Ok(players)
    }

    async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<Player>> {
        let filter = doc! { "email": email };
        let doc = self
            .collection
            .find_one(filter, None)
            .await
            .map_err(map_mongo_err)?;
        doc.map(|d| mongodb::bson::from_document(d).map_err(map_bson_de_err))
            .transpose()
    }

    async fn find_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let filter = doc! { "uuid": player_uuid.to_string() };
        let doc = self
            .collection
            .find_one(filter, None)
            .await
            .map_err(map_mongo_err)?;
        doc.map(|d| mongodb::bson::from_document(d).map_err(map_bson_de_err))
            .transpose()
    }

    async fn update_team_name_by_uuid(
        &self,
        player_uuid: Uuid,
        team_name: TeamName,
    ) -> RepositoryResult<Option<Player>> {
        let Some(mut player) = self.find_by_uuid(player_uuid).await? else {
            return Ok(None);
        };
        player.update_team_name(team_name);
        self.save(&player).await?;
        Ok(Some(player))
    }

    async fn delete_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<bool> {
        let filter = doc! { "uuid": player_uuid.to_string() };
        let result = self
            .collection
            .delete_one(filter, None)
            .await
            .map_err(map_mongo_err)?;
        Ok(result.deleted_count > 0)
    }

    async fn add_car_by_uuid(
        &self,
        player_uuid: Uuid,
        car: Car,
    ) -> RepositoryResult<Option<Player>> {
        let Some(mut player) = self.find_by_uuid(player_uuid).await? else {
            return Ok(None);
        };
        player.cars.push(car);
        player.updated_at = Utc::now();
        self.save(&player).await?;
        Ok(Some(player))
    }

    async fn remove_car_by_uuid(
        &self,
        player_uuid: Uuid,
        car_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>> {
        let Some(mut player) = self.find_by_uuid(player_uuid).await? else {
            return Ok(None);
        };
        player.cars.retain(|car| car.uuid != car_uuid);
        player.updated_at = Utc::now();
        self.save(&player).await?;
        Ok(Some(player))
    }

    async fn add_pilot_by_uuid(
        &self,
        player_uuid: Uuid,
        pilot: Pilot,
    ) -> RepositoryResult<Option<Player>> {
        let Some(mut player) = self.find_by_uuid(player_uuid).await? else {
            return Ok(None);
        };
        player.pilots.push(pilot);
        player.updated_at = Utc::now();
        self.save(&player).await?;
        Ok(Some(player))
    }

    async fn remove_pilot_by_uuid(
        &self,
        player_uuid: Uuid,
        pilot_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>> {
        let Some(mut player) = self.find_by_uuid(player_uuid).await? else {
            return Ok(None);
        };
        player.pilots.retain(|pilot| pilot.uuid != pilot_uuid);
        player.updated_at = Utc::now();
        self.save(&player).await?;
        Ok(Some(player))
    }

    async fn set_cars_by_uuid(
        &self,
        player_uuid: Uuid,
        cars: Vec<Car>,
    ) -> RepositoryResult<Option<Player>> {
        let Some(mut player) = self.find_by_uuid(player_uuid).await? else {
            return Ok(None);
        };
        player.cars = cars;
        player.updated_at = Utc::now();
        self.save(&player).await?;
        Ok(Some(player))
    }
}

impl MongoPlayerRepository {
    /// Persist a full `Player` document (upsert by uuid).
    async fn save(&self, player: &Player) -> RepositoryResult<()> {
        let filter = doc! { "uuid": player.uuid.to_string() };
        let doc = mongodb::bson::to_document(player).map_err(map_bson_ser_err)?;
        self.collection
            .replace_one(filter, doc, ReplaceOptions::builder().upsert(true).build())
            .await
            .map_err(map_mongo_err)?;
        Ok(())
    }
}

/// `MongoDB`-backed `RaceRepository`. Documents are keyed by `uuid` (stored as
/// a string, matching `Race`'s `uuid_as_string` serde helper).
#[derive(Clone)]
pub struct MongoRaceRepository {
    collection: Collection<Document>,
}

impl MongoRaceRepository {
    /// Connect to the `races` collection and ensure a unique index on `uuid`.
    pub async fn new(database: &Database) -> Result<Self, mongodb::error::Error> {
        let collection = database.collection::<Document>("races");
        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "uuid": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
                None,
            )
            .await?;
        Ok(Self { collection })
    }

    async fn load(&self, race_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        let filter = doc! { "uuid": race_uuid.to_string() };
        let doc = self
            .collection
            .find_one(filter, None)
            .await
            .map_err(map_mongo_err)?;
        doc.map(|d| mongodb::bson::from_document(d).map_err(map_bson_de_err))
            .transpose()
    }

    async fn save(&self, race: &Race) -> RepositoryResult<()> {
        let filter = doc! { "uuid": race.uuid.to_string() };
        let doc = mongodb::bson::to_document(race).map_err(map_bson_ser_err)?;
        self.collection
            .replace_one(filter, doc, ReplaceOptions::builder().upsert(true).build())
            .await
            .map_err(map_mongo_err)?;
        Ok(())
    }
}

#[async_trait]
impl RaceRepository for MongoRaceRepository {
    async fn create(&self, race: &Race) -> RepositoryResult<Race> {
        self.save(race).await?;
        Ok(race.clone())
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Race>> {
        let mut cursor = self
            .collection
            .find(None, None)
            .await
            .map_err(map_mongo_err)?;
        let mut races = Vec::new();
        while cursor.advance().await.map_err(map_mongo_err)? {
            let doc = cursor.deserialize_current().map_err(map_mongo_err)?;
            races.push(mongodb::bson::from_document(doc).map_err(map_bson_de_err)?);
        }
        Ok(races)
    }

    async fn find_by_uuid(&self, race_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        self.load(race_uuid).await
    }

    async fn find_by_pilot_uuid(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        // Participants are stored in an array; matching a nested field inside
        // an array element requires no extra options beyond the dotted path.
        let filter = doc! { "participants.pilot_uuid": pilot_uuid.to_string() };
        let doc = self
            .collection
            .find_one(filter, None)
            .await
            .map_err(map_mongo_err)?;
        doc.map(|d| mongodb::bson::from_document(d).map_err(map_bson_de_err))
            .transpose()
    }

    async fn find_active_race_for_pilot(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        // Status is a Rust enum serialized as a bare string tag (e.g.
        // "Waiting"); filter in Rust to avoid depending on the exact bson
        // representation of the enum variants.
        let races = self.find_all().await?;
        Ok(races.into_iter().find(|race| {
            matches!(race.status, RaceStatus::Waiting | RaceStatus::InProgress)
                && race
                    .participants
                    .iter()
                    .any(|participant| participant.pilot_uuid == pilot_uuid)
        }))
    }

    async fn join_race(
        &self,
        race_uuid: Uuid,
        pilot_uuid: Uuid,
        car_data: &ValidatedCarData,
    ) -> RepositoryResult<Option<Race>> {
        let Some(mut race) = self.load(race_uuid).await? else {
            return Err(RepositoryError::NotFound);
        };

        if !matches!(race.status, RaceStatus::Waiting) {
            return Err(RepositoryError::Validation(
                "Race is not accepting new players".to_string(),
            ));
        }

        if race.participants.iter().any(|p| p.pilot_uuid == pilot_uuid) {
            return Err(RepositoryError::Conflict(
                "Pilot already in race".to_string(),
            ));
        }

        race.add_participant(car_data.pilot.uuid, car_data.car.uuid, pilot_uuid)
            .map_err(RepositoryError::Validation)?;

        self.save(&race).await?;
        Ok(Some(race))
    }

    async fn process_turn_actions(
        &self,
        race_uuid: Uuid,
        _pilot_uuid: Uuid,
        actions: Vec<LapAction>,
    ) -> RepositoryResult<Option<(LapResult, RaceStatus)>> {
        let Some(mut race) = self.load(race_uuid).await? else {
            return Err(RepositoryError::NotFound);
        };

        let lap_result = race
            .process_lap(&actions)
            .map_err(RepositoryError::Validation)?;
        let race_status = race.status.clone();

        self.save(&race).await?;
        Ok(Some((lap_result, race_status)))
    }

    async fn submit_turn_action(
        &self,
        race_uuid: Uuid,
        pilot_uuid: Uuid,
        _boost_value: u32,
    ) -> RepositoryResult<Option<Race>> {
        let Some(race) = self.load(race_uuid).await? else {
            return Err(RepositoryError::NotFound);
        };

        if race.participants.iter().any(|p| p.pilot_uuid == pilot_uuid) {
            Ok(Some(race))
        } else {
            Err(RepositoryError::NotFound)
        }
    }

    async fn update_race_status(
        &self,
        race_uuid: Uuid,
        status: RaceStatus,
    ) -> RepositoryResult<Option<Race>> {
        let Some(mut race) = self.load(race_uuid).await? else {
            return Ok(None);
        };
        race.status = status;
        self.save(&race).await?;
        Ok(Some(race))
    }

    async fn get_races_by_status(&self, status: RaceStatus) -> RepositoryResult<Vec<Race>> {
        let races = self.find_all().await?;
        Ok(races
            .into_iter()
            .filter(|race| race.status == status)
            .collect())
    }
}

/// `MongoDB`-backed `SessionRepository`. Documents are keyed by `token`.
#[derive(Clone)]
pub struct MongoSessionRepository {
    collection: Collection<Document>,
}

impl MongoSessionRepository {
    /// Connect to the `sessions` collection and ensure a unique index on `token`.
    pub async fn new(database: &Database) -> Result<Self, mongodb::error::Error> {
        let collection = database.collection::<Document>("sessions");
        collection
            .create_index(
                IndexModel::builder()
                    .keys(doc! { "token": 1 })
                    .options(IndexOptions::builder().unique(true).build())
                    .build(),
                None,
            )
            .await?;
        Ok(Self { collection })
    }
}

#[async_trait]
impl SessionRepository for MongoSessionRepository {
    async fn create(&self, session: &Session) -> RepositoryResult<()> {
        let doc = mongodb::bson::to_document(session).map_err(map_bson_ser_err)?;
        self.collection
            .insert_one(doc, None)
            .await
            .map_err(map_mongo_err)?;
        Ok(())
    }

    async fn find_by_token(&self, token: &str) -> RepositoryResult<Option<Session>> {
        let filter = doc! { "token": token };
        let doc = self
            .collection
            .find_one(filter, None)
            .await
            .map_err(map_mongo_err)?;
        let Some(doc) = doc else {
            return Ok(None);
        };
        let session: Session = mongodb::bson::from_document(doc).map_err(map_bson_de_err)?;
        if session.is_active && session.expires_at > Utc::now() {
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    async fn deactivate(&self, token: &str) -> RepositoryResult<()> {
        let filter = doc! { "token": token };
        let update = doc! {
            "$set": {
                "is_active": false,
                "updated_at": mongodb::bson::DateTime::from_chrono(Utc::now()),
            }
        };
        self.collection
            .update_one(filter, update, None)
            .await
            .map_err(map_mongo_err)?;
        Ok(())
    }

    async fn deactivate_all_for_user(&self, user_uuid: Uuid) -> RepositoryResult<()> {
        let filter = doc! { "user_uuid": user_uuid.to_string(), "is_active": true };
        let update = doc! {
            "$set": {
                "is_active": false,
                "updated_at": mongodb::bson::DateTime::from_chrono(Utc::now()),
            }
        };
        self.collection
            .update_many(filter, update, None)
            .await
            .map_err(map_mongo_err)?;
        Ok(())
    }

    async fn cleanup_expired(&self, now: DateTime<Utc>) -> RepositoryResult<u64> {
        let filter = doc! {
            "expires_at": { "$lte": mongodb::bson::DateTime::from_chrono(now) }
        };
        let result = self
            .collection
            .delete_many(filter, None)
            .await
            .map_err(map_mongo_err)?;
        Ok(result.deleted_count)
    }

    async fn count_active_for_user(&self, user_uuid: Uuid) -> RepositoryResult<usize> {
        let filter = doc! {
            "user_uuid": user_uuid.to_string(),
            "is_active": true,
            "expires_at": { "$gt": mongodb::bson::DateTime::from_chrono(Utc::now()) }
        };
        let count = self
            .collection
            .count_documents(filter, None)
            .await
            .map_err(map_mongo_err)?;
        Ok(count as usize)
    }
}

/// `MongoDB`'s duplicate-key error code.
const DUPLICATE_KEY_CODE: i32 = 11000;

/// Detect a duplicate-key write error (used to map the unique `uuid`/`email`
/// index violation to `RepositoryError::Conflict`, mirroring the mock's
/// explicit "already exists" check).
fn is_duplicate_key_error(err: &mongodb::error::Error) -> bool {
    use mongodb::error::{ErrorKind, WriteFailure};

    match err.kind.as_ref() {
        ErrorKind::Write(WriteFailure::WriteError(we)) => we.code == DUPLICATE_KEY_CODE,
        ErrorKind::BulkWrite(bwe) => bwe
            .write_errors
            .as_ref()
            .is_some_and(|errs| errs.iter().any(|e| e.code == DUPLICATE_KEY_CODE)),
        _ => false,
    }
}
