use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::view;
use shared::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/index", get(view::home))
        .route("/saudacao", get(view::saudacao))
        .merge(modulo_router())
        .merge(permission_router())
        .merge(perfil_router())
        .merge(user_router())
}

fn modulo_router() -> Router<SharedState> {
    Router::new()
        .route("/modulo", get(view::list_modules))
        .route("/modulo-list", get(view::list_modulo))
        .route(
            "/modulo-form",
            get(view::show_module_form).post(view::create_model),
        )
        .route(
            "/modulo-form/{id}",
            get(view::get_modulo).post(view::update_modulo),
        )
        .route("/modulo/{id}", delete(view::delete_module))
        .merge(api_modulo_router())
}

fn api_modulo_router() -> Router<SharedState> {
    Router::new().route("/modulo-api", get(view::modules_list_api))
}

fn permission_router() -> Router<SharedState> {
    Router::new()
        .route("/permission", get(view::list_permissions))
        .route(
            "/permission-form",
            get(view::show_permission_form).post(view::create_permission),
        )
        .route(
            "/permission-form/{id}",
            get(view::get_permission).post(view::update_permission),
        )
        .route("/permission/{id}", delete(view::delete_permission))
}

fn perfil_router() -> Router<SharedState> {
    Router::new()
        .route("/perfil", get(view::list_perfil))
        .route(
            "/perfil-form",
            get(view::show_perfil_form).post(view::create_perfil),
        )
        .route(
            "/perfil-form/{id}",
            get(view::get_perfil).post(view::update_perfil),
        )
        .route("/perfil/{id}", delete(view::delete_perfil))
}

fn user_router() -> Router<SharedState> {
    Router::new().route("/user", get(view::list_user)).route(
        "/user-form",
        get(view::show_user_form).post(view::create_user),
    )
    .route(
        "/user-form/{id}",
        get(view::get_user),//.post(view::update_user),
    )
    /*.route("/user/{id}", delete(view::delete_user))
    */
}
