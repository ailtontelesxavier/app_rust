use axum::{
    Router,
    routing::{delete, get, post},
};

use crate::permissao::view;
use shared::SharedState;

pub fn router() -> Router<SharedState> {
    Router::new()
        .route("/index", get(view::home))
        .route("/saudacao", get(view::saudacao))
        .merge(modulo_router())
        .merge(permission_router())
        .merge(perfil_router())
        .merge(user_router())
        .merge(user_gestao_perfil_router())
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
        .merge(api_permission_router())
}

fn api_permission_router() -> Router<SharedState> {
    Router::new().route("/permission-api", get(view::permission_list_api))
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
        .merge(api_perfil_router())
}

fn api_perfil_router() -> Router<SharedState> {
    Router::new().route("/perfil-api", get(view::perfil_list_api))
}

fn user_router() -> Router<SharedState> {
    Router::new()
        .route("/user", get(view::list_user))
        .route(
            "/user-form",
            get(view::show_user_form).post(view::create_user),
        )
        .route(
            "/user-form/{id}",
            get(view::get_user).post(view::update_user),
        )
        .route("/user-form/otp/{id}", post(view::update_user_otp))
        .route("/user-form-senha/{id}", post(view::update_senha_user))
        .route(
            "/senha-form",
            post(view::user_update_senha_local).get(view::user_update_senha_local_form),
        )
        .merge(api_user_router())
    /*.route("/user/{id}", delete(view::delete_user))
     */
}

fn api_user_router() -> Router<SharedState> {
    Router::new().route("/user-api", get(view::users_list_api))
}

fn user_gestao_perfil_router() -> Router<SharedState> {
    Router::new()
        .route(
            "/user-gestao-perfil",
            get(view::get_user_gestao_perfil).post(view::create_user_gestao_perfil),
        )
        .route(
            "/user-gestao-perfil/{id}",
            delete(view::delete_user_gestao_perfil),
        )
        .route(
            "/gestao-perfil",
            get(view::get_gestao_perfil).post(view::create_gestao_perfil),
        )
        .route("/gestao-perfil/{id}", delete(view::delete_gestao_perfil))
}
