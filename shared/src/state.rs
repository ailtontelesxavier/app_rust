use std::sync::Arc;
use sqlx::PgPool;
use minijinja::Environment;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
    pub templates: Arc<Environment<'static>>,
}

pub type SharedState = Arc<AppState>;