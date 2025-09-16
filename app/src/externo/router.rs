use axum::{
    Router,
    routing::{delete, get, post},
};
use shared::SharedState;

use crate::externo::view;

pub fn router() -> Router<SharedState> {
    Router::new()
        .merge(router_tipo())
        .merge(router_contato())
        .merge(router_regiao())
        .merge(router_regiao_user())
        .merge(router_linha_user())
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
        .merge(api_linha_router())
}

fn api_linha_router() -> Router<SharedState> {
    Router::new()
        .route("/linha-api", get(view::linha_api))
}

fn router_contato() -> Router<SharedState> {
    Router::new()
        .route("/contato", get(view::list_contato))
        .route("/contato-form", get(view::contato_form))
        .route("/contato-form", post(view::create_contato))
        .route("/contato-form-pronaf", post(view::create_contato_pronaf))
        .route(
            "/contato-form/{id}",
            get(view::get_contato).post(view::update_contato),
        )
        .route("/contato/{id}", delete(view::delete_contato))
}

fn router_regiao() -> Router<SharedState> {
    Router::new()
        .route("/regiao", get(view::list_regiao))
        .route("/regiao-form", get(view::regiao_form))
        .route("/regiao-form", post(view::create_regiao))
        .route(
            "/regiao-form/{id}",
            get(view::get_regiao).post(view::update_regiao),
        )
        .route("/regiao/{id}", delete(view::delete_regiao))
        .merge(api_regiao_router())
        .route(
            "/regiao-gestao",
            get(view::get_gestao_regiao).post(view::create_gestao_regiao),
        )
        .route("/regiao-gestao/{id}", delete(view::delete_gestao_regiao))
}

fn api_regiao_router() -> Router<SharedState> {
    Router::new()
        .route("/regiao-api", get(view::regiao_api))
}

fn router_regiao_user() -> Router<SharedState> {
    Router::new()
        .route(
            "/regiao-user",
            get(view::get_regiao_por_usuario).post(view::create_regiao_por_usuario),
        )
        .route("/regiao-user/{id}", delete(view::delete_regiao_por_usuario))
}


fn router_linha_user() -> Router<SharedState> {
    Router::new()
        .route(
            "/linha-user",
            get(view::get_linha_por_usuario).post(view::create_linha_por_usuario),
        )
        .route("/linha-user/{id}", delete(view::delete_linha_por_usuario))
}
