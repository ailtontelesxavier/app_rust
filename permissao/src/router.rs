use axum::{
    Router,
    routing::{get, post},
};

use shared::SharedState;

use crate::view;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/index", get(view::home))
        .route("/saudacao", get(view::saudacao))
        .route("/lista", get(view::list_modules))
        .route("/modulo", get(view::list_modulo))
        .route(
            "/modulo-form",
            get(view::show_module_form).post(view::create_model),
        )
        .route(
            "/modulo-form/{id}",
            get(view::get_modulo).post(view::update_modulo),
        )
}
