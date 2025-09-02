use axum::{Router, routing::{get, post}};
use shared::SharedState;

use crate::externo::view;

pub fn router() -> Router<SharedState> {
    Router::new().merge(router_tipo())
}

fn router_tipo() -> Router<SharedState> {
    Router::new().route("/linha", get(view::list_linha)).route(
        "/linha-form",
        get(view::linha_form),
    )
    .route(
        "/linha-form",
        post(view::create_linha),
    )
    /*.route(
        "/linha-form/{id}",
        get(view::get_tipo).post(view::update_tipo),
    )
    .route("/linha/{id}", delete(view::delete_tipo)) */
}
