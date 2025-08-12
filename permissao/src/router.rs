use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::view;
use shared::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/index", get(view::home))
        .route("/saudacao", get(view::saudacao))
        .route("/modulo-api", get(view::modules_list_api))
        .route("/modulo", get(view::list_modules))
        .route("/modulo-list", get(view::list_modulo))
        .route(
            "/modulo-form",
            get(view::show_module_form).post(view::create_model),
        )
        .route(
            "/modulo-form/{id}",
            get(view::get_modulo).post(view::update_modulo),
        )
        .route("/modulo/{id}", delete(view::delete_module))
        .route("/permission", get(view::list_permissions))
        .route(
            "/permission-form",
            get(view::show_permission_form).post(view::create_permission),
        )
        .route(
            "/permission-form/{id}",
            get(view::get_permission).post(view::update_permission),
        )
        .route("/permission/{id}", delete(view::delete_permission))
}
