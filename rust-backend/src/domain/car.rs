use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Car {
    #[serde(with = "uuid_as_string")]
    #[schema(value_type = String, format = "uuid")]
    pub uuid: Uuid,
    pub nft_mint_address: Option<String>, // Solana NFT mint address
    pub name: CarName,
    #[schema(value_type = Option<String>, format = "uuid")]
    pub pilot_uuid: Option<Uuid>,  // Assigned pilot
    #[schema(value_type = Option<String>, format = "uuid")]
    pub engine_uuid: Option<Uuid>, // Assigned engine
    #[schema(value_type = Option<String>, format = "uuid")]
    pub body_uuid: Option<Uuid>,   // Assigned body
    pub is_equipped: bool,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CarName(String);



impl Car {
    pub fn new(
        name: CarName,
        nft_mint_address: Option<String>,
    ) -> Result<Self, String> {
        let now = Utc::now();
        Ok(Self {
            uuid: Uuid::new_v4(),
            nft_mint_address,
            name,
            pilot_uuid: None,
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

    pub fn assign_pilot(&mut self, pilot_uuid: Uuid) {
        self.pilot_uuid = Some(pilot_uuid);
        self.updated_at = Utc::now();
    }

    pub fn unassign_pilot(&mut self) {
        self.pilot_uuid = None;
        self.updated_at = Utc::now();
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
        self.pilot_uuid.is_some() && self.engine_uuid.is_some() && self.body_uuid.is_some()
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