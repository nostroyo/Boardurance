use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use unicode_segmentation::UnicodeSegmentation;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Car {
    #[serde(with = "uuid_as_string")]
    #[schema(value_type = String, format = "uuid")]
    pub uuid: Uuid,
    pub nft_mint_address: Option<String>, // Solana NFT mint address
    pub name: CarName,
    #[schema(value_type = Vec<String>, format = "uuid")]
    pub pilot_uuids: Vec<Uuid>, // Assigned pilots (exactly 3 required)
    #[schema(value_type = Option<String>, format = "uuid")]
    pub engine_uuid: Option<Uuid>, // Assigned engine
    #[schema(value_type = Option<String>, format = "uuid")]
    pub body_uuid: Option<Uuid>, // Assigned body
    pub is_equipped: bool,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CarName(String);

impl Car {
    pub fn new(name: CarName, nft_mint_address: Option<String>) -> Result<Self, String> {
        let now = Utc::now();
        Ok(Self {
            uuid: Uuid::new_v4(),
            nft_mint_address,
            name,
            pilot_uuids: Vec::new(),
            engine_uuid: None,
            body_uuid: None,
            is_equipped: false,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn equip(&mut self) {
        self.is_equipped = true;
        self.updated_at = Utc::now();
    }

    pub fn unequip(&mut self) {
        self.is_equipped = false;
        self.updated_at = Utc::now();
    }

    pub fn assign_pilots(&mut self, pilot_uuids: Vec<Uuid>) -> Result<(), String> {
        if pilot_uuids.len() != 3 {
            return Err(format!(
                "Car must have exactly 3 pilots, got {}",
                pilot_uuids.len()
            ));
        }

        // Check for duplicate pilots
        let mut unique_pilots = pilot_uuids.clone();
        unique_pilots.sort();
        unique_pilots.dedup();
        if unique_pilots.len() != 3 {
            return Err("All 3 pilots must be unique".to_string());
        }

        self.pilot_uuids = pilot_uuids;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn add_pilot(&mut self, pilot_uuid: Uuid) -> Result<(), String> {
        if self.pilot_uuids.len() >= 3 {
            return Err("Car already has maximum of 3 pilots".to_string());
        }

        if self.pilot_uuids.contains(&pilot_uuid) {
            return Err("Pilot is already assigned to this car".to_string());
        }

        self.pilot_uuids.push(pilot_uuid);
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn remove_pilot(&mut self, pilot_uuid: Uuid) -> Result<(), String> {
        let initial_len = self.pilot_uuids.len();
        self.pilot_uuids.retain(|&uuid| uuid != pilot_uuid);

        if self.pilot_uuids.len() == initial_len {
            return Err("Pilot not found in car".to_string());
        }

        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn clear_pilots(&mut self) {
        self.pilot_uuids.clear();
        self.updated_at = Utc::now();
    }

    pub fn validate_pilots(&self) -> Result<(), String> {
        if self.pilot_uuids.len() != 3 {
            return Err(format!(
                "Car must have exactly 3 pilots, currently has {}",
                self.pilot_uuids.len()
            ));
        }

        // Check for duplicate pilots
        let mut unique_pilots = self.pilot_uuids.clone();
        unique_pilots.sort();
        unique_pilots.dedup();
        if unique_pilots.len() != 3 {
            return Err("All 3 pilots must be unique".to_string());
        }

        Ok(())
    }

    pub fn assign_engine(&mut self, engine_uuid: Uuid) {
        self.engine_uuid = Some(engine_uuid);
        self.updated_at = Utc::now();
    }

    pub fn unassign_engine(&mut self) {
        self.engine_uuid = None;
        self.updated_at = Utc::now();
    }

    pub fn assign_body(&mut self, body_uuid: Uuid) {
        self.body_uuid = Some(body_uuid);
        self.updated_at = Utc::now();
    }

    pub fn unassign_body(&mut self) {
        self.body_uuid = None;
        self.updated_at = Utc::now();
    }

    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.pilot_uuids.len() == 3 && self.engine_uuid.is_some() && self.body_uuid.is_some()
    }

    #[must_use]
    pub fn is_ready_for_race(&self) -> bool {
        self.is_complete() && self.validate_pilots().is_ok()
    }

    #[must_use]
    pub fn get_pilot_count(&self) -> usize {
        self.pilot_uuids.len()
    }

    #[must_use]
    pub fn has_pilot(&self, pilot_uuid: Uuid) -> bool {
        self.pilot_uuids.contains(&pilot_uuid)
    }
}

impl CarName {
    pub fn parse(s: &str) -> Result<CarName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 30;
        let is_too_short = s.graphemes(true).count() < 1;
        let forbidden_characters = ['<', '>', '"', '\'', '&', '\n', '\r', '\t'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace {
            Err("Car name cannot be empty".to_string())
        } else if is_too_short {
            Err("Car name must be at least 1 character long".to_string())
        } else if is_too_long {
            Err("Car name cannot be longer than 30 characters".to_string())
        } else if contains_forbidden_characters {
            Err("Car name contains forbidden characters".to_string())
        } else {
            Ok(Self(s.trim().to_string()))
        }
    }
}

impl AsRef<str> for CarName {
    fn as_ref(&self) -> &str {
        &self.0
    }
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
