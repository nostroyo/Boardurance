use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{Body, Car, Engine, HashedPassword, Password, Pilot, UserRole};

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Player {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    #[schema(value_type = Option<String>)]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    #[serde(with = "uuid_as_string")]
    #[schema(value_type = String, format = "uuid")]
    pub uuid: Uuid,
    pub email: Email,
    pub password_hash: HashedPassword,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wallet_address: Option<WalletAddress>,
    pub team_name: TeamName,
    pub role: UserRole,
    pub cars: Vec<Car>,
    pub pilots: Vec<Pilot>,
    pub engines: Vec<Engine>,
    pub bodies: Vec<Body>,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
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

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema, PartialEq)]
pub struct Email(String);

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TeamName(String);

impl Player {
    pub fn new(
        email: Email,
        password_hash: HashedPassword,
        team_name: TeamName,
        cars: Vec<Car>,
        pilots: Vec<Pilot>,
    ) -> Result<Self, String> {
        let now = Utc::now();
        Ok(Self {
            id: None,
            uuid: Uuid::new_v4(),
            email,
            password_hash,
            wallet_address: None,
            team_name,
            role: UserRole::default(),
            cars,
            pilots,
            engines: vec![],
            bodies: vec![],
            created_at: now,
            updated_at: now,
        })
    }

    pub fn new_with_assets(
        email: Email,
        password_hash: HashedPassword,
        team_name: TeamName,
        mut cars: Vec<Car>,
        pilots: Vec<Pilot>,
        engines: Vec<Engine>,
        bodies: Vec<Body>,
    ) -> Result<Self, String> {
        let now = Utc::now();

        let first_car = cars
            .get_mut(0)
            .ok_or("Player must have at least one car".to_string())?;
        first_car.assign_pilots(pilots.iter().take(3).map(|pilot| pilot.uuid).collect())?;
        first_car.assign_body(
            bodies
                .first()
                .ok_or("Player must have a least one body".to_owned())?
                .uuid,
        );
        first_car.assign_engine(
            engines
                .first()
                .ok_or("Player must have a least one engine".to_owned())?
                .uuid,
        );

        Ok(Self {
            id: None,
            uuid: Uuid::new_v4(),
            email,
            password_hash,
            wallet_address: None,
            team_name,
            role: UserRole::default(),
            cars,
            pilots,
            engines,
            bodies,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn validate_for_game(&self) -> Result<(), String> {
        let car = self
            .cars
            .first()
            .ok_or("Player must have at least one car".to_string())?;

        if !car.is_complete() {
            return Err("Car must be complete (have pilots, engine, and body) to play".to_string());
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
        self.wallet_address
            .as_ref()
            .map(std::convert::AsRef::as_ref)
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

    pub fn add_engine(&mut self, engine: Engine) {
        self.engines.push(engine);
        self.updated_at = Utc::now();
    }

    pub fn remove_engine(&mut self, engine_uuid: Uuid) -> Result<(), String> {
        let initial_len = self.engines.len();
        self.engines.retain(|engine| engine.uuid != engine_uuid);

        if self.engines.len() == initial_len {
            return Err("Engine not found".to_string());
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_body(&mut self, body: Body) {
        self.bodies.push(body);
        self.updated_at = Utc::now();
    }

    pub fn remove_body(&mut self, body_uuid: Uuid) -> Result<(), String> {
        let initial_len = self.bodies.len();
        self.bodies.retain(|body| body.uuid != body_uuid);

        if self.bodies.len() == initial_len {
            return Err("Body not found".to_string());
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    #[must_use]
    pub fn get_engine(&self, engine_uuid: Uuid) -> Option<&Engine> {
        self.engines
            .iter()
            .find(|engine| engine.uuid == engine_uuid)
    }

    #[must_use]
    pub fn get_body(&self, body_uuid: Uuid) -> Option<&Body> {
        self.bodies.iter().find(|body| body.uuid == body_uuid)
    }

    #[must_use]
    pub fn get_pilot(&self, pilot_uuid: Uuid) -> Option<&Pilot> {
        self.pilots.iter().find(|pilot| pilot.uuid == pilot_uuid)
    }

    /// Verify a password against the stored hash
    pub fn verify_password(&self, password: &Password) -> Result<bool, String> {
        self.password_hash.verify(password)
    }

    /// Update the password hash
    pub fn update_password(&mut self, new_password_hash: HashedPassword) {
        self.password_hash = new_password_hash;
        self.updated_at = Utc::now();
    }

    /// Check if this player has admin privileges
    #[must_use]
    pub fn is_admin(&self) -> bool {
        self.role.is_admin()
    }

    /// Check if this player can access a resource owned by another player
    #[must_use]
    pub fn can_access_resource(&self, resource_owner_uuid: Uuid) -> bool {
        self.is_admin() || self.uuid == resource_owner_uuid
    }

    /// Update the player's role (admin operation)
    pub fn update_role(&mut self, new_role: UserRole) {
        self.role = new_role;
        self.updated_at = Utc::now();
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

impl Email {
    pub fn parse(s: &str) -> Result<Email, String> {
        let trimmed = s.trim();

        if trimmed.is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        // Basic email validation
        if !trimmed.contains('@') {
            return Err("Email must contain @ symbol".to_string());
        }

        let parts: Vec<&str> = trimmed.split('@').collect();
        if parts.len() != 2 {
            return Err("Email must have exactly one @ symbol".to_string());
        }

        let local = parts[0];
        let domain = parts[1];

        if local.is_empty() {
            return Err("Email local part cannot be empty".to_string());
        }

        if domain.is_empty() {
            return Err("Email domain cannot be empty".to_string());
        }

        if !domain.contains('.') {
            return Err("Email domain must contain at least one dot".to_string());
        }

        if trimmed.len() > 254 {
            return Err("Email cannot be longer than 254 characters".to_string());
        }

        Ok(Self(trimmed.to_string()))
    }
}

impl AsRef<str> for Email {
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
