use axum::{
    Router,
    routing::{delete, get, post},
};

use shared::SharedState;

use crate::chamado::view;

pub fn router() -> Router<SharedState> {
    Router::new().route("/tipo", get(view::list_tipo_chamado))
}
