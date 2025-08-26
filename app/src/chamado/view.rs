use std::collections::HashMap;
use tokio::fs;

use axum::{
    Extension, Form, Json,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};

use chrono::{Datelike, Local};
use minijinja::context;
use shared::{FlashStatus, ListParams, PaginatedResponse, PaginationQuery, SharedState, helpers};
use tracing::debug;
use uuid::Uuid;

use crate::{
    chamado::{
        model::{ServicoChamado, StatusChamado, TipoChamado},
        schema::{
            CreateCategoriaChamadoSchema, CreateChamado, CreateServicoChamadoSchema,
            CreateTipoChamadoSchema, UpdateCategoriaChamadoSchema, UpdateChamado,
            UpdateServicoChamadoSchema, UpdateTipoChamadoSchema,
        },
        service::{CategoriaService, ChamadoService, ServicoService, TipoChamadoService},
    }, middlewares::CurrentUser
};

pub async fn list_tipo_chamado(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = TipoChamadoService::new();

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

            match state.templates.get_template("chamado/tipo_list.html") {
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
            debug!("Erro ao buscar permissions: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar permissions: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn show_tipo_form(
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

    match state.templates.get_template("chamado/tipo_form.html") {
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

pub async fn get_tipo(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let service = TipoChamadoService::new();

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
    let template = match state.templates.get_template("chamado/tipo_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    let perfil = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar tipo: {}", e);
            let flash_url = helpers::create_flash_url(
                "/chamado/tipo-form",
                &format!("tipo não encontrado: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    // Preparar o contexto
    let ctx = context! {
        row => perfil,
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

pub async fn create_tipo(
    State(state): State<SharedState>,
    Form(body): Form<CreateTipoChamadoSchema>,
) -> impl IntoResponse {
    let service = TipoChamadoService::new();

    match service.get_by_name(&*state.db, body.nome.clone()).await {
        Ok(tipo_existente) if tipo_existente.id > 0 => {
            let flash_url =
                helpers::create_flash_url("/chamado/tipo", "Tipo existe!", FlashStatus::Error);
            return Redirect::to(&flash_url).into_response();
        }
        _ => {
            // Não existe, pode criar
            match service.create(&*state.db, body).await {
                Ok(_) => {
                    let flash_url = helpers::create_flash_url(
                        "/chamado/tipo",
                        "Tipo criado com sucesso!",
                        FlashStatus::Success,
                    );
                    Redirect::to(&flash_url).into_response()
                }
                Err(err) => {
                    let flash_url = helpers::create_flash_url(
                        "/chamado/tipo-form",
                        &format!("Erro ao criar Tipo: {}", err),
                        FlashStatus::Error,
                    );
                    Redirect::to(&flash_url).into_response()
                }
            }
        }
    }
}

pub async fn update_tipo(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Form(input): Form<UpdateTipoChamadoSchema>,
) -> impl IntoResponse {
    let service = TipoChamadoService::new();

    match service.update(&*state.db, id, input).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/tipo"),
                &format!("Tipo atualizado com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/tipo-form/{}", id),
                &format!("Erro ao atualizar tipo: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_tipo(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let service = TipoChamadoService::new();
    match service.delete(&*state.db, id).await {
        Ok(()) => {
            let flash_url = helpers::create_flash_url(
                "/chamado/tipo",
                "Tipo excluído com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/chamado/tipo",
                &format!("Erro ao excluir tipo: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn tipo_list_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<TipoChamado>>, StatusCode> {
    let service = TipoChamadoService::new();
    let res = service
        .get_paginated(
            &state.db,
            q.find.as_deref(),
            q.page.unwrap_or(1) as i32,
            q.page_size.unwrap_or(10) as i32,
        )
        .await
        .map_err(|err| {
            debug!("error:{}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(res))
}

/*
==========================================

------------- CATEGORIA ------------------
==========================================

*/

pub async fn list_categoria(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = CategoriaService::new();

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

            match state.templates.get_template("chamado/categoria_list.html") {
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
            debug!("Erro ao buscar permissions: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar permissions: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn show_categoria_form(
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

    match state.templates.get_template("chamado/categoria_form.html") {
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

pub async fn get_categoria(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let service = CategoriaService::new();

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
    let template = match state.templates.get_template("chamado/categoria_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    let perfil = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar categoria: {}", e);
            let flash_url = helpers::create_flash_url(
                "/chamado/categoria-form",
                &format!("categoria não encontrado: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    // Preparar o contexto
    let ctx = context! {
        row => perfil,
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

pub async fn create_categoria(
    State(state): State<SharedState>,
    Form(body): Form<CreateCategoriaChamadoSchema>,
) -> impl IntoResponse {
    let service = CategoriaService::new();

    match service.get_by_name(&*state.db, body.nome.clone()).await {
        Ok(categoria) if categoria.id > 0 => {
            let flash_url = helpers::create_flash_url(
                "/chamado/categoria",
                "Categoria já existe!",
                FlashStatus::Error,
            );
            return Redirect::to(&flash_url).into_response();
        }
        _ => {
            // Não existe, pode criar
            match service.create(&*state.db, body).await {
                Ok(_) => {
                    let flash_url = helpers::create_flash_url(
                        "/chamado/categoria",
                        "Categoria criado com sucesso!",
                        FlashStatus::Success,
                    );
                    Redirect::to(&flash_url).into_response()
                }
                Err(err) => {
                    let flash_url = helpers::create_flash_url(
                        "/chamado/categoria-form",
                        &format!("Erro ao criar Categoria: {}", err),
                        FlashStatus::Error,
                    );
                    Redirect::to(&flash_url).into_response()
                }
            }
        }
    }
}

pub async fn update_categoria(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Form(input): Form<UpdateCategoriaChamadoSchema>,
) -> impl IntoResponse {
    let service = CategoriaService::new();

    match service.update(&*state.db, id, input).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/categoria"),
                &format!("Categoria atualizado com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/categoria-form/{}", id),
                &format!("Erro ao atualizar categoria: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_categoria(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let service = CategoriaService::new();
    match service.delete(&*state.db, id).await {
        Ok(()) => {
            let flash_url = helpers::create_flash_url(
                "/chamado/categoria",
                "Categoria excluído com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/chamado/categoria",
                &format!("Erro ao excluir categoria: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

/*
==========================================

------------- Serviço ------------------
==========================================

*/

pub async fn list_servico(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = ServicoService::new();

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

    let permissions_result = service
        .get_paginated_with_tipo(
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

            match state.templates.get_template("chamado/servico_list.html") {
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
            debug!("Erro ao buscar serviços: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar permissions: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn show_servico_form(
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

    match state.templates.get_template("chamado/servico_form.html") {
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

pub async fn get_servico(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let service = ServicoService::new();

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
    let template = match state.templates.get_template("chamado/servico_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    let perfil = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar servico: {}", e);
            let flash_url = helpers::create_flash_url(
                "/chamado/servico-form",
                &format!("servico não encontrado: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    let tipo = TipoChamadoService::new()
        .get_by_id(&*state.db, perfil.tipo_id)
        .await
        .unwrap();

    // Preparar o contexto
    let ctx = context! {
        row => perfil,
        tipo => tipo,
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

pub async fn create_servico(
    State(state): State<SharedState>,
    Form(body): Form<CreateServicoChamadoSchema>,
) -> impl IntoResponse {
    let service = ServicoService::new();

    match service.get_by_name(&*state.db, body.nome.clone()).await {
        Ok(servico) if servico.id > 0 => {
            let flash_url = helpers::create_flash_url(
                "/chamado/servico",
                "Serviço já existe!",
                FlashStatus::Error,
            );
            return Redirect::to(&flash_url).into_response();
        }
        _ => {
            // Não existe, pode criar
            match service.create(&*state.db, body).await {
                Ok(_) => {
                    let flash_url = helpers::create_flash_url(
                        "/chamado/servico",
                        "Serviço criado com sucesso!",
                        FlashStatus::Success,
                    );
                    Redirect::to(&flash_url).into_response()
                }
                Err(err) => {
                    let flash_url = helpers::create_flash_url(
                        "/chamado/servico-form",
                        &format!("Erro ao criar servico: {}", err),
                        FlashStatus::Error,
                    );
                    Redirect::to(&flash_url).into_response()
                }
            }
        }
    }
}

pub async fn update_servico(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Form(input): Form<UpdateServicoChamadoSchema>,
) -> impl IntoResponse {
    let service = ServicoService::new();

    match service.update(&*state.db, id, input).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/servico"),
                &format!("Serviço atualizado com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/servico-form/{}", id),
                &format!("Erro ao atualizar serviço: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_servico(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let service = ServicoService::new();
    match service.delete(&*state.db, id).await {
        Ok(()) => {
            let flash_url = helpers::create_flash_url(
                "/chamado/servico",
                "Serviço excluído com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/chamado/servico",
                &format!("Erro ao excluir serviço: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn servico_list_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<ServicoChamado>>, StatusCode> {
    let service = ServicoService::new();
    let res = service
        .get_paginated(
            &state.db,
            q.find.as_deref(),
            q.page.unwrap_or(1) as i32,
            q.page_size.unwrap_or(10) as i32,
        )
        .await
        .map_err(|err| {
            debug!("error:{}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(res))
}

/*
==========================================

------------- CHAMADO ------------------
==========================================

*/

pub async fn list_chamado(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = ChamadoService::new();

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

            match state.templates.get_template("chamado/chamado_list.html") {
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
            debug!("Erro ao buscar serviços: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar chamados: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn show_chamado_form(
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

    match state.templates.get_template("chamado/chamado_form.html") {
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

pub async fn get_chamado(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let service = ChamadoService::new();

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
    let template = match state.templates.get_template("chamado/chamado_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    let chamado = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar chamado: {}", e);
            let flash_url = helpers::create_flash_url(
                "/chamado/chamado-form",
                &format!("chamado não encontrado: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    let tipo = TipoChamadoService::new()
        .get_by_id(&*state.db, chamado.tipo_id)
        .await
        .unwrap();

    let servico = ServicoService::new()
        .get_by_id(&*state.db, chamado.servico_id)
        .await
        .unwrap();

    // Preparar o contexto
    let ctx = context! {
        row => chamado,
        tipo => tipo,
        servico => servico,
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

pub async fn create_chamado(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Form(mut body): Form<CreateChamado>,
) -> impl IntoResponse {
    let service = ChamadoService::new();

    // Preenche o usuário logado
    body.user_solic_id = Some(current_user.current_user.id);
    // Garante que todo chamado novo comece como "Aberto"
    body.status = Some(StatusChamado::Aberto as i32);

    match service.create(&*state.db, body).await {
        Ok(chamado) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/chamado-form/{}", chamado.id),
                "Chamado criado com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/chamado/chamado-form",
                &format!("Erro ao criar chamado: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn update_chamado(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Form(input): Form<UpdateChamado>,
) -> impl IntoResponse {
    let service = ChamadoService::new();

    //verificar se o chamado é do usuario
    //ou administrador
    if !ChamadoService::can_access(&current_user.current_user, id, &*state.db).await {
        let flash_url = helpers::create_flash_url(
            &format!("/chamado/chamado"),
            &format!("Você não tem permissão para atualizar chamado: {}", id),
            FlashStatus::Error,
        );
        return Redirect::to(&flash_url).into_response();
    }

    match service.update(&*state.db, id, input).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/chamado"),
                &format!("Chamado atualizado com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/chamado-form/{}", id),
                &format!("Erro ao atualizar chamado: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn upload_imagem(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<SharedState>,
    Path(id_chamado): Path<i64>,
    mut multipart: Multipart,
) -> Json<serde_json::Value> {
    //verificar se o chamado é do usuario
    if !ChamadoService::can_access(&current_user.current_user, id_chamado, &*state.db).await {
        return Json(serde_json::json!({
            "success": 0,
            "error": "Você não tem permissão para acessar este chamado."
        }));
    }

    let mut file_url = String::new();

    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or_default();

        if name == "image" {
            let agora = Local::now();
            let ano = agora.year();
            let mes = agora.month();

            // Inclui o ID do chamado no path para organização
            let path_file = format!("uploads/chamado/{}/{}/{}", ano, mes, id_chamado);

            // Cria o diretório se não existir
            if let Err(e) = fs::create_dir_all(&path_file).await {
                eprintln!("Erro ao criar diretório {}: {}", path_file, e);
                continue;
            }

            // Obtém o nome do arquivo e extensão
            let file_name = field.file_name().unwrap_or("img");
            let ext = file_name.rsplit('.').next().unwrap_or("png").to_lowercase();

            // Gera nome único para o arquivo
            let filename = format!("{}/{}.{}", path_file, Uuid::new_v4(), ext);

            // Lê os dados do arquivo
            let data = match field.bytes().await {
                Ok(data) => data,
                Err(e) => {
                    eprintln!("Erro ao ler dados do arquivo: {}", e);
                    continue;
                }
            };

            // Salva o arquivo
            match fs::write(&filename, &data).await {
                Ok(_) => {
                    file_url = format!("/{}", filename);
                    break; // Sai do loop após processar a primeira imagem
                }
                Err(e) => {
                    eprintln!("Erro ao salvar arquivo {}: {}", filename, e);
                }
            }
        }
    }

    // Retorno esperado pelo Editor.js
    Json(serde_json::json!({
        "success": if file_url.is_empty() { 0 } else { 1 },
        "file": {
            "url": file_url
        }
    }))
}

pub async fn delete_chamado(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    let service = ChamadoService::new();

    //verificar se o chamado é do usuario
    if !ChamadoService::can_access(&current_user.current_user, id, &*state.db).await {
        let flash_url = helpers::create_flash_url(
            &format!("/chamado/chamado"),
            &format!("Você não tem permissão para deletar este chamado: {}", id),
            FlashStatus::Error,
        );
        return Redirect::to(&flash_url).into_response();
    }

    match service.get_by_id(&state.db, id).await {
        Ok(chamado) => {
            // Converta o i32 para StatusChamado antes de comparar
        let status = match chamado.status {
            Some(status_code) => StatusChamado::from_i32(status_code),
            None => {
                let flash_url = helpers::create_flash_url(
                    "/chamado/chamado",
                    "Status do chamado não encontrado.",
                    FlashStatus::Error,
                );
                return Redirect::to(&flash_url).into_response();
            }
        };
        
        if status != StatusChamado::Aberto {
            let flash_url = helpers::create_flash_url(
                "/chamado/chamado",
                "Chamado não pode ser excluído pois não está mais em Aberto.",
                FlashStatus::Error,
            );
            return Redirect::to(&flash_url).into_response();
        }
        },
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/chamado/chamado"),
                &format!("Erro ao obter chamado: {}", err),
                FlashStatus::Error,
            );
            return Redirect::to(&flash_url).into_response();
        }
    };

    match service.delete(&*state.db, id).await {
        Ok(()) => {
            let flash_url = helpers::create_flash_url(
                "/chamado/chamado",
                "Chamado excluído com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/chamado/chamado",
                &format!("Erro ao excluir chamado: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}
