use minijinja::Environment;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<PgPool>,
    pub templates: Arc<Environment<'static>>,
}

pub type SharedState = Arc<AppState>;

#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub is_authenticated: bool,
    pub user_id: Option<i32>,
}
