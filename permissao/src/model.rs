use sqlx::FromRow;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
//use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Module {
    pub id: i32,
    pub title: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}