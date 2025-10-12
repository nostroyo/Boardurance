use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Car {
    #[serde(with = "uuid_as_string")]
    pub uuid: Uuid,
    pub nft_mint_address: Option<String>, // Solana NFT mint address
    pub name: CarName,
    pub car_type: CarType,
    pub rarity: CarRarity,
    pub stats: CarStats,
    pub performance: CarPerformance,
    pub is_equipped: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CarName(String);

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum CarType {
    Sports,
    Racing,
    Luxury,
    Electric,
    Vintage,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum CarRarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CarStats {
    pub speed: u8,        // 1-100
    pub acceleration: u8, // 1-100
    pub handling: u8,     // 1-100
    pub durability: u8,   // 1-100
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct CarPerformance {
    pub engine: EngineStats,
    pub body: BodyStats,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct EngineStats {
    pub straight_value: u8,  // 1-100
    pub curve_value: u8,     // 1-100
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct BodyStats {
    pub straight_value: u8,  // 1-100
    pub curve_value: u8,     // 1-100
}

impl Car {
    pub fn new(
        name: CarName,
        car_type: CarType,
        rarity: CarRarity,
        stats: CarStats,
        performance: CarPerformance,
        nft_mint_address: Option<String>,
    ) -> Result<Self, String> {
        stats.validate()?;
        performance.validate()?;
        
        let now = Utc::now();
        Ok(Self {
            uuid: Uuid::new_v4(),
            nft_mint_address,
            name,
            car_type,
            rarity,
            stats,
            performance,
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

    pub fn update_stats(&mut self, new_stats: CarStats) -> Result<(), String> {
        new_stats.validate()?;
        self.stats = new_stats;
        self.updated_at = Utc::now();
        Ok(())
    }

    #[must_use]
    pub fn calculate_overall_rating(&self) -> u8 {
        let total = u16::from(self.stats.speed)
            + u16::from(self.stats.acceleration)
            + u16::from(self.stats.handling)
            + u16::from(self.stats.durability);
        #[allow(clippy::cast_possible_truncation)]
        {
            (total / 4) as u8
        }
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

impl CarStats {
    pub fn new(speed: u8, acceleration: u8, handling: u8, durability: u8) -> Result<Self, String> {
        let stats = Self {
            speed,
            acceleration,
            handling,
            durability,
        };
        stats.validate()?;
        Ok(stats)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.speed == 0 || self.speed > 100 {
            return Err("Speed must be between 1 and 100".to_string());
        }
        if self.acceleration == 0 || self.acceleration > 100 {
            return Err("Acceleration must be between 1 and 100".to_string());
        }
        if self.handling == 0 || self.handling > 100 {
            return Err("Handling must be between 1 and 100".to_string());
        }
        if self.durability == 0 || self.durability > 100 {
            return Err("Durability must be between 1 and 100".to_string());
        }
        Ok(())
    }
}

impl CarPerformance {
    pub fn new(engine: EngineStats, body: BodyStats) -> Result<Self, String> {
        let performance = Self { engine, body };
        performance.validate()?;
        Ok(performance)
    }

    pub fn validate(&self) -> Result<(), String> {
        self.engine.validate()?;
        self.body.validate()?;
        Ok(())
    }
}

impl EngineStats {
    pub fn new(straight_value: u8, curve_value: u8) -> Result<Self, String> {
        let stats = Self { straight_value, curve_value };
        stats.validate()?;
        Ok(stats)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.straight_value == 0 || self.straight_value > 100 {
            return Err("Engine straight value must be between 1 and 100".to_string());
        }
        if self.curve_value == 0 || self.curve_value > 100 {
            return Err("Engine curve value must be between 1 and 100".to_string());
        }
        Ok(())
    }
}

impl BodyStats {
    pub fn new(straight_value: u8, curve_value: u8) -> Result<Self, String> {
        let stats = Self { straight_value, curve_value };
        stats.validate()?;
        Ok(stats)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.straight_value == 0 || self.straight_value > 100 {
            return Err("Body straight value must be between 1 and 100".to_string());
        }
        if self.curve_value == 0 || self.curve_value > 100 {
            return Err("Body curve value must be between 1 and 100".to_string());
        }
        Ok(())
    }
}

impl CarRarity {
    #[must_use]
    pub fn get_stat_multiplier(&self) -> f32 {
        match self {
            CarRarity::Common => 1.0,
            CarRarity::Uncommon => 1.1,
            CarRarity::Rare => 1.25,
            CarRarity::Epic => 1.5,
            CarRarity::Legendary => 2.0,
        }
    }

    #[must_use]
    pub fn get_max_stats(&self) -> u8 {
        match self {
            CarRarity::Common => 70,
            CarRarity::Uncommon => 75,
            CarRarity::Rare => 85,
            CarRarity::Epic => 95,
            CarRarity::Legendary => 100,
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