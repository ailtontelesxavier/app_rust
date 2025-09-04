use axum::{Router, routing::get};

use shared::SharedState;

use crate::core::view;

pub fn router() -> Router<SharedState> {
    Router::new().merge(router_municipio())
}

fn router_municipio() -> Router<SharedState> {
    Router::new()
        .route("/atualizar_ibge", get(view::atualizar_ibge))
        .route("/cidades", get(view::list_municipio))
}

pub fn router_public() -> Router<SharedState> {
    Router::new()
        .route("/cidades-por-ibge", get(view::read_cidade_por_ibge))
        .route("/buscar-cep", get(view::buscar_cep))
        .merge(api_cidade_router())
}

fn api_cidade_router() -> Router<SharedState> {
    Router::new()
        .route("/cidades-br-api", get(view::cidade_br_list_api))
        .route("/cidades-to-api", get(view::cidades_to_api))
}
