mod filters;
mod middlewares;
use axum::{
    http::{header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method, StatusCode}, middleware, response::{IntoResponse, Response}, routing::get, Router
};
use axum_flash::{Flash, IncomingFlashes, Key, Message};
use filters::register_filters;
use minijinja::{Environment, path_loader};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tower_http::services::{ServeDir, ServeFile};
use tower_http::cors::CorsLayer;

use sqlx::postgres::PgPoolOptions;

use dotenv::dotenv;

use permissao::router as router_permissao;

use shared::{AppState, SharedState};
use middlewares::method_override;

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
    // Crie o ambiente MiniJinja
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    // Registre os filtros
    register_filters(&mut env);

    let templates = Arc::new(env);

    let key = Key::generate();

    let state = Arc::new(AppState {
        db: Arc::new(db_pool),
        templates,
    });

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "info,debug,tower_http=debug",
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2000").await.unwrap();

    let server_dir = ServeDir::new("static");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:2000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = Router::new()
        .layer(cors)
        .route("/hello", get(hello_world))
        .nest("/permissao", router_permissao().with_state(state.clone()))
        //.layer(middleware::from_fn(method_override))
        .nest_service("/static", server_dir)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone())
        .layer(Flash::layer(key));

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
