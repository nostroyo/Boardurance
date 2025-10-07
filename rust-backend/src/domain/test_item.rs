use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TestItem {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub uuid: Uuid,
    pub name: TestItemName,
    pub description: Option<TestItemDescription>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TestItemName(String);

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct TestItemDescription(String);

impl TestItem {
    pub fn new(name: TestItemName, description: Option<TestItemDescription>) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            uuid: Uuid::new_v4(),
            name,
            description,
            created_at: now,
            updated_at: now,
        }
    }
}

impl TestItemName {
    pub fn parse(s: String) -> Result<TestItemName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

        if is_empty_or_whitespace {
            Err("Name cannot be empty".to_string())
        } else if is_too_long {
            Err("Name is too long".to_string())
        } else if contains_forbidden_characters {
            Err("Name contains forbidden characters".to_string())
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for TestItemName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TestItemDescription {
    pub fn parse(s: String) -> Result<TestItemDescription, String> {
        let is_too_long = s.graphemes(true).count() > 1000;

        if is_too_long {
            Err("Description is too long".to_string())
        } else {
            Ok(Self(s))
        }
    }
}

impl AsRef<str> for TestItemDescription {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

use unicode_segmentation::UnicodeSegmentation;