use axum::{
    routing::{delete, get, post}, Router
};

use shared::SharedState;

use crate::chamado::view;

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(router_tipo())
        .merge(router_categoria())
        .merge(router_servico())
        .merge(router_chamado())
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
        .merge(api_tipo_router())
}

fn api_tipo_router() -> Router<SharedState> {
    Router::new().route("/tipo-api", get(view::tipo_list_api))
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
        .merge(api_servico_router())
}

fn api_servico_router() -> Router<SharedState> {
    Router::new().route("/servico-api", get(view::servico_list_api))
}

fn router_chamado() -> Router<SharedState> {
    Router::new()
        .route("/chamado", get(view::list_chamado))
        .route(
            "/chamado-form",
            get(view::show_chamado_form).post(view::create_chamado),
        )
        .route(
            "/chamado-form/{id}",
            get(view::get_chamado).post(view::update_chamado),
        )
        .route(
            "/chamado-upload-imagem/{id_chamado}",
            post(view::upload_imagem),
        )
        .route("/chamado/{id}", delete(view::delete_chamado))
}
