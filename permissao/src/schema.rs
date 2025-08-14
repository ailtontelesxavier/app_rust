use std::net::IpAddr;

use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Deserialize)]
pub struct UserCreateSchema {
    pub username: String,
    pub password: String,
    pub email: String,
    pub full_name: String,
    pub otp_base32: Option<String>,
    pub is_active: bool,
    pub is_staff: bool,
    pub is_superuser: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserUpdateSchema {
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub otp_base32: Option<String>,
    pub is_active: Option<bool>,
    pub is_staff: Option<bool>,
    pub is_superuser: Option<bool>,
    pub ip_last_login: Option<IpNetwork>,
}
