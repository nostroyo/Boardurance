use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Engine {
    #[serde(with = "uuid_as_string")]
    pub uuid: Uuid,
    pub nft_mint_address: Option<String>, // Solana NFT mint address
    pub name: EngineName,
    pub rarity: ComponentRarity,
    pub straight_value: u8,  // 1-100
    pub curve_value: u8,     // 1-100
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EngineName(String);

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum ComponentRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Engine {
    pub fn new(
        name: EngineName,
        rarity: ComponentRarity,
        straight_value: u8,
        curve_value: u8,
        nft_mint_address: Option<String>,
    ) -> Result<Self, String> {
        if straight_value == 0 || straight_value > 100 {
            return Err("Engine straight value must be between 1 and 100".to_string());
        }
        if curve_value == 0 || curve_value > 100 {
            return Err("Engine curve value must be between 1 and 100".to_string());
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
        if straight_value == 0 || straight_value > 100 {
            return Err("Engine straight value must be between 1 and 100".to_string());
        }
        if curve_value == 0 || curve_value > 100 {
            return Err("Engine curve value must be between 1 and 100".to_string());
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

impl EngineName {
    pub fn parse(s: &str) -> Result<EngineName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 30;
        let is_too_short = s.graphemes(true).count() < 1;
        let forbidden_characters = ['<', '>', '"', '\'', '&', '\n', '\r', '\t'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace {
            Err("Engine name cannot be empty".to_string())
        } else if is_too_short {
            Err("Engine name must be at least 1 character long".to_string())
        } else if is_too_long {
            Err("Engine name cannot be longer than 30 characters".to_string())
        } else if contains_forbidden_characters {
            Err("Engine name contains forbidden characters".to_string())
        } else {
            Ok(Self(s.trim().to_string()))
        }
    }
}

impl AsRef<str> for EngineName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ComponentRarity {
    #[must_use]
    pub fn get_value_multiplier(&self) -> f32 {
        match self {
            ComponentRarity::Common => 1.0,
            ComponentRarity::Uncommon => 1.1,
            ComponentRarity::Rare => 1.25,
            ComponentRarity::Epic => 1.5,
            ComponentRarity::Legendary => 2.0,
        }
    }

    #[must_use]
    pub fn get_max_values(&self) -> u8 {
        match self {
            ComponentRarity::Common => 70,
            ComponentRarity::Uncommon => 75,
            ComponentRarity::Rare => 85,
            ComponentRarity::Epic => 95,
            ComponentRarity::Legendary => 100,
        }
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