use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use mongodb::{bson::doc, Database};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::domain::{TestItem, TestItemDescription, TestItemName};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTestItemRequest {
    pub name: String,
    pub description: Option<String>,
}

pub fn routes() -> Router<Database> {
    Router::new()
        .route("/test", post(create_test_item))
        .route("/test", get(get_test_items))
}

/// Create a new test item
#[utoipa::path(
    post,
    path = "/api/v1/test",
    request_body = CreateTestItemRequest,
    responses(
        (status = 201, description = "Test item created successfully", body = TestItem),
        (status = 400, description = "Bad request"),
        (status = 500, description = "Internal server error")
    ),
    tag = "test"
)]
#[tracing::instrument(
    name = "Adding a new test item",
    skip(database, payload),
    fields(
        test_item_name = %payload.name,
        test_item_description = payload.description.as_deref().unwrap_or("None")
    )
)]
pub async fn create_test_item(
    State(database): State<Database>,
    Json(payload): Json<CreateTestItemRequest>,
) -> Result<(StatusCode, Json<TestItem>), StatusCode> {
    let name = match TestItemName::parse(payload.name) {
        Ok(name) => name,
        Err(e) => {
            tracing::warn!("Invalid test item name: {}", e);
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    let description = match payload.description {
        Some(desc) => match TestItemDescription::parse(desc) {
            Ok(description) => Some(description),
            Err(e) => {
                tracing::warn!("Invalid test item description: {}", e);
                return Err(StatusCode::BAD_REQUEST);
            }
        },
        None => None,
    };

    let test_item = TestItem::new(name, description);

    match insert_test_item(&database, &test_item).await {
        Ok(created_item) => {
            tracing::info!("Test item created successfully");
            Ok((StatusCode::CREATED, Json(created_item)))
        }
        Err(e) => {
            tracing::error!("Failed to create test item: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get all test items
#[utoipa::path(
    get,
    path = "/api/v1/test",
    responses(
        (status = 200, description = "List of test items", body = Vec<TestItem>),
        (status = 500, description = "Internal server error")
    ),
    tag = "test"
)]
#[tracing::instrument(name = "Fetching all test items", skip(database))]
pub async fn get_test_items(State(database): State<Database>) -> Result<Json<Vec<TestItem>>, StatusCode> {
    match get_all_test_items(&database).await {
        Ok(items) => {
            tracing::info!("Successfully fetched {} test items", items.len());
            Ok(Json(items))
        }
        Err(e) => {
            tracing::error!("Failed to fetch test items: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[tracing::instrument(name = "Saving new test item in the database", skip(database, test_item))]
pub async fn insert_test_item(
    database: &Database,
    test_item: &TestItem,
) -> Result<TestItem, mongodb::error::Error> {
    let collection = database.collection::<TestItem>("test_items");
    let result = collection.insert_one(test_item, None).await?;
    
    let mut created_item = test_item.clone();
    created_item.id = Some(result.inserted_id.as_object_id().unwrap());
    Ok(created_item)
}

#[tracing::instrument(name = "Getting all test items from the database", skip(database))]
pub async fn get_all_test_items(database: &Database) -> Result<Vec<TestItem>, mongodb::error::Error> {
    let collection = database.collection::<TestItem>("test_items");
    let mut cursor = collection.find(None, None).await?;
    
    let mut items = Vec::new();
    while cursor.advance().await? {
        let item = cursor.deserialize_current()?;
        items.push(item);
    }
    
    Ok(items)
}