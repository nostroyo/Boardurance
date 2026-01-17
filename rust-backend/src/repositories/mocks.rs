use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::{
    PlayerRepository, RaceRepository, RepositoryError, RepositoryResult, SessionRepository,
};
use crate::domain::{
    Car, LapAction, LapResult, Pilot, Player, Race, RaceStatus, TeamName, WalletAddress,
};
use crate::services::car_validation::ValidatedCarData;
use crate::services::session::Session;

/// Mock implementation of `PlayerRepository` for testing
#[derive(Clone)]
pub struct MockPlayerRepository {
    players: Arc<Mutex<HashMap<String, Player>>>, // Using email as key for simplicity
    players_by_uuid: Arc<Mutex<HashMap<Uuid, Player>>>,
}

impl MockPlayerRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            players: Arc::new(Mutex::new(HashMap::new())),
            players_by_uuid: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[must_use]
    pub fn with_players(players: Vec<Player>) -> Self {
        let mut email_map = HashMap::new();
        let mut uuid_map = HashMap::new();

        for player in players {
            email_map.insert(player.email.as_ref().to_string(), player.clone());
            uuid_map.insert(player.uuid, player);
        }

        Self {
            players: Arc::new(Mutex::new(email_map)),
            players_by_uuid: Arc::new(Mutex::new(uuid_map)),
        }
    }
}

impl Default for MockPlayerRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PlayerRepository for MockPlayerRepository {
    async fn create(&self, player: &Player) -> RepositoryResult<Player> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        let email_key = player.email.as_ref().to_string();
        if players.contains_key(&email_key) {
            return Err(RepositoryError::Conflict(
                "Player with this email already exists".to_string(),
            ));
        }

        players.insert(email_key, player.clone());
        players_by_uuid.insert(player.uuid, player.clone());
        Ok(player.clone())
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Player>> {
        let players = self.players.lock().unwrap();
        Ok(players.values().cloned().collect())
    }

    async fn find_by_wallet_address(
        &self,
        wallet_address: &str,
    ) -> RepositoryResult<Option<Player>> {
        let players = self.players.lock().unwrap();
        Ok(players
            .values()
            .find(|p| {
                p.wallet_address.as_ref().map(std::convert::AsRef::as_ref) == Some(wallet_address)
            })
            .cloned())
    }

    async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<Player>> {
        let players = self.players.lock().unwrap();
        Ok(players.get(email).cloned())
    }

    async fn find_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let players_by_uuid = self.players_by_uuid.lock().unwrap();
        Ok(players_by_uuid.get(&player_uuid).cloned())
    }

    async fn update_team_name_by_wallet(
        &self,
        wallet_address: &str,
        team_name: TeamName,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        for player in players.values_mut() {
            if player
                .wallet_address
                .as_ref()
                .map(std::convert::AsRef::as_ref)
                == Some(wallet_address)
            {
                player.team_name = team_name;
                player.updated_at = Utc::now();
                players_by_uuid.insert(player.uuid, player.clone());
                return Ok(Some(player.clone()));
            }
        }
        Ok(None)
    }

    async fn update_team_name_by_uuid(
        &self,
        player_uuid: Uuid,
        team_name: TeamName,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.team_name = team_name;
            player.updated_at = Utc::now();
            let email_key = player.email.as_ref().to_string();
            players.insert(email_key, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn update_wallet_address(
        &self,
        player_uuid: Uuid,
        wallet_address: WalletAddress,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.wallet_address = Some(wallet_address);
            player.updated_at = Utc::now();
            let email_key = player.email.as_ref().to_string();
            players.insert(email_key, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn delete_by_wallet_address(&self, wallet_address: &str) -> RepositoryResult<bool> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        let mut found_player = None;
        for (email, player) in players.iter() {
            if player
                .wallet_address
                .as_ref()
                .map(std::convert::AsRef::as_ref)
                == Some(wallet_address)
            {
                found_player = Some((email.clone(), player.uuid));
                break;
            }
        }

        if let Some((email, uuid)) = found_player {
            players.remove(&email);
            players_by_uuid.remove(&uuid);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<bool> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        if let Some(player) = players_by_uuid.remove(&player_uuid) {
            let email_key = player.email.as_ref().to_string();
            players.remove(&email_key);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn add_car_by_wallet(
        &self,
        wallet_address: &str,
        car: Car,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        for player in players.values_mut() {
            if player
                .wallet_address
                .as_ref()
                .map(std::convert::AsRef::as_ref)
                == Some(wallet_address)
            {
                player.cars.push(car);
                player.updated_at = Utc::now();
                players_by_uuid.insert(player.uuid, player.clone());
                return Ok(Some(player.clone()));
            }
        }
        Ok(None)
    }

    async fn add_car_by_uuid(
        &self,
        player_uuid: Uuid,
        car: Car,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.cars.push(car);
            player.updated_at = Utc::now();
            let email_key = player.email.as_ref().to_string();
            players.insert(email_key, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn remove_car_by_wallet(
        &self,
        wallet_address: &str,
        car_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        for player in players.values_mut() {
            if player
                .wallet_address
                .as_ref()
                .map(std::convert::AsRef::as_ref)
                == Some(wallet_address)
            {
                player.cars.retain(|car| car.uuid != car_uuid);
                player.updated_at = Utc::now();
                players_by_uuid.insert(player.uuid, player.clone());
                return Ok(Some(player.clone()));
            }
        }
        Ok(None)
    }

    async fn remove_car_by_uuid(
        &self,
        player_uuid: Uuid,
        car_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.cars.retain(|car| car.uuid != car_uuid);
            player.updated_at = Utc::now();
            let email_key = player.email.as_ref().to_string();
            players.insert(email_key, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn add_pilot_by_wallet(
        &self,
        wallet_address: &str,
        pilot: Pilot,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        for player in players.values_mut() {
            if player
                .wallet_address
                .as_ref()
                .map(std::convert::AsRef::as_ref)
                == Some(wallet_address)
            {
                player.pilots.push(pilot);
                player.updated_at = Utc::now();
                players_by_uuid.insert(player.uuid, player.clone());
                return Ok(Some(player.clone()));
            }
        }
        Ok(None)
    }

    async fn add_pilot_by_uuid(
        &self,
        player_uuid: Uuid,
        pilot: Pilot,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.pilots.push(pilot);
            player.updated_at = Utc::now();
            let email_key = player.email.as_ref().to_string();
            players.insert(email_key, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn remove_pilot_by_wallet(
        &self,
        wallet_address: &str,
        pilot_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        for player in players.values_mut() {
            if player
                .wallet_address
                .as_ref()
                .map(std::convert::AsRef::as_ref)
                == Some(wallet_address)
            {
                player.pilots.retain(|pilot| pilot.uuid != pilot_uuid);
                player.updated_at = Utc::now();
                players_by_uuid.insert(player.uuid, player.clone());
                return Ok(Some(player.clone()));
            }
        }
        Ok(None)
    }

    async fn remove_pilot_by_uuid(
        &self,
        player_uuid: Uuid,
        pilot_uuid: Uuid,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.pilots.retain(|pilot| pilot.uuid != pilot_uuid);
            player.updated_at = Utc::now();
            let email_key = player.email.as_ref().to_string();
            players.insert(email_key, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn set_cars_by_uuid(
        &self,
        player_uuid: Uuid,
        cars: Vec<Car>,
    ) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();

        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.cars = cars;
            player.updated_at = Utc::now();
            let email_key = player.email.as_ref().to_string();
            players.insert(email_key, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }
}

/// Mock implementation of `RaceRepository` for testing
#[derive(Clone)]
pub struct MockRaceRepository {
    races: Arc<Mutex<HashMap<Uuid, Race>>>,
}

impl MockRaceRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            races: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[must_use]
    pub fn with_races(races: Vec<Race>) -> Self {
        let mut race_map = HashMap::new();
        for race in races {
            race_map.insert(race.uuid, race);
        }

        Self {
            races: Arc::new(Mutex::new(race_map)),
        }
    }
}

impl Default for MockRaceRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl RaceRepository for MockRaceRepository {
    async fn create(&self, race: &Race) -> RepositoryResult<Race> {
        let mut races = self.races.lock().unwrap();
        races.insert(race.uuid, race.clone());
        Ok(race.clone())
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Race>> {
        let races = self.races.lock().unwrap();
        Ok(races.values().cloned().collect())
    }

    async fn find_by_uuid(&self, race_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        let races = self.races.lock().unwrap();
        Ok(races.get(&race_uuid).cloned())
    }

    async fn find_by_pilot_uuid(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        let races = self.races.lock().unwrap();
        Ok(races
            .values()
            .find(|race| {
                race.participants
                    .iter()
                    .any(|participant| participant.pilot_uuid == pilot_uuid)
            })
            .cloned())
    }

    async fn find_active_race_for_pilot(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        let races = self.races.lock().unwrap();
        Ok(races
            .values()
            .find(|race| {
                matches!(race.status, RaceStatus::Waiting | RaceStatus::InProgress)
                    && race
                        .participants
                        .iter()
                        .any(|participant| participant.pilot_uuid == pilot_uuid)
            })
            .cloned())
    }

    async fn join_race(
        &self,
        race_uuid: Uuid,
        pilot_uuid: Uuid,
        car_data: &ValidatedCarData,
    ) -> RepositoryResult<Option<Race>> {
        let mut races = self.races.lock().unwrap();

        if let Some(race) = races.get_mut(&race_uuid) {
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

            // Add participant using the race's add_participant method
            // For mock implementation, we'll use the pilot's UUID as player UUID for simplicity
            race.add_participant(car_data.pilot.uuid, car_data.car.uuid, pilot_uuid)
                .map_err(RepositoryError::Validation)?;

            Ok(Some(race.clone()))
        } else {
            Err(RepositoryError::NotFound)
        }
    }

    async fn process_turn_actions(
        &self,
        race_uuid: Uuid,
        _pilot_uuid: Uuid,
        actions: Vec<LapAction>,
    ) -> RepositoryResult<Option<(LapResult, RaceStatus)>> {
        let mut races = self.races.lock().unwrap();

        if let Some(race) = races.get_mut(&race_uuid) {
            // For mock implementation, just process the actions with simple logic
            let lap_result = race
                .process_lap(&actions)
                .map_err(RepositoryError::Validation)?;

            let race_status = race.status.clone();

            Ok(Some((lap_result, race_status)))
        } else {
            Err(RepositoryError::NotFound)
        }
    }

    async fn submit_turn_action(
        &self,
        race_uuid: Uuid,
        pilot_uuid: Uuid,
        _boost_value: u32,
    ) -> RepositoryResult<Option<Race>> {
        let mut races = self.races.lock().unwrap();

        if let Some(race) = races.get_mut(&race_uuid) {
            // For mock implementation, just mark that an action was submitted
            // In a real implementation, this would store the action for batch processing
            if race.participants.iter().any(|p| p.pilot_uuid == pilot_uuid) {
                Ok(Some(race.clone()))
            } else {
                Err(RepositoryError::NotFound)
            }
        } else {
            Err(RepositoryError::NotFound)
        }
    }

    async fn update_race_status(
        &self,
        race_uuid: Uuid,
        status: RaceStatus,
    ) -> RepositoryResult<Option<Race>> {
        let mut races = self.races.lock().unwrap();

        if let Some(race) = races.get_mut(&race_uuid) {
            race.status = status;
            Ok(Some(race.clone()))
        } else {
            Ok(None)
        }
    }

    async fn get_races_by_status(&self, status: RaceStatus) -> RepositoryResult<Vec<Race>> {
        let races = self.races.lock().unwrap();
        Ok(races
            .values()
            .filter(|race| race.status == status)
            .cloned()
            .collect())
    }
}

/// Mock implementation of `SessionRepository` for testing
#[derive(Clone)]
pub struct MockSessionRepository {
    sessions: Arc<Mutex<HashMap<String, Session>>>, // Using token as key
}

impl MockSessionRepository {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    #[must_use]
    pub fn with_sessions(sessions: Vec<Session>) -> Self {
        let mut session_map = HashMap::new();
        for session in sessions {
            session_map.insert(session.token.clone(), session);
        }

        Self {
            sessions: Arc::new(Mutex::new(session_map)),
        }
    }
}

impl Default for MockSessionRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SessionRepository for MockSessionRepository {
    async fn create(&self, session: &Session) -> RepositoryResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.insert(session.token.clone(), session.clone());
        Ok(())
    }

    async fn find_by_token(&self, token: &str) -> RepositoryResult<Option<Session>> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(token) {
            if session.is_active && session.expires_at > Utc::now() {
                Ok(Some(session.clone()))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    async fn deactivate(&self, token: &str) -> RepositoryResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(token) {
            session.is_active = false;
            session.updated_at = Utc::now();
        }
        Ok(())
    }

    async fn deactivate_all_for_user(&self, user_uuid: Uuid) -> RepositoryResult<()> {
        let mut sessions = self.sessions.lock().unwrap();
        for session in sessions.values_mut() {
            if session.user_uuid == user_uuid && session.is_active {
                session.is_active = false;
                session.updated_at = Utc::now();
            }
        }
        Ok(())
    }

    async fn cleanup_expired(&self, now: DateTime<Utc>) -> RepositoryResult<u64> {
        let mut sessions = self.sessions.lock().unwrap();
        let initial_count = sessions.len();
        sessions.retain(|_, session| session.expires_at > now);
        let final_count = sessions.len();
        Ok((initial_count - final_count) as u64)
    }

    async fn count_active_for_user(&self, user_uuid: Uuid) -> RepositoryResult<usize> {
        let sessions = self.sessions.lock().unwrap();
        let count = sessions
            .values()
            .filter(|session| {
                session.user_uuid == user_uuid
                    && session.is_active
                    && session.expires_at > Utc::now()
            })
            .count();
        Ok(count)
    }
}
