use serde::{Deserialize, Serialize};

use regex::Regex;
use std::{sync::LazyLock};
use validator::Validate;

static EMAIL_RX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateModuleSchema {
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateModuleSchema {
    pub title: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PermissionCreateSchema {
    pub name: String,
    pub description: Option<String>,
    pub module_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PermissionUpdateSchema {
    pub name: String,
    pub description: Option<String>,
    pub module_id: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PerfilCreateSchema {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PerfilUpdateSchema {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserCreateSchema {
    pub username: String,
    #[validate(length(min = 8, message = "Password must have at least 8 characters"))]
    pub password: Option<String>,
    #[validate(regex(path=*EMAIL_RX, message = "Invalid email"))]
    pub email: String,
    pub full_name: String,
    pub otp_base32: Option<String>,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_staff: bool,
    #[serde(default)]
    pub is_superuser: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UserUpdateSchema {
    pub username: Option<String>,
    #[validate(regex(path=*EMAIL_RX, message = "Invalid email"))]
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub otp_base32: Option<String>,
    pub is_active: Option<bool>,
    pub is_staff: Option<bool>,
    pub is_superuser: Option<bool>,
    pub ip_last_login: Option<String>,
}

#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct UserPasswordUpdateDtoSchema {
    #[validate(
        length(min = 6, message = "new password must be at least 6 characters")
    )]
    pub new_password: String,

    #[validate(
        length(min = 6, message = "new password confirm must be at least 6 characters"),
        must_match(other = "new_password", message="new passwords do not match")
    )]
    pub new_password_confirm: String,

    #[validate(
        length(min = 6, message = "Old password must be at least 6 characters")
    )]
    pub old_password: String,
}

#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct UserPasswordUpdateSchema {
    #[validate(
        length(min = 6, message = "new password must be at least 6 characters")
    )]
    pub password: String,
}

/* 

fn checkbox_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(s.as_deref() == Some("on") || s.as_deref() == Some("true"))
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UserCreateSchema {
    pub username: String,
    #[validate(length(min = 8, message = "Password must have at least 8 characters"))]
    pub password: String,
    #[validate(regex(path=*EMAIL_RX, message = "Invalid email"))]
    pub email: String,
    pub full_name: String,
    pub otp_base32: Option<String>,
    #[serde(deserialize_with = "checkbox_bool")]
    pub is_active: bool,
    #[serde(deserialize_with = "checkbox_bool")]
    pub is_staff: bool,
    #[serde(deserialize_with = "checkbox_bool")]
    pub is_superuser: bool,
}

*/