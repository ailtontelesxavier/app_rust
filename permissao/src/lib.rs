mod router;
mod handler;


pub use shared::AppState;
use shared::SharedState;

pub use handler::home;

use axum::{
    routing::{get, post},
    Router,
};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/index", get(handler::home))
        .route("/saudacao", get(handler::saudacao))
}