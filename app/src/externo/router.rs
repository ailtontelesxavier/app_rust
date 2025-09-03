use axum::{
    Router,
    routing::{delete, get, post},
};
use shared::SharedState;

use crate::externo::view;

pub fn router() -> Router<SharedState> {
    Router::new().merge(router_tipo()).merge(router_contato())
}

fn router_tipo() -> Router<SharedState> {
    Router::new()
        .route("/linha", get(view::list_linha))
        .route("/linha-form", get(view::linha_form))
        .route("/linha-form", post(view::create_linha))
        .route(
            "/linha-form/{id}",
            get(view::get_linha).post(view::update_linha),
        )
        .route("/linha/{id}", delete(view::delete_linha))
}

fn router_contato() -> Router<SharedState> {
    Router::new()
     .route("/contato", get(view::list_contato))
    /*.route("/contato-form", get(view::contato_form))
    .route("/contato-form", post(view::create_contato))
    .route(
        "/contato-form/{id}",
        get(view::get_contato).post(view::update_contato),
    )
    .route("/contato/{id}", delete(view::delete_contato)) */
}
