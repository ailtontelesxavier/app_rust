use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use minijinja::{path_loader, Environment};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tower_http::trace::TraceLayer;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::{fmt, sync::Arc};
use tower_http::services::ServeDir;
use tracing::{info, debug};

use std::fs;
use std::path::Path;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use dotenv::dotenv;

use permissao::{router as router_permissao};


use shared::{AppState, SharedState};

const SECRET: &[u8] = b"super-secret-key";

#[derive(Debug)]
enum AppError {
    InvalidInput(String),
    InternalServerError,
    NotFound,
}

// Implement IntoResponse for AppError
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Resource Not Found".to_string()),
        };

        (status, error_message).into_response()
    }
}

async fn hello_world() -> &'static str {
    "Welcome!"
}

#[tokio::main]
async fn main() {

    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Carrega os templates
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));

    let templates = Arc::new(env);

    let state = Arc::new(AppState { 
        db: Arc::new(db_pool),
        templates
    });

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info,debug,tower_http=debug"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2000").await.unwrap();

    let app = Router::new()
        .route("/hello", get(hello_world))
        .nest("/permissao", router_permissao().with_state(state.clone()))
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());


    info!("Starting server on http://0.0.0.0:2000");
    debug!("Server running");
    //println!("Server running on http://0.0.0.0:2000");
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soma() {
        assert_eq!(1 + 1, 2);
    }
}