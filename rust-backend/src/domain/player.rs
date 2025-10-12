use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use unicode_segmentation::UnicodeSegmentation;

use super::{Car, Pilot};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Player {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    #[serde(with = "uuid_as_string")]
    pub uuid: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<WalletAddress>,
    pub team_name: TeamName,
    pub cars: Vec<Car>,
    pub pilots: Vec<Pilot>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

mod uuid_as_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use uuid::Uuid;

    pub fn serialize<S>(uuid: &Uuid, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&uuid.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Uuid::parse_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct WalletAddress(String);

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TeamName(String);

impl Player {
    pub fn new(
        wallet_address: Option<WalletAddress>,
        team_name: TeamName,
        cars: Vec<Car>,
        pilots: Vec<Pilot>,
    ) -> Result<Self, String> {
        let now = Utc::now();
        Ok(Self {
            id: None,
            uuid: Uuid::new_v4(),
            wallet_address,
            team_name,
            cars,
            pilots,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn validate_for_game(&self) -> Result<(), String> {
        // Validate player constraints for game participation
        if self.wallet_address.is_none() {
            return Err("Player must have a connected wallet to participate in games".to_string());
        }

        if self.cars.len() != 2 {
            return Err("Player must have exactly 2 cars to participate in games".to_string());
        }

        if self.pilots.is_empty() {
            return Err("Player must have at least one pilot to participate in games".to_string());
        }

        Ok(())
    }

    pub fn connect_wallet(&mut self, wallet_address: WalletAddress) -> Result<(), String> {
        if self.wallet_address.is_some() {
            return Err("Player already has a connected wallet".to_string());
        }
        self.wallet_address = Some(wallet_address);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn disconnect_wallet(&mut self) {
        self.wallet_address = None;
        self.updated_at = Utc::now();
    }

    #[must_use]
    pub fn is_wallet_connected(&self) -> bool {
        self.wallet_address.is_some()
    }

    #[must_use]
    pub fn get_wallet_address(&self) -> Option<&str> {
        self.wallet_address.as_ref().map(std::convert::AsRef::as_ref)
    }

    pub fn update_team_name(&mut self, new_team_name: TeamName) {
        self.team_name = new_team_name;
        self.updated_at = Utc::now();
    }

    pub fn add_car(&mut self, car: Car) -> Result<(), String> {
        if self.cars.len() >= 2 {
            return Err("Player can only have 2 cars maximum".to_string());
        }
        self.cars.push(car);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_car(&mut self, car_uuid: Uuid) -> Result<(), String> {
        let initial_len = self.cars.len();
        self.cars.retain(|car| car.uuid != car_uuid);
        
        if self.cars.len() == initial_len {
            return Err("Car not found".to_string());
        }
        
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_pilot(&mut self, pilot: Pilot) {
        self.pilots.push(pilot);
        self.updated_at = Utc::now();
    }

    pub fn remove_pilot(&mut self, pilot_uuid: Uuid) -> Result<(), String> {
        if self.pilots.len() <= 1 {
            return Err("Player must have at least one pilot".to_string());
        }

        let initial_len = self.pilots.len();
        self.pilots.retain(|pilot| pilot.uuid != pilot_uuid);
        
        if self.pilots.len() == initial_len {
            return Err("Pilot not found".to_string());
        }
        
        self.updated_at = Utc::now();
        Ok(())
    }
}

impl WalletAddress {
    pub fn parse(s: &str) -> Result<WalletAddress, String> {
        let trimmed = s.trim();
        
        // Basic Solana wallet address validation
        if trimmed.is_empty() {
            return Err("Wallet address cannot be empty".to_string());
        }
        
        if trimmed.len() < 32 || trimmed.len() > 44 {
            return Err("Invalid wallet address length".to_string());
        }
        
        // Check if it contains only valid base58 characters
        let valid_chars = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
        if !trimmed.chars().all(|c| valid_chars.contains(c)) {
            return Err("Wallet address contains invalid characters".to_string());
        }
        
        Ok(Self(trimmed.to_string()))
    }
}

impl AsRef<str> for WalletAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TeamName {
    pub fn parse(s: &str) -> Result<TeamName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 50;
        let is_too_short = s.graphemes(true).count() < 2;
        let forbidden_characters = ['<', '>', '"', '\'', '&', '\n', '\r', '\t'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace {
            Err("Team name cannot be empty".to_string())
        } else if is_too_short {
            Err("Team name must be at least 2 characters long".to_string())
        } else if is_too_long {
            Err("Team name cannot be longer than 50 characters".to_string())
        } else if contains_forbidden_characters {
            Err("Team name contains forbidden characters".to_string())
        } else {
            Ok(Self(s.trim().to_string()))
        }
    }
}

impl AsRef<str> for TeamName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}