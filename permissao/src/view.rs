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
use tracing::{debug, error};

use crate::schema::ModuleCreateShema;
use crate::{
    repository::{ModuleRepository, PaginatedResponse, Repository},
};
use crate::model::module::Module;

pub async fn home(State(state): State<SharedState>) -> Html<String> {
    let template = state.templates.get_template("index.html").unwrap();
    let html = template.render(()).unwrap();
    Html(html)
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

pub async fn list_modules(
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
