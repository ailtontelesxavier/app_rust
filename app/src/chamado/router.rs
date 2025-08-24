use axum::{
    Router,
    routing::{delete, get, post},
};

use shared::SharedState;

use crate::chamado::view;

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(router_tipo())
        .merge(router_categoria())
        .merge(router_servico())
}

fn router_tipo() -> Router<SharedState> {
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

fn router_categoria() -> Router<SharedState> {
    Router::new()
        .route("/categoria", get(view::list_categoria))
        .route(
            "/categoria-form",
            get(view::show_categoria_form).post(view::create_categoria),
        )
        .route(
            "/categoria-form/{id}",
            get(view::get_categoria).post(view::update_categoria),
        )
        .route("/categoria/{id}", delete(view::delete_categoria))
}

fn router_servico() -> Router<SharedState> {
    Router::new()
        .route("/servico", get(view::list_servico))
        .route(
            "/servico-form",
            get(view::show_servico_form).post(view::create_servico),
        )
        .route(
            "/servico-form/{id}",
            get(view::get_servico).post(view::update_servico),
        )
        .route("/servico/{id}", delete(view::delete_servico))
}
