use axum::{
    Json,
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};

use minijinja::Value;
use minijinja::context;
use serde::Deserialize;
use shared::{SharedState, helpers, FlashStatus};
use std::collections::{BTreeMap, HashMap};
use tracing::debug;

use crate::schema::{ModuleCreateShema, PermissionCreateShema, PermissionUpdateShema};
use crate::{
    repository::{ModuleRepository, PaginatedResponse, Repository},
};
use crate::model::{module::Module, permission::{Permission, PermissionWithModule}};

#[derive(Deserialize)]
pub struct ListParams {
    pub page: Option<usize>,
    pub find: Option<String>,
    pub msg: Option<String>,
    pub status: Option<String>,
}

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
    let flash_message = params.msg.as_ref().map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.status.as_ref()
        .and_then(|s| match s.as_str() {
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

    let total_pages = (total_records + per_page - 1) / per_page;

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
            ).into_response(),
        },
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Falha ao carregar template: {}", err),
        ).into_response(),
    }
}

pub async fn create_model(
    State(state): State<SharedState>,
    Form(body): Form<ModuleCreateShema>,
) -> Response {
    let query_result = sqlx::query_as!(
        Module,
        "INSERT INTO module (title) VALUES ($1) RETURNING *",
        body.title.to_string(),
    )
    .fetch_one(&*state.db)
    .await;

    match query_result {
        Ok(module) => {
            // Redirecionar com mensagem de sucesso
            let redirect_url = helpers::create_flash_url(
                &format!("/permissao/modulo-form/{}", module.id),
                "Módulo criado com sucesso!",
                FlashStatus::Success
            );
            Redirect::to(&redirect_url).into_response()
        }
        Err(e) => {
            let error_message = if e.to_string()
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
                FlashStatus::Error
            );
            
            debug!("Erro ao criar módulo: {}", e);
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

pub async fn list_modules_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<Module>>, StatusCode> {
    let repo = ModuleRepository;
    let res = repo
        .get_paginated(
            &state.db,
            q.find.as_deref(),
            q.page.unwrap_or(1),
            q.page_size.unwrap_or(10),
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
    let flash_message = params.get("msg").map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.get("status")
        .and_then(|s| match s.as_str() {
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
    let repo = ModuleRepository;

    let res = repo
        .get_paginated(
            &state.db,
            q.find.as_deref(),
            q.page.unwrap_or(1),
            q.page_size.unwrap_or(10),
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
    let flash_message = params.get("msg").map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.get("status")
        .and_then(|s| match s.as_str() {
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
    Form(body): Form<ModuleCreateShema>,
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
                FlashStatus::Success
            );
            Redirect::to(&redirect_url).into_response()
        }
        Err(e) => {
            let error_message = if e.to_string()
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
                FlashStatus::Error
            );
            
            debug!("Erro ao atualizar módulo: {}", e);
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn delete_module(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
) -> Response {
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
                    FlashStatus::Success
                );
                Redirect::to(&redirect_url).into_response()
            } else {
                // Módulo não encontrado
                let redirect_url = helpers::create_flash_url(
                    "/permissao/modulo",
                    "Módulo não encontrado",
                    FlashStatus::Error
                );
                Redirect::to(&redirect_url).into_response()
            }
        }
        Err(e) => {
            debug!("Erro ao excluir módulo: {}", e);
            let redirect_url = helpers::create_flash_url(
                "/permissao/modulo",
                "Erro ao excluir módulo",
                FlashStatus::Error
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

// Implementação simplificada do handler de permissões
pub struct PermissionHandler;

impl PermissionHandler {
    pub async fn count_items(
        &self,
        pool: &sqlx::Pool<sqlx::Postgres>, 
        params: &shared::generic_list::GenericListParams
    ) -> Result<usize, sqlx::Error> {
        let search_term = params.search.as_deref().unwrap_or("");
        
        let count: i64 = if search_term.is_empty() {
            sqlx::query_scalar("SELECT COUNT(*) FROM permission p INNER JOIN module m ON p.module_id = m.id")
                .fetch_one(pool)
                .await?
        } else {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM permission p INNER JOIN module m ON p.module_id = m.id 
                 WHERE p.permission ILIKE $1 OR m.name ILIKE $1"
            )
            .bind(format!("%{}%", search_term))
            .fetch_one(pool)
            .await?
        };
        
        Ok(count as usize)
    }
    
    pub async fn fetch_items(
        &self,
        pool: &sqlx::Pool<sqlx::Postgres>, 
        params: &shared::generic_list::GenericListParams
    ) -> Result<Vec<crate::model::permission::PermissionWithModule>, sqlx::Error> {
        let page = params.page.unwrap_or(1);
        let limit = 10_i64;
        let offset = ((page - 1) * 10) as i64;
        let search_term = params.search.as_deref().unwrap_or("");
        
        let items = if search_term.is_empty() {
            sqlx::query_as::<_, crate::model::permission::PermissionWithModule>(
                "SELECT p.id, p.name, p.description, p.module_id, p.created_at, p.updated_at, m.name as module_name
                 FROM permission p 
                 INNER JOIN module m ON p.module_id = m.id
                 ORDER BY p.id DESC
                 LIMIT $1 OFFSET $2"
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, crate::model::permission::PermissionWithModule>(
                "SELECT p.id, p.name, p.description, p.module_id, p.created_at, p.updated_at, m.name as module_name
                 FROM permission p 
                 INNER JOIN module m ON p.module_id = m.id
                 WHERE p.name ILIKE $1 OR m.name ILIKE $1
                 ORDER BY p.id DESC
                 LIMIT $2 OFFSET $3"
            )
            .bind(format!("%{}%", search_term))
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
        };
        
        Ok(items)
    }
}

pub async fn list_permissions(
    Query(params): Query<shared::generic_list::GenericListParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let handler = PermissionHandler;
    let config = shared::generic_list::ListConfig {
        entity_name: "permission".to_string(),
        entity_label: "Permissão".to_string(),
        plural_label: "Permissões".to_string(),
        base_url: "/permission".to_string(),
        fields: vec![
            ("id".to_string(), "ID".to_string()),
            ("permission".to_string(), "Permissão".to_string()),
            ("module_name".to_string(), "Módulo".to_string()),
            ("created_at".to_string(), "Criado em".to_string()),
        ],
        searchable_fields: vec!["permission".to_string(), "module_name".to_string()],
        items_per_page: 10,
    };
    
    let items_result = handler.fetch_items(&state.db, &params).await;
    let count_result = handler.count_items(&state.db, &params).await;
    
    match (items_result, count_result) {
        (Ok(items), Ok(total_count)) => {
            let page = params.page.unwrap_or(1);
            let total_pages = (total_count + config.items_per_page - 1) / config.items_per_page;
            
            let context = minijinja::context! {
                items => items,
                config => config,
                search => params.search.as_deref().unwrap_or(""),
                current_page => page,
                total_pages => total_pages,
                total_count => total_count,
                flash_status => params.flash_status,
                flash_message => params.flash_message,
            };
            
            match state.templates.get_template("shared/generic_list.html") {
                Ok(template) => match template.render(context) {
                    Ok(rendered) => Html(rendered).into_response(),
                    Err(err) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Erro ao renderizar template: {}", err),
                    ).into_response(),
                },
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Erro ao carregar template: {}", err),
                ).into_response(),
            }
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Erro ao carregar dados das permissões".to_string(),
        ).into_response(),
    }
}

pub async fn show_permission_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, Response> {
    // Buscar todos os módulos para o dropdown
    let modules = match sqlx::query_as::<_, Module>("SELECT * FROM module ORDER BY name")
        .fetch_all(&*state.db)
        .await
    {
        Ok(modules) => modules,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erro ao carregar módulos".to_string(),
            ).into_response())
        }
    };

    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params.get("msg").map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.get("status")
        .and_then(|s| match s.as_str() {
            "success" => Some("success"),
            "error" => Some("error"),
            _ => None,
        });

    let context = minijinja::context! {
        modules => modules,
        flash_message => flash_message,
        flash_status => flash_status,
    };

    match state.templates.get_template("permissao/permission_form.html") {
        Ok(template) => match template.render(context) {
            Ok(html) => Ok(Html(html)),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao renderizar template: {}", err),
            ).into_response()),
        },
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao carregar template: {}", err),
        ).into_response()),
    }
}

pub async fn create_permission(
    State(state): State<SharedState>,
    Form(input): Form<PermissionCreateShema>,
) -> Response {
    match sqlx::query!(
        "INSERT INTO permission (name, description, module_id) VALUES ($1, $2, $3)",
        input.name,
        input.description,
        input.module_id
    )
    .execute(&*state.db)
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
    // Buscar a permissão no banco de dados
    let permission_result = sqlx::query_as::<_, Permission>("SELECT * FROM permission WHERE id = $1")
        .bind(id)
        .fetch_one(&*state.db)
        .await;

    // Buscar todos os módulos para o dropdown
    let modules = match sqlx::query_as::<_, Module>("SELECT * FROM module ORDER BY name")
        .fetch_all(&*state.db)
        .await
    {
        Ok(modules) => modules,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Erro ao carregar módulos".to_string(),
            ).into_response())
        }
    };

    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params.get("msg").map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.get("status")
        .and_then(|s| match s.as_str() {
            "success" => Some("success"),
            "error" => Some("error"),
            _ => None,
        });

    match permission_result {
        Ok(permission) => {
            let context = minijinja::context! {
                row => permission,
                modules => modules,
                flash_message => flash_message,
                flash_status => flash_status,
            };
            match state.templates.get_template("permissao/permission_form.html") {
                Ok(template) => match template.render(context) {
                    Ok(html) => Ok(Html(html)),
                    Err(err) => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Erro ao renderizar template: {}", err),
                    ).into_response()),
                },
                Err(err) => Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Erro ao carregar template: {}", err),
                ).into_response()),
            }
        }
        Err(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/permissao/permission"),
                &format!("Permissão não encontrada"),
                FlashStatus::Error,
            );
            Err(Redirect::to(&flash_url).into_response())
        }
    }
}

pub async fn update_permission(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Form(input): Form<PermissionUpdateShema>,
) -> Response {
    match sqlx::query!(
        "UPDATE permission SET name = $1, description = $2, module_id = $3, updated_at = NOW() WHERE id = $4",
        input.name,
        input.description,
        input.module_id,
        id
    )
    .execute(&*state.db)
    .await
    {
        Ok(result) => {
            if result.rows_affected() > 0 {
                let flash_url = helpers::create_flash_url(
                    &format!("/permissao/permission"),
                    &format!("Permissão atualizada com sucesso!"),
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
                &format!("/permissao/permission-form/{}", id),
                &format!("Erro ao atualizar permissão: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_permission(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
) -> Response {
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
