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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "moduleId")]
    pub module_id: i32,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

// Estrutura para relacionamento com Module
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PermissionWithModule {
    #[serde(flatten)]
    #[sqlx(flatten)]
    pub permission: Permission,
    #[sqlx(flatten)]
    pub module: Module,
}

//role
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Perfil {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip_serializing)] // nunca aparece no JSON utilizado na api
    pub password: String,
    pub email: String,
    pub full_name: String,
    #[serde(skip_serializing)] // nunca aparece no JSON utilizado na api
    pub otp_base32: Option<String>,
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
    pub ip_last_login: Option<String>,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRoles {
    pub id: i32,
    #[serde(rename = "user_id")]
    pub user_id: i32,
    #[serde(rename = "role_id")]
    pub role_id: i32,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RolePermission {
    pub id: i64,
    pub role_id: i32,
    pub permission_id: i32,
}
