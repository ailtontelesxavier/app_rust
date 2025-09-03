use std::collections::HashMap;

use axum::{
    Extension, Form,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use minijinja::context;
use shared::{FlashStatus, ListParams, SharedState, helpers};
use tracing::debug;
use uuid::Uuid;

use crate::{
    externo::{
        LinhaService,
        schema::{CreateContato, CreateLinhaSchema, UpdateContato, UpdateLinhaSchema}, service::ContatoService,
    },
    middlewares::CurrentUser,
};

pub async fn list_linha(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = LinhaService::new();

    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .msg
        .as_ref()
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.status.as_ref().and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    // Usar o PermissionService para buscar dados paginados
    let permissions_result = service
        .get_paginated(
            &state.db,
            params.find.as_deref(),
            params.page.unwrap_or(1),
            params.page_size.unwrap_or(10),
        )
        .await;

    match permissions_result {
        Ok(paginated_response) => {
            let context = minijinja::context! {
                rows => paginated_response.data,
                current_page => paginated_response.page,
                total_pages => paginated_response.total_pages,
                page_size => paginated_response.page_size,
                total_records => paginated_response.total_records,
                find => params.find.unwrap_or_default(),
                flash_message => flash_message,
                flash_status => flash_status,
            };

            match state.templates.get_template("externo/linha_list.html") {
                Ok(template) => match template.render(context) {
                    Ok(html) => Html(html).into_response(),
                    Err(err) => {
                        debug!("Erro ao renderizar template: {}", err);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                },
                Err(err) => {
                    debug!("Erro ao carregar template: {}", err);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(err) => {
            debug!("Erro ao buscar linhas: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar linhas: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn linha_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .get("msg")
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.get("status").and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    let context = minijinja::context! {
        flash_message => flash_message,
        flash_status => flash_status,
    };

    match state.templates.get_template("externo/linha_form.html") {
        Ok(template) => match template.render(context) {
            Ok(html) => Ok(Html(html)),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao renderizar template: {}", err),
            )
                .into_response()),
        },
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao carregar template: {}", err),
        )
            .into_response()),
    }
}

pub async fn create_linha(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Form(mut body): Form<CreateLinhaSchema>,
) -> impl IntoResponse {
    if !current_user.current_user.is_superuser {
        let flash_url = helpers::create_flash_url(
            &format!("/externo/linha"),
            &"Você não tem permissão para criar uma linha".to_string(),
            FlashStatus::Error,
        );
        return Redirect::to(&flash_url).into_response();
    }

    let service = LinhaService::new();

    match service.create(&*state.db, body).await {
        Ok(linha) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/linha-form/{}", linha.id),
                "Linha criada com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/linha-form",
                &format!("Erro ao criar linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn get_linha(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let service = LinhaService::new();

    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .get("msg")
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());

    let flash_status = params.get("status").and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    // Carregar o template
    let template = match state.templates.get_template("externo/linha_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    let linha = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar linha: {}", e);
            let flash_url = helpers::create_flash_url(
                "/externo/linha-form",
                &format!("linha não encontrada: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    // Preparar o contexto
    let ctx = context! {
        row => linha,
        flash_message => flash_message,
        flash_status => flash_status,
    };

    // Renderizar o template
    match template.render(&ctx) {
        Ok(html) => Ok(Html(html)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Falha ao renderizar template: {}", err),
        )
            .into_response()),
    }
}

pub async fn update_linha(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Form(input): Form<UpdateLinhaSchema>,
) -> impl IntoResponse {
    let service = LinhaService::new();

    match service.update(&*state.db, id, input).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/linha"),
                &format!("Linha atualizada com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/linha-form/{}", id),
                &format!("Erro ao atualizar linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_linha(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let service = LinhaService::new();

    match service.delete(&*state.db, id).await {
        Ok(()) => {
            let flash_url = helpers::create_flash_url(
                "/externo/linha",
                "Linha excluída com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/linha",
                &format!("Erro ao excluir linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

/*
==========================================

----------------- Contato ---------------
==========================================
*/

pub async fn list_contato(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = ContatoService::new();

    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .msg
        .as_ref()
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.status.as_ref().and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    // Usar o PermissionService para buscar dados paginados
    let permissions_result = service
        .get_paginated(
            &state.db,
            params.find.as_deref(),
            params.page.unwrap_or(1),
            params.page_size.unwrap_or(10),
        )
        .await;

    match permissions_result {
        Ok(paginated_response) => {
            let context = minijinja::context! {
                rows => paginated_response.data,
                current_page => paginated_response.page,
                total_pages => paginated_response.total_pages,
                page_size => paginated_response.page_size,
                total_records => paginated_response.total_records,
                find => params.find.unwrap_or_default(),
                flash_message => flash_message,
                flash_status => flash_status,
            };

            match state.templates.get_template("externo/contato_list.html") {
                Ok(template) => match template.render(context) {
                    Ok(html) => Html(html).into_response(),
                    Err(err) => {
                        debug!("Erro ao renderizar template: {}", err);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                },
                Err(err) => {
                    debug!("Erro ao carregar template: {}", err);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(err) => {
            debug!("Erro ao buscar contatos: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar contatos: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

/* 
 formulario para contato
*/
pub async fn contato_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .get("msg")
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.get("status").and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    let context = minijinja::context! {
        flash_message => flash_message,
        flash_status => flash_status,
    };

    match state.templates.get_template("externo/contato_form.html") {
        Ok(template) => match template.render(context) {
            Ok(html) => Ok(Html(html)),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao renderizar template: {}", err),
            )
                .into_response()),
        },
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao carregar template: {}", err),
        )
            .into_response()),
    }
}

/* 
 cria contato interno
*/
pub async fn create_contato(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Form(mut body): Form<CreateContato>,
) -> impl IntoResponse {

    let service = ContatoService::new();

    match service.create(&*state.db, body).await {
        Ok(contato) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/contato-form/{}", contato.id),
                "Contato criado com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/contato-form",
                &format!("Erro ao criar contato: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn get_contato(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let service = ContatoService::new();

    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .get("msg")
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());

    let flash_status = params.get("status").and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    // Carregar o template
    let template = match state.templates.get_template("externo/contato_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    let contato = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar contato: {}", e);
            let flash_url = helpers::create_flash_url(
                "/externo/contato-form",
                &format!("contato não encontrado: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    // Preparar o contexto
    let ctx = context! {
        row => contato,
        flash_message => flash_message,
        flash_status => flash_status,
    };

    // Renderizar o template
    match template.render(&ctx) {
        Ok(html) => Ok(Html(html)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Falha ao renderizar template: {}", err),
        )
            .into_response()),
    }
}


pub async fn update_contato(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Form(input): Form<UpdateContato>,
) -> impl IntoResponse {
    let service = ContatoService::new();

    match service.update(&*state.db, id, input).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/contato"),
                &format!("Contato atualizado com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/linha-form/{}", id),
                &format!("Erro ao atualizar linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_contato(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let service = ContatoService::new();

    match service.delete(&*state.db, id).await {
        Ok(()) => {
            let flash_url = helpers::create_flash_url(
                "/externo/contato",
                "Contato excluído com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/contato",
                &format!("Erro ao excluir contato: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}


