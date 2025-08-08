use axum::{
    Json,
    extract::{Form, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};

use minijinja::Value;
use minijinja::context;
use serde::Deserialize;
use shared::SharedState;
use std::collections::BTreeMap;
use tracing::debug;

use crate::schema::ModuleCreateShema;
use crate::{
    repository::{ModuleRepository, PaginatedResponse, Repository},
};
use crate::{model::module::Module, schema::ModuleUpdateShema};

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
            let uri = format!("/permissao/modulo-form/{}", module.id);
            Redirect::to(&uri).into_response()
        }
        Err(e) => {
            debug!("Erro ao criar módulo: {}", e);
            Redirect::to("/permissao/modulo-form").into_response()
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
) -> impl IntoResponse {
    let context = context! {};

    match state.templates.get_template("permissao/modulo_form.html") {
        Ok(template) => {
            match template.render(context) {
                Ok(content) => Html(content).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to render template: {}", e),
                ).into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to load template: {}", e),
        ).into_response(),
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
    Path(id): Path<i32>,
    State(state): State<SharedState>,
) -> Response {
    // Buscar o módulo no banco de dados
    let module_result = sqlx::query_as::<_, Module>("SELECT * FROM module WHERE id = $1")
        .bind(id)
        .fetch_one(&*state.db)
        .await;

    // Carregar o template
    let template = match state.templates.get_template("permissao/modulo_form.html") {
        Ok(t) => t,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response();
        }
    };

    match module_result {
        Ok(module) => {
            let ctx = context! {
                row => module,
            };
            match template.render(ctx) {
                Ok(html) => Html(html).into_response(),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Falha ao renderizar template: {}", err),
                )
                    .into_response(),
            }
        }
        Err(e) => {
            debug!("Erro ao buscar módulo: {}", e);
            Redirect::to("/permissao/modulo").into_response()
        }
    }
}

pub async fn update_modulo(
    Path(id): Path<i32>,
    State(state): State<SharedState>,
    Form(form): Form<ModuleUpdateShema>,
) -> Response {
    // 1. Tentar atualizar o módulo no banco de dados
    let update_result = sqlx::query_as::<_, Module>(
        "UPDATE module SET title = $1, updated_at = NOW() WHERE id = $2 RETURNING *",
    )
    .bind(&form.title)
    .bind(id)
    .fetch_one(&*state.db)
    .await;

    match update_result {
        Ok(updated_module) => {
            // Redirecionar para o formulário do módulo atualizado
            let uri = format!("/permissao/modulo-form/{}", updated_module.id);
            Redirect::to(&uri).into_response()
        }
        Err(sqlx::Error::RowNotFound) => {
            // Módulo não existe - redirecionar para lista
            Redirect::to("/permissao/modulo").into_response()
        }
        Err(e) => {
            debug!("Erro ao atualizar módulo: {}", e);
            // Redirecionar de volta para o formulário do módulo
            let uri = format!("/permissao/modulo-form/{}", id);
            Redirect::to(&uri).into_response()
        }
    }
}
