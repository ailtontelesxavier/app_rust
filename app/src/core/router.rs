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
