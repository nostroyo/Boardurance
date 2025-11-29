use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use unicode_segmentation::UnicodeSegmentation;

use super::engine::ComponentRarity;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Body {
    #[serde(with = "uuid_as_string")]
    #[schema(value_type = String, format = "uuid")]
    pub uuid: Uuid,
    pub nft_mint_address: Option<String>, // Solana NFT mint address
    pub name: BodyName,
    pub rarity: ComponentRarity,
    pub straight_value: u8,  // 0-10
    pub curve_value: u8,     // 0-10
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BodyName(String);

impl Body {
    pub fn new(
        name: BodyName,
        rarity: ComponentRarity,
        straight_value: u8,
        curve_value: u8,
        nft_mint_address: Option<String>,
    ) -> Result<Self, String> {
        if straight_value > 10 {
            return Err("Body straight value must be between 0 and 10".to_string());
        }
        if curve_value > 10 {
            return Err("Body curve value must be between 0 and 10".to_string());
        }
        
        let now = Utc::now();
        Ok(Self {
            uuid: Uuid::new_v4(),
            nft_mint_address,
            name,
            rarity,
            straight_value,
            curve_value,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn update_values(&mut self, straight_value: u8, curve_value: u8) -> Result<(), String> {
        if straight_value > 10 {
            return Err("Body straight value must be between 0 and 10".to_string());
        }
        if curve_value > 10 {
            return Err("Body curve value must be between 0 and 10".to_string());
        }
        
        self.straight_value = straight_value;
        self.curve_value = curve_value;
        self.updated_at = Utc::now();
        Ok(())
    }

    #[must_use]
    pub fn calculate_overall_rating(&self) -> u8 {
        let total = u16::from(self.straight_value) + u16::from(self.curve_value);
        #[allow(clippy::cast_possible_truncation)]
        {
            (total / 2) as u8
        }
    }
}

impl BodyName {
    pub fn parse(s: &str) -> Result<BodyName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 30;
        let is_too_short = s.graphemes(true).count() < 1;
        let forbidden_characters = ['<', '>', '"', '\'', '&', '\n', '\r', '\t'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace {
            Err("Body name cannot be empty".to_string())
        } else if is_too_short {
            Err("Body name must be at least 1 character long".to_string())
        } else if is_too_long {
            Err("Body name cannot be longer than 30 characters".to_string())
        } else if contains_forbidden_characters {
            Err("Body name contains forbidden characters".to_string())
        } else {
            Ok(Self(s.trim().to_string()))
        }
    }
}

impl AsRef<str> for BodyName {
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