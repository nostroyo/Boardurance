use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Pilot {
    #[serde(with = "uuid_as_string")]
    #[schema(value_type = String, format = "uuid")]
    pub uuid: Uuid,
    pub nft_mint_address: Option<String>, // Solana NFT mint address
    pub name: PilotName,
    pub pilot_class: PilotClass,
    pub rarity: PilotRarity,
    pub skills: PilotSkills,
    pub performance: PilotPerformance,
    pub experience_level: u32,
    pub is_active: bool,
    #[schema(value_type = String, format = "date-time")]
    pub created_at: DateTime<Utc>,
    #[schema(value_type = String, format = "date-time")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PilotName(String);

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum PilotClass {
    Speedster,    // Focuses on speed and acceleration
    Technician,   // Focuses on handling and precision
    Endurance,    // Focuses on durability and consistency
    AllRounder,   // Balanced across all skills
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub enum PilotRarity {
    Rookie,
    Professional,
    Expert,
    Champion,
    Legend,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PilotSkills {
    pub reaction_time: u8,    // 0-10 - affects acceleration
    pub precision: u8,        // 0-10 - affects handling
    pub focus: u8,           // 0-10 - affects consistency
    pub stamina: u8,         // 0-10 - affects performance over time
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PilotPerformance {
    pub straight_value: u8,  // 0-10
    pub curve_value: u8,     // 0-10
}

impl Pilot {
    pub fn new(
        name: PilotName,
        pilot_class: PilotClass,
        rarity: PilotRarity,
        skills: PilotSkills,
        performance: PilotPerformance,
        nft_mint_address: Option<String>,
    ) -> Result<Self, String> {
        skills.validate()?;
        performance.validate()?;
        
        let now = Utc::now();
        Ok(Self {
            uuid: Uuid::new_v4(),
            nft_mint_address,
            name,
            pilot_class,
            rarity,
            skills,
            performance,
            experience_level: 1,
            is_active: false,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn activate(&mut self) {
        self.is_active = true;
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
        self.updated_at = Utc::now();
    }

    pub fn gain_experience(&mut self, amount: u32) {
        self.experience_level += amount;
        self.updated_at = Utc::now();
    }

    pub fn update_skills(&mut self, new_skills: PilotSkills) -> Result<(), String> {
        new_skills.validate()?;
        self.skills = new_skills;
        self.updated_at = Utc::now();
        Ok(())
    }

    #[must_use]
    pub fn calculate_overall_skill(&self) -> u8 {
        let total = u16::from(self.skills.reaction_time)
            + u16::from(self.skills.precision)
            + u16::from(self.skills.focus)
            + u16::from(self.skills.stamina);
        #[allow(clippy::cast_possible_truncation)]
        {
            (total / 4) as u8
        }
    }

    #[must_use]
    pub fn get_class_bonus(&self) -> PilotClassBonus {
        match self.pilot_class {
            PilotClass::Speedster => PilotClassBonus {
                speed_bonus: 2,
                acceleration_bonus: 3,
                handling_bonus: 0,
                durability_bonus: 0,
            },
            PilotClass::Technician => PilotClassBonus {
                speed_bonus: 0,
                acceleration_bonus: 1,
                handling_bonus: 3,
                durability_bonus: 1,
            },
            PilotClass::Endurance => PilotClassBonus {
                speed_bonus: 0,
                acceleration_bonus: 0,
                handling_bonus: 1,
                durability_bonus: 4,
            },
            PilotClass::AllRounder => PilotClassBonus {
                speed_bonus: 1,
                acceleration_bonus: 1,
                handling_bonus: 1,
                durability_bonus: 1,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct PilotClassBonus {
    pub speed_bonus: u8,
    pub acceleration_bonus: u8,
    pub handling_bonus: u8,
    pub durability_bonus: u8,
}

impl PilotName {
    pub fn parse(s: &str) -> Result<PilotName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 25;
        let is_too_short = s.graphemes(true).count() < 2;
        let forbidden_characters = ['<', '>', '"', '\'', '&', '\n', '\r', '\t'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace {
            Err("Pilot name cannot be empty".to_string())
        } else if is_too_short {
            Err("Pilot name must be at least 2 characters long".to_string())
        } else if is_too_long {
            Err("Pilot name cannot be longer than 25 characters".to_string())
        } else if contains_forbidden_characters {
            Err("Pilot name contains forbidden characters".to_string())
        } else {
            Ok(Self(s.trim().to_string()))
        }
    }
}

impl AsRef<str> for PilotName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl PilotSkills {
    pub fn new(reaction_time: u8, precision: u8, focus: u8, stamina: u8) -> Result<Self, String> {
        let skills = Self {
            reaction_time,
            precision,
            focus,
            stamina,
        };
        skills.validate()?;
        Ok(skills)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.reaction_time > 10 {
            return Err("Reaction time must be between 0 and 10".to_string());
        }
        if self.precision > 10 {
            return Err("Precision must be between 0 and 10".to_string());
        }
        if self.focus > 10 {
            return Err("Focus must be between 0 and 10".to_string());
        }
        if self.stamina > 10 {
            return Err("Stamina must be between 0 and 10".to_string());
        }
        Ok(())
    }
}

impl PilotPerformance {
    pub fn new(straight_value: u8, curve_value: u8) -> Result<Self, String> {
        let performance = Self { straight_value, curve_value };
        performance.validate()?;
        Ok(performance)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.straight_value > 10 {
            return Err("Pilot straight value must be between 0 and 10".to_string());
        }
        if self.curve_value > 10 {
            return Err("Pilot curve value must be between 0 and 10".to_string());
        }
        Ok(())
    }
}

impl PilotRarity {
    #[must_use]
    pub fn get_skill_multiplier(&self) -> f32 {
        match self {
            PilotRarity::Rookie => 1.0,
            PilotRarity::Professional => 1.15,
            PilotRarity::Expert => 1.3,
            PilotRarity::Champion => 1.5,
            PilotRarity::Legend => 1.8,
        }
    }

    #[must_use]
    pub fn get_max_skills(&self) -> u8 {
        match self {
            PilotRarity::Rookie => 5,
            PilotRarity::Professional => 6,
            PilotRarity::Expert => 7,
            PilotRarity::Champion => 9,
            PilotRarity::Legend => 10,
        }
    }

    #[must_use]
    pub fn get_experience_multiplier(&self) -> f32 {
        match self {
            PilotRarity::Rookie => 1.0,
            PilotRarity::Professional => 1.2,
            PilotRarity::Expert => 1.4,
            PilotRarity::Champion => 1.6,
            PilotRarity::Legend => 2.0,
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