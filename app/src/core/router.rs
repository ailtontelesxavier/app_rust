use axum::{Router, routing::get};

use shared::SharedState;

use crate::core::view::atualizar_ibge;

pub fn router() -> Router<SharedState> {
    Router::new().merge(router_municipio())
}

fn router_municipio() -> Router<SharedState> {
    Router::new().route("/atualizar_ibge", get(atualizar_ibge))
}
