use axum::{
    extract::{Form, Path, Query, State}, http::StatusCode, response::{Html, IntoResponse, Redirect, Response}, Extension, Json
};

use minijinja::Value;
use minijinja::context;
use serde::Deserialize;
use shared::{
    helpers::{self, get_qr_code_base64}, AppError, FlashStatus, ListParams, PaginatedResponse, SharedState
};
use std::collections::{BTreeMap, HashMap};
use tracing::debug;
use validator::Validate;

use crate::{middlewares, permissao::{model::module::{Module, Permission}, schema::{IdParams, RolePermissionCreateSchema, UserLocalPasswordUpdateSchema, UserUpdateSchema}, service::RolePermissionService}};
use crate::permissao::{
    User,
    model::module::Perfil,
    repository::{
        ModuleRepository, PerfilRepository, PermissionRepository,
        UserRepository,
    },
    schema::{
        PerfilCreateSchema, PerfilUpdateSchema, UserCreateSchema, UserParams,
        UserPasswordUpdateSchema, UserRolesCreateSchema,
    },
    service::{PerfilService, PermissionService, UserRolesService, UserService},
};
use crate::{
    permissao::schema::{CreateModuleSchema, PermissionCreateSchema, PermissionUpdateSchema},
    permissao::service::ModuleService,
};

pub async fn home(State(state): State<SharedState>) -> Html<String> {
    let template = state.templates.get_template("index.html").unwrap();
    let html = template.render(()).unwrap();
    Html(html)
}

pub async fn list_modules(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let page = params.page.unwrap_or(1);
    let per_page = 10; // Itens por página
    let search_term = params.find.unwrap_or_default();

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

    // Buscar módulos com filtro e paginação
    let (modules, total_records) = if search_term.is_empty() {
        // Buscar todos os módulos
        let count_result = sqlx::query_scalar!("SELECT COUNT(*) FROM module")
            .fetch_one(&*state.db)
            .await;

        let modules_result = sqlx::query_as!(
            Module,
            "SELECT * FROM module ORDER BY id DESC LIMIT $1 OFFSET $2",
            per_page as i64,
            ((page - 1) * per_page) as i64
        )
        .fetch_all(&*state.db)
        .await;

        match (count_result, modules_result) {
            (Ok(count), Ok(modules)) => (modules, count.unwrap_or(0) as usize),
            _ => (Vec::new(), 0),
        }
    } else {
        // Buscar com filtro
        let search_pattern = format!("%{}%", search_term);

        let count_result = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM module WHERE title ILIKE $1",
            search_pattern
        )
        .fetch_one(&*state.db)
        .await;

        let modules_result = sqlx::query_as!(
            Module,
            "SELECT * FROM module WHERE title ILIKE $1 ORDER BY id DESC LIMIT $2 OFFSET $3",
            search_pattern,
            per_page as i64,
            ((page - 1) * per_page) as i64
        )
        .fetch_all(&*state.db)
        .await;

        match (count_result, modules_result) {
            (Ok(count), Ok(modules)) => (modules, count.unwrap_or(0) as usize),
            _ => (Vec::new(), 0),
        }
    };

    let total_pages = (total_records + per_page as usize - 1) / per_page as usize;

    let ctx = context! {
        title => "Lista de Módulos",
        rows => modules,
        current_page => page,
        total_pages => total_pages,
        total_records => total_records,
        find => if search_term.is_empty() { None } else { Some(search_term) },
        flash_message => flash_message,
        flash_status => flash_status,
    };

    match state.templates.get_template("permissao/modulo_list.html") {
        Ok(template) => match template.render(ctx) {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao renderizar template: {}", err),
            )
                .into_response(),
        },
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Falha ao carregar template: {}", err),
        )
            .into_response(),
    }
}

pub async fn create_model(
    State(state): State<SharedState>,
    Form(body): Form<CreateModuleSchema>,
) -> Response {
    let service = ModuleService::new();

    match service.create(&state.db, body).await {
        Ok(module) => {
            // Redirecionar com mensagem de sucesso
            let redirect_url = helpers::create_flash_url(
                &format!("/permissao/modulo-form/{}", module.id),
                "Módulo criado com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&redirect_url).into_response()
        }
        Err(err) => {
            let error_message = if err
                .to_string()
                .contains("duplicate key value violates unique constraint")
            {
                "Este módulo já existe"
            } else {
                "Ocorreu um erro ao criar o módulo"
            };

            // Redirecionar com mensagem de erro
            let redirect_url = helpers::create_flash_url(
                "/permissao/modulo-form",
                error_message,
                FlashStatus::Error,
            );

            debug!("Erro ao criar módulo: {}", err);
            Redirect::to(&redirect_url).into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub find: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

pub async fn modules_list_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<Module>>, StatusCode> {
    let service = ModuleService::new();
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

pub async fn saudacao(State(state): State<SharedState>) -> Html<String> {
    let context = [("nome", "João")];
    let template = state.templates.get_template("saudacao.html").unwrap();
    let html = template.render(context).unwrap();
    Html(html)
}

pub async fn show_module_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .get("msg")
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.get("status").and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    let ctx = context! {
        title => "Cadastro de Módulo",
        flash_message => flash_message,
        flash_status => flash_status,
    };

    match state.templates.get_template("permissao/modulo_form.html") {
        Ok(template) => match template.render(ctx) {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao renderizar template: {}", err),
            )
                .into_response(),
        },
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Falha ao carregar template: {}", err),
        )
            .into_response(),
    }
}

pub async fn list_modulo(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Html<String>, StatusCode> {
    let service = ModuleService::new();

    let res = service
        .get_paginated(
            &state.db,
            q.find.as_deref(),
            q.page.unwrap_or(1) as i32,
            q.page_size.unwrap_or(10) as i32,
        )
        .await
        .map_err(|err| {
            debug!("Erro ao buscar módulos: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let _dados_formatados: Vec<Value> = res
        .data
        .clone()
        .into_iter()
        .map(|m| {
            let mut map = BTreeMap::new();
            map.insert("id".to_string(), Value::from(m.id));
            map.insert("title".to_string(), Value::from(m.title));
            map.insert(
                "createdAt".to_string(),
                Value::from(
                    m.created_at
                        .map(|dt| dt.format("%d/%m/%Y %H:%M").to_string())
                        .unwrap_or_default(),
                ),
            );
            map.insert(
                "updatedAt".to_string(),
                Value::from(
                    m.updated_at
                        .map(|dt| dt.format("%d/%m/%Y %H:%M").to_string())
                        .unwrap_or_default(),
                ),
            );
            Value::from(map)
        })
        .collect();

    let context = context! {
        //dados => dados_formatados,
        dados => res.data,
        total => res.total_records,
        pagina => res.page,
        por_pagina => res.page_size,
        total_paginas => res.total_pages,
    };

    let template = state
        .templates
        .get_template("permissao/modulo.html")
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let html = template.render(&context).map_err(|err| {
        debug!("Erro ao renderizar template: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Html(html))
}

pub async fn get_modulo(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, Response> {
    // Buscar o módulo no banco de dados
    let module_result = sqlx::query_as::<_, Module>("SELECT * FROM module WHERE id = $1")
        .bind(id)
        .fetch_one(&*state.db)
        .await;

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
    let template = match state.templates.get_template("permissao/modulo_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    match module_result {
        Ok(module) => {
            let ctx = context! {
                row => module,
                flash_message => flash_message,
                flash_status => flash_status,
            };
            match template.render(ctx) {
                Ok(html) => Ok(Html(html)),
                Err(err) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Falha ao renderizar template: {}", err),
                )
                    .into_response()),
            }
        }
        Err(e) => {
            debug!("Erro ao buscar módulo: {}", e);
            Err(Redirect::to("/permissao/modulo").into_response())
        }
    }
}

pub async fn update_modulo(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Form(body): Form<CreateModuleSchema>,
) -> Response {
    let query_result = sqlx::query_as!(
        Module,
        "UPDATE module SET title = $1 WHERE id = $2 RETURNING *",
        body.title.to_string(),
        id
    )
    .fetch_one(&*state.db)
    .await;

    match query_result {
        Ok(_) => {
            // Redirecionar com mensagem de sucesso
            let redirect_url = helpers::create_flash_url(
                &format!("/permissao/modulo-form/{}", id),
                "Módulo atualizado com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&redirect_url).into_response()
        }
        Err(e) => {
            let error_message = if e
                .to_string()
                .contains("duplicate key value violates unique constraint")
            {
                "Este título já existe para outro módulo"
            } else {
                "Ocorreu um erro ao atualizar o módulo"
            };

            // Redirecionar com mensagem de erro
            let redirect_url = helpers::create_flash_url(
                &format!("/permissao/modulo-form/{}", id),
                error_message,
                FlashStatus::Error,
            );

            debug!("Erro ao atualizar módulo: {}", e);
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn delete_module(State(state): State<SharedState>, Path(id): Path<i32>) -> Response {
    let query_result = sqlx::query!("DELETE FROM module WHERE id = $1", id)
        .execute(&*state.db)
        .await;

    match query_result {
        Ok(result) => {
            if result.rows_affected() > 0 {
                // Redirecionar com mensagem de sucesso
                let redirect_url = helpers::create_flash_url(
                    "/permissao/modulo",
                    "Módulo excluído com sucesso!",
                    FlashStatus::Success,
                );
                Redirect::to(&redirect_url).into_response()
            } else {
                // Módulo não encontrado
                let redirect_url = helpers::create_flash_url(
                    "/permissao/modulo",
                    "Módulo não encontrado",
                    FlashStatus::Error,
                );
                Redirect::to(&redirect_url).into_response()
            }
        }
        Err(e) => {
            debug!("Erro ao excluir módulo: {}", e);
            let redirect_url = helpers::create_flash_url(
                "/permissao/modulo",
                "Erro ao excluir módulo",
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn list_permissions(
    Query(params): Query<ListParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let service = PermissionService::new();

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
        .get_paginated_with_module(
            &state.db,
            params.find.as_deref(),
            params.page.unwrap_or(1) as i64,
            params.page_size.unwrap_or(10) as i64,
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

            match state
                .templates
                .get_template("permissao/permission_list.html")
            {
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
                "/permission",
                &format!("Erro ao carregar permissions: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn show_permission_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, Response> {
    // Buscar todos os módulos para o dropdown
    let modules = match sqlx::query_as::<_, Module>("SELECT * FROM module ORDER BY title")
        .fetch_all(&*state.db)
        .await
    {
        Ok(modules) => modules,
        Err(_) => {
            vec![]
        }
    };

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
        modules => modules,
        flash_message => flash_message,
        flash_status => flash_status,
    };

    match state
        .templates
        .get_template("permissao/permission_form.html")
    {
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

pub async fn create_permission(
    State(state): State<SharedState>,
    Form(body): Form<PermissionCreateSchema>,
) -> Response {
    match sqlx::query!(
        "INSERT INTO permission (name, description, module_id) VALUES ($1, $2, $3) RETURNING *",
        body.name.to_string(),
        body.description,
        body.module_id
    )
    .fetch_one(&*state.db)
    .await
    {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/permission",
                "Permissão criada com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/permission-form",
                &format!("Erro ao criar permissão: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn get_permission(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, Response> {
    let service = PermissionService::new();
    let serv_module = ModuleService::new();

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
    let template = match state
        .templates
        .get_template("permissao/permission_form.html")
    {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    // Buscar a permissão
    let permission = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar permissão: {}", e);
            let flash_url = helpers::create_flash_url(
                "/permissao/permission",
                &format!("Permissão não encontrada: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    // Buscar o módulo associado
    let module = match serv_module.get_by_id(&state.db, permission.module_id).await {
        Ok(m) => Some(m),
        Err(e) => {
            debug!("Erro ao buscar módulo: {}", e);
            None
        }
    };

    // Preparar o contexto
    let ctx = context! {
        row => permission,
        modulo => module,
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

pub async fn update_permission(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Form(input): Form<PermissionUpdateSchema>,
) -> Response {
    let query_result = sqlx::query!(
        "UPDATE permission SET name = $1, description = $2, module_id = $3, updated_at = NOW() WHERE id = $4 RETURNING *",
        input.name,
        input.description,
        input.module_id,
        id
    )
    .fetch_one(&*state.db)
    .await;
    match query_result {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/permissao/permission"),
                &format!("Permissão atualizada com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/permissao/permission-form/{}", id),
                &format!("Erro ao atualizar permissão: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_permission(State(state): State<SharedState>, Path(id): Path<i32>) -> Response {
    match sqlx::query!("DELETE FROM permission WHERE id = $1", id)
        .execute(&*state.db)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                let flash_url = helpers::create_flash_url(
                    "/permissao/permission",
                    "Permissão excluída com sucesso!",
                    FlashStatus::Success,
                );
                Redirect::to(&flash_url).into_response()
            } else {
                let flash_url = helpers::create_flash_url(
                    "/permissao/permission",
                    "Permissão não encontrada",
                    FlashStatus::Error,
                );
                Redirect::to(&flash_url).into_response()
            }
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/permission",
                &format!("Erro ao excluir permissão: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn permission_list_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<Permission>>, StatusCode> {
    let service = PermissionService::new();
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

//===========================
// PERFIL(ROLE)
//===========================

pub async fn list_perfil(
    Query(params): Query<ListParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let service = PerfilService::new();

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

            match state.templates.get_template("permissao/perfil_list.html") {
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
                "/perfil",
                &format!("Erro ao carregar permissions: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn show_perfil_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, Response> {
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

    match state.templates.get_template("permissao/perfil_form.html") {
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

pub async fn create_perfil(
    State(state): State<SharedState>,
    Form(body): Form<PerfilCreateSchema>,
) -> Response {
    match sqlx::query!(
        "INSERT INTO roles (name) VALUES ($1) RETURNING *",
        body.name.to_string(),
    )
    .fetch_one(&*state.db)
    .await
    {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/perfil",
                "Perfil criado com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/perfil-form",
                &format!("Erro ao criar perfil: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn get_perfil(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, Response> {
    let service = PerfilService::new();

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
    let template = match state.templates.get_template("permissao/perfil_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    // Buscar o perfil
    let perfil = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar perfil: {}", e);
            let flash_url = helpers::create_flash_url(
                "/permissao/perfil",
                &format!("Perfil não encontrado: {}", e),
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

pub async fn update_perfil(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Form(input): Form<PerfilUpdateSchema>,
) -> Response {
    let query_result = sqlx::query!(
        "UPDATE roles SET name = $1 WHERE id = $2 RETURNING *",
        input.name,
        id
    )
    .fetch_one(&*state.db)
    .await;
    match query_result {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/permissao/perfil"),
                &format!("Perfil atualizado com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/permissao/perfil-form/{}", id),
                &format!("Erro ao atualizar perfil: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_perfil(State(state): State<SharedState>, Path(id): Path<i32>) -> Response {
    match sqlx::query!("DELETE FROM roles WHERE id = $1", id)
        .execute(&*state.db)
        .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                let flash_url = helpers::create_flash_url(
                    "/permissao/perfil",
                    "Perfil excluído com sucesso!",
                    FlashStatus::Success,
                );
                Redirect::to(&flash_url).into_response()
            } else {
                let flash_url = helpers::create_flash_url(
                    "/permissao/perfil",
                    "Perfil não encontrado",
                    FlashStatus::Error,
                );
                Redirect::to(&flash_url).into_response()
            }
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/perfil",
                &format!("Erro ao excluir perfil: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn perfil_list_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<Perfil>>, StatusCode> {
    let service = PerfilService::new();
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

//===========================
// User
//===========================

pub async fn list_user(
    Query(params): Query<ListParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let service = UserService::new();

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

    // Usar o UserService para buscar dados paginados
    let result = service
        .get_paginated(
            &state.db,
            params.find.as_deref(),
            params.page.unwrap_or(1),
            params.page_size.unwrap_or(10),
        )
        .await;

    match result {
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

            match state.templates.get_template("permissao/user_list.html") {
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
            debug!("Erro ao buscar users: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/user",
                &format!("Erro ao carregar users: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn show_user_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, Response> {
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

    match state.templates.get_template("permissao/user_form.html") {
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

pub async fn create_user(
    State(state): State<SharedState>,
    Form(body): Form<UserCreateSchema>,
) -> Response {
    let service = UserService::new();

    /* service.validate_email(&body.email).await.map_err(|err| {
        let flash_url = helpers::create_flash_url(
            "/permissao/user-form",
            &format!("Erro ao validar email: {}", err),
            FlashStatus::Error,
        );
        Redirect::to(&flash_url).into_response()
    })?; */

    match service.create(&state.db, body).await {
        Ok(user) => {
            let flash_url = helpers::create_flash_url(
                &format!("/permissao/user-form/{}", user.id),
                "User criado com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/user-form",
                &format!("Erro ao criar user: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn get_user(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, Response> {
    let service = UserService::new();

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
    let template = match state.templates.get_template("permissao/user_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    // Buscar o perfil
    let perfil = match service.get_by_id(&state.db, id as i64).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar perfil: {}", e);
            let flash_url = helpers::create_flash_url(
                "/permissao/perfil",
                &format!("Perfil não encontrado: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    let otp_url = match &perfil.otp_base32 {
        Some(secret) => UserService::get_otp_url(&perfil.full_name, secret),
        None => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Usuário não possui OTP configurado".to_string(),
            )
                .into_response());
        }
    };

    // gera o QR Code em Base64 a partir da URL
    let qrcode = get_qr_code_base64(&otp_url).unwrap();

    // Preparar o contexto
    let ctx = context! {
        row => perfil,
        qrcode => qrcode,
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

/* 
somente usuarios super podem atualizar usuarios
*/
pub async fn update_user(
    State(state): State<SharedState>,
    Extension(current_user): Extension<middlewares::CurrentUser>,
    Path(id): Path<i64>,
    Form(body): Form<UserUpdateSchema>,
) -> Response {

    if !current_user.current_user.is_superuser {
        let flash_url = helpers::create_flash_url(
            &format!("/permissao/user-form/{}", id),
            &"Você não tem permissão para atualizar este usuário".to_string(),
            FlashStatus::Error,
        );
        return Redirect::to(&flash_url).into_response();
    }

    let service  = UserService::new();
    match service.update(&state.db, id, body).await {
        Ok(result) => {
            if result.id > 0 {
                let flash_url = helpers::create_flash_url(
                    &format!("/permissao/user-form/{}", result.id),
                    "Usuário atualizado com sucesso!",
                    FlashStatus::Success,
                );
                Redirect::to(&flash_url).into_response()
            } else {
                let flash_url = helpers::create_flash_url(
                    "/permissao/user-form",
                    "Usuário não atualizado",
                    FlashStatus::Error,
                );
                Redirect::to(&flash_url).into_response()
            }
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/permissao/user-form/{}", id),
                &format!("Erro ao atualizar usuário: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}


/* 
somente usuarios super podem atualizar OTP
*/
pub async fn update_user_otp(
    State(state): State<SharedState>,
    Extension(current_user): Extension<middlewares::CurrentUser>,
    Path(id): Path<i64>
) -> Response {

    if !current_user.current_user.is_superuser {
        let flash_url = helpers::create_flash_url(
            &format!("/permissao/user-form/{}", id),
            &"Você não tem permissão para atualizar este usuário".to_string(),
            FlashStatus::Error,
        );
        return Redirect::to(&flash_url).into_response();
    }

    match UserService::update_otp(&state.db, id).await {
        Ok(result) => {
            if result.id > 0 {
                let flash_url = helpers::create_flash_url(
                    &format!("/permissao/user-form/{}", result.id),
                    "OTP atualizado com sucesso!",
                    FlashStatus::Success,
                );
                Redirect::to(&flash_url).into_response()
            } else {
                let flash_url = helpers::create_flash_url(
                    "/permissao/user-form",
                    "OTP não atualizado",
                    FlashStatus::Error,
                );
                Redirect::to(&flash_url).into_response()
            }
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/permissao/user-form/{}", id),
                &format!("Erro ao atualizar OTP: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}


/*
atualizar senhas de usuarios somente para super usuarios admin
*/
pub async fn update_senha_user(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
    Form(body): Form<UserPasswordUpdateSchema>,
) -> Response {
    match UserService::update_password(&state.db, id, body).await {
        Ok(result) => {
            if result.id > 0 {
                let flash_url = helpers::create_flash_url(
                    &format!("/permissao/user-form/{}", result.id),
                    "Senha atualizada com sucesso!",
                    FlashStatus::Success,
                );
                Redirect::to(&flash_url).into_response()
            } else {
                let flash_url = helpers::create_flash_url(
                    "/permissao/user-form",
                    "Senha não atualizada",
                    FlashStatus::Error,
                );
                Redirect::to(&flash_url).into_response()
            }
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/user-form",
                &format!("Erro ao atualizar senha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}


/* troca de usuario pelo proprio usuario */
pub async fn user_update_senha_local_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, Response> {

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

    match state.templates.get_template("settings/senha_form.html") {
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

/* troca de usuario pelo proprio usuario */
pub async fn user_update_senha_local(
    State(state): State<SharedState>,
    Extension(current_user): Extension<middlewares::CurrentUser>,
    Form(form): Form<UserLocalPasswordUpdateSchema>,
) -> Response {

    //validar senha atual
    if !UserService::verify_password(&form.password, &current_user.current_user.password).unwrap() {
        let flash_url = helpers::create_flash_url(
            "/permissao/senha-form",
            "Senha atual incorreta",
            FlashStatus::Error,
        );
        return Redirect::to(&flash_url).into_response();
    }

    // validar nova senha
    if let Err(errors) = form.validate() {
        //debug!("Erros de validação: {:?}", errors);

        let flash_url = helpers::create_flash_url(
            "/permissao/senha-form",
            &format!("Nova senha inválida: {:?}", errors),
            FlashStatus::Error,
        );
        return Redirect::to(&flash_url).into_response();
    }

    match UserService::update_password(
        &state.db,
        current_user.current_user.id,
        UserPasswordUpdateSchema{password: form.password}
    ).await {
        Ok(user) => {
            let flash_url = helpers::create_flash_url(
                &format!("/permissao/senha-form"),
                "Senha atualizada com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/senha-form",
                &format!("Erro ao atualizar senha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }

}

pub async fn users_list_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<User>>, StatusCode> {
    let service = UserService::new();
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

//===========================
// User Gestão Perfil
//===========================

pub async fn get_user_gestao_perfil(
    Query(params): Query<ListParams>,
    Query(form): Query<UserParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    debug!("parametro: {:?}", form.user_id);
    let service_user = UserService::new();
    let service_user_roles = UserRolesService::new();

    let mut user: Option<User> = None;

    let mut context = minijinja::context! {};

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

    match form.user_id {
        Some(user_id) => {
            user = Some(service_user.get_by_id(&*state.db, user_id).await.unwrap());
            let result = service_user_roles
                .get_paginated_with_roles(
                    &state.db,
                    params.find.as_deref(),
                    params.page.unwrap_or(1) as i64,
                    params.page_size.unwrap_or(10) as i64,
                )
                .await;

            match result {
                Ok(paginated_response) => {
                    context = minijinja::context! {
                        user => user,
                        rows => paginated_response.data,
                        current_page => paginated_response.page,
                        total_pages => paginated_response.total_pages,
                        page_size => paginated_response.page_size,
                        total_records => paginated_response.total_records,
                        find => params.find.unwrap_or_default(),
                        flash_message => flash_message,
                        flash_status => flash_status,
                    };
                }
                Err(_err) => {}
            }
        }
        None => (),
    }

    match state.templates.get_template("permissao/perfil_usuario.html") {
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

pub async fn create_user_gestao_perfil(
    State(state): State<SharedState>,
    Form(body): Form<UserRolesCreateSchema>,
) -> Response {
    let service = UserRolesService::new();

    let user_id = body.user_id.clone();

    match service.create(&state.db, body).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!(
                    "/permissao/user-gestao-perfil?user_id={}",
                    user_id.to_string()
                ),
                "Perfil adicionado com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/user-gestao-perfil",
                &format!("Erro ao adicionar perfil: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_user_gestao_perfil(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
) -> Response {
    let service = UserRolesService::new();

    match service.delete(&state.db, id).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/user-gestao-perfil",
                "Perfil removido com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/user-gestao-perfil",
                &format!("Erro ao remover perfil: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}


//===========================
// Gestão Perfil
//===========================

pub async fn get_gestao_perfil(
    Query(params): Query<ListParams>,
    Query(form): Query<IdParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let service_perfil = PerfilService::new();
    let service = RolePermissionService::new();

    let mut perfil: Option<Perfil> = None;

    let mut context = minijinja::context! {};

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

    match form.id {
        Some(id) => {
            perfil = Some(service_perfil.get_by_id(&*state.db, id as i32).await.unwrap());
            let result = service
                .get_paginated_with_permission(
                    &state.db,
                    params.find.as_deref(),
                    params.page.unwrap_or(1) as i64,
                    params.page_size.unwrap_or(10) as i64,
                )
                .await;

            match result {
                Ok(paginated_response) => {
                    context = minijinja::context! {
                        perfil => perfil,
                        rows => paginated_response.data,
                        current_page => paginated_response.page,
                        total_pages => paginated_response.total_pages,
                        page_size => paginated_response.page_size,
                        total_records => paginated_response.total_records,
                        find => params.find.unwrap_or_default(),
                        flash_message => flash_message,
                        flash_status => flash_status,
                    };
                }
                Err(_err) => {}
            }
        }
        None => (),
    }

    match state.templates.get_template("permissao/perfil_gestao.html") {
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

pub async fn create_gestao_perfil(
    State(state): State<SharedState>,
    Form(body): Form<RolePermissionCreateSchema>,
) -> Response {
    let service = RolePermissionService::new();

    let role_id = body.role_id.clone();

    match service.create(&state.db, body).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!(
                    "/permissao/gestao-perfil?id={}",
                    role_id.to_string()
                ),
                "Permissão adicionada com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/gestao-perfil",
                &format!("Erro ao adicionar permissão: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_gestao_perfil(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Response {
    let service = RolePermissionService::new();

    match service.delete(&state.db, id).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/gestao-perfil",
                "Perfil removido com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/permissao/gestao-perfil",
                &format!("Erro ao remover perfil: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

