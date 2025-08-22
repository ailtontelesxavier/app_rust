use minijinja::Environment;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
    pub templates: Arc<Environment<'static>>,
    pub message: Arc<MessageResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub status: String,
    pub message: String,
}

pub type SharedState = Arc<AppState>;
