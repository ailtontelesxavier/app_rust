use axum::{
    Router,
    routing::{get, post},
};

use shared::SharedState;

use crate::view::{self, get_modulo};

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/index", get(view::home).post(view::create_model))
        .route("/saudacao", get(view::saudacao))
        .route("/lista", get(view::list_modules))
        .route("/modulo", get(view::list_modulo))
        .route("/modulo-form", get(view::saudacao))
        .route("/modulo-form/{id}", get(view::get_modulo))
}
