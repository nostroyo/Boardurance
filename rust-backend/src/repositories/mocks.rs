use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::domain::{Car, LapAction, LapResult, Pilot, Player, Race, RaceStatus, TeamName, WalletAddress};
use crate::services::car_validation::ValidatedCarData;
use crate::services::session::Session;
use super::{PlayerRepository, RaceRepository, SessionRepository, RepositoryError, RepositoryResult};

/// Mock implementation of PlayerRepository for testing
#[derive(Clone)]
pub struct MockPlayerRepository {
    players: Arc<Mutex<HashMap<String, Player>>>, // Using wallet_address as key
    players_by_uuid: Arc<Mutex<HashMap<Uuid, Player>>>,
}

impl MockPlayerRepository {
    pub fn new() -> Self {
        Self {
            players: Arc::new(Mutex::new(HashMap::new())),
            players_by_uuid: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_players(players: Vec<Player>) -> Self {
        let mut wallet_map = HashMap::new();
        let mut uuid_map = HashMap::new();
        
        for player in players {
            wallet_map.insert(player.wallet_address.to_string(), player.clone());
            uuid_map.insert(player.uuid, player);
        }
        
        Self {
            players: Arc::new(Mutex::new(wallet_map)),
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
        
        if players.contains_key(&player.wallet_address.to_string()) {
            return Err(RepositoryError::Conflict("Player with this wallet address already exists".to_string()));
        }
        
        players.insert(player.wallet_address.to_string(), player.clone());
        players_by_uuid.insert(player.uuid, player.clone());
        Ok(player.clone())
    }

    async fn find_all(&self) -> RepositoryResult<Vec<Player>> {
        let players = self.players.lock().unwrap();
        Ok(players.values().cloned().collect())
    }

    async fn find_by_wallet_address(&self, wallet_address: &str) -> RepositoryResult<Option<Player>> {
        let players = self.players.lock().unwrap();
        Ok(players.get(wallet_address).cloned())
    }

    async fn find_by_email(&self, email: &str) -> RepositoryResult<Option<Player>> {
        let players = self.players.lock().unwrap();
        Ok(players.values().find(|p| p.email == email).cloned())
    }

    async fn find_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let players_by_uuid = self.players_by_uuid.lock().unwrap();
        Ok(players_by_uuid.get(&player_uuid).cloned())
    }

    async fn update_team_name_by_wallet(&self, wallet_address: &str, team_name: TeamName) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players.get_mut(wallet_address) {
            player.team_name = team_name;
            players_by_uuid.insert(player.uuid, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn update_team_name_by_uuid(&self, player_uuid: Uuid, team_name: TeamName) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.team_name = team_name;
            players.insert(player.wallet_address.to_string(), player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn update_wallet_address(&self, player_uuid: Uuid, wallet_address: WalletAddress) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(mut player) = players_by_uuid.get(&player_uuid).cloned() {
            // Remove old wallet address entry
            players.remove(&player.wallet_address.to_string());
            
            // Update wallet address
            player.wallet_address = wallet_address;
            
            // Insert with new wallet address
            players.insert(player.wallet_address.to_string(), player.clone());
            players_by_uuid.insert(player_uuid, player.clone());
            
            Ok(Some(player))
        } else {
            Ok(None)
        }
    }

    async fn delete_by_wallet_address(&self, wallet_address: &str) -> RepositoryResult<bool> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players.remove(wallet_address) {
            players_by_uuid.remove(&player.uuid);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn delete_by_uuid(&self, player_uuid: Uuid) -> RepositoryResult<bool> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players_by_uuid.remove(&player_uuid) {
            players.remove(&player.wallet_address.to_string());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    async fn add_car_by_wallet(&self, wallet_address: &str, car: Car) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players.get_mut(wallet_address) {
            player.cars.push(car);
            players_by_uuid.insert(player.uuid, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn add_car_by_uuid(&self, player_uuid: Uuid, car: Car) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.cars.push(car);
            players.insert(player.wallet_address.to_string(), player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn remove_car_by_wallet(&self, wallet_address: &str, car_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players.get_mut(wallet_address) {
            player.cars.retain(|car| car.uuid != car_uuid);
            players_by_uuid.insert(player.uuid, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn remove_car_by_uuid(&self, player_uuid: Uuid, car_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.cars.retain(|car| car.uuid != car_uuid);
            players.insert(player.wallet_address.to_string(), player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn add_pilot_by_wallet(&self, wallet_address: &str, pilot: Pilot) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players.get_mut(wallet_address) {
            player.pilots.push(pilot);
            players_by_uuid.insert(player.uuid, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn add_pilot_by_uuid(&self, player_uuid: Uuid, pilot: Pilot) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.pilots.push(pilot);
            players.insert(player.wallet_address.to_string(), player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn remove_pilot_by_wallet(&self, wallet_address: &str, pilot_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players.get_mut(wallet_address) {
            player.pilots.retain(|pilot| pilot.uuid != pilot_uuid);
            players_by_uuid.insert(player.uuid, player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn remove_pilot_by_uuid(&self, player_uuid: Uuid, pilot_uuid: Uuid) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.pilots.retain(|pilot| pilot.uuid != pilot_uuid);
            players.insert(player.wallet_address.to_string(), player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }

    async fn set_cars_by_uuid(&self, player_uuid: Uuid, cars: Vec<Car>) -> RepositoryResult<Option<Player>> {
        let mut players = self.players.lock().unwrap();
        let mut players_by_uuid = self.players_by_uuid.lock().unwrap();
        
        if let Some(player) = players_by_uuid.get_mut(&player_uuid) {
            player.cars = cars;
            players.insert(player.wallet_address.to_string(), player.clone());
            Ok(Some(player.clone()))
        } else {
            Ok(None)
        }
    }
}

/// Mock implementation of RaceRepository for testing
#[derive(Clone)]
pub struct MockRaceRepository {
    races: Arc<Mutex<HashMap<Uuid, Race>>>,
}

impl MockRaceRepository {
    pub fn new() -> Self {
        Self {
            races: Arc::new(Mutex::new(HashMap::new())),
        }
    }

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
        Ok(races.values().find(|race| {
            race.pilots.iter().any(|pilot| pilot.uuid == pilot_uuid)
        }).cloned())
    }

    async fn find_active_race_for_pilot(&self, pilot_uuid: Uuid) -> RepositoryResult<Option<Race>> {
        let races = self.races.lock().unwrap();
        Ok(races.values().find(|race| {
            matches!(race.status, RaceStatus::WaitingForPlayers | RaceStatus::InProgress) &&
            race.pilots.iter().any(|pilot| pilot.uuid == pilot_uuid)
        }).cloned())
    }

    async fn join_race(&self, race_uuid: Uuid, pilot_uuid: Uuid, car_data: &ValidatedCarData) -> RepositoryResult<Option<Race>> {
        let mut races = self.races.lock().unwrap();
        
        if let Some(race) = races.get_mut(&race_uuid) {
            if !matches!(race.status, RaceStatus::WaitingForPlayers) {
                return Err(RepositoryError::Validation("Race is not accepting new players".to_string()));
            }
            
            if race.pilots.iter().any(|p| p.uuid == pilot_uuid) {
                return Err(RepositoryError::Conflict("Pilot already in race".to_string()));
            }
            
            let pilot_with_car = crate::domain::PilotWithCar {
                uuid: pilot_uuid,
                car: car_data.car.clone(),
                current_lap: 0,
                current_sector: 0,
                total_time: 0,
                sector_times: Vec::new(),
                lap_times: Vec::new(),
                position: race.pilots.len() as u32 + 1,
                boost_hand: car_data.boost_hand.clone(),
                actions_submitted: false,
            };
            
            race.pilots.push(pilot_with_car);
            Ok(Some(race.clone()))
        } else {
            Err(RepositoryError::NotFound)
        }
    }

    async fn process_turn_actions(&self, race_uuid: Uuid, pilot_uuid: Uuid, actions: Vec<LapAction>) -> RepositoryResult<Option<(LapResult, RaceStatus)>> {
        let mut races = self.races.lock().unwrap();
        
        if let Some(race) = races.get_mut(&race_uuid) {
            if let Some(pilot_index) = race.pilots.iter().position(|p| p.uuid == pilot_uuid) {
                let lap_result = LapResult {
                    lap_number: race.pilots[pilot_index].current_lap + 1,
                    sector_times: vec![30000, 32000, 28000],
                    total_lap_time: 90000,
                    position: race.pilots[pilot_index].position,
                    actions_used: actions,
                };
                
                race.pilots[pilot_index].current_lap += 1;
                race.pilots[pilot_index].lap_times.push(lap_result.total_lap_time);
                race.pilots[pilot_index].total_time += lap_result.total_lap_time;
                
                let race_status = if race.pilots[pilot_index].current_lap >= race.total_laps {
                    RaceStatus::Finished
                } else {
                    race.status
                };
                
                race.status = race_status;
                
                Ok(Some((lap_result, race_status)))
            } else {
                Err(RepositoryError::NotFound)
            }
        } else {
            Err(RepositoryError::NotFound)
        }
    }

    async fn submit_turn_action(&self, race_uuid: Uuid, pilot_uuid: Uuid, _boost_value: u32) -> RepositoryResult<Option<Race>> {
        let mut races = self.races.lock().unwrap();
        
        if let Some(race) = races.get_mut(&race_uuid) {
            if let Some(pilot) = race.pilots.iter_mut().find(|p| p.uuid == pilot_uuid) {
                pilot.actions_submitted = true;
                Ok(Some(race.clone()))
            } else {
                Err(RepositoryError::NotFound)
            }
        } else {
            Err(RepositoryError::NotFound)
        }
    }

    async fn update_race_status(&self, race_uuid: Uuid, status: RaceStatus) -> RepositoryResult<Option<Race>> {
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
        Ok(races.values().filter(|race| race.status == status).cloned().collect())
    }
}

/// Mock implementation of SessionRepository for testing
#[derive(Clone)]
pub struct MockSessionRepository {
    sessions: Arc<Mutex<HashMap<String, Session>>>, // Using token as key
}

impl MockSessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

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
        let count = sessions.values()
            .filter(|session| {
                session.user_uuid == user_uuid && 
                session.is_active && 
                session.expires_at > Utc::now()
            })
            .count();
        Ok(count)
    }
}