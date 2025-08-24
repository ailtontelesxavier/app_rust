use axum::{
    Router,
    routing::{delete, get, post},
};

use shared::SharedState;

use crate::chamado::view;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/tipo", get(view::list_tipo_chamado))
        .route(
            "/tipo-form",
            get(view::show_tipo_form).post(view::create_tipo),
        )
        .route(
            "/tipo-form/{id}",
            get(view::get_tipo).post(view::update_tipo),
        )
        .route("/tipo/{id}", delete(view::delete_tipo))
}
