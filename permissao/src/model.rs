use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
//use uuid::Uuid;
use regex::Regex;
use std::sync::LazyLock;
//use validator::Validate;

static EMAIL_RX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Module {
    pub id: i32,
    pub title: String,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

/*
#[derive(Deserialize, Validate)]
pub struct AuthFormModel {
    #[validate(regex(path=*EMAIL_RX, message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must have at least 8 characters"))]
    pub password: String,
}
*/
