use std::sync::Arc;

use axum::{
    Extension, Json,
    extract::State,
    extract::{Form, Path, Query},
    http::StatusCode,
    response::Html,
    response::IntoResponse,
};
use minijinja::Value;
use minijinja::context;
use serde::Deserialize;
use serde_json::json;
use shared::SharedState;
use sqlx::FromRow;
use std::collections::BTreeMap;
use tracing::{debug, info};

use crate::model::module::Module;
use crate::schema::ModuleCreateShema;
use crate::{
    handler,
    repository::{ModuleRepository, PaginatedResponse, Repository},
};

pub async fn home(State(state): State<SharedState>) -> Html<String> {
    let template = state.templates.get_template("index.html").unwrap();
    let html = template.render(()).unwrap();
    Html(html)
}

pub async fn create_model(
    State(state): State<SharedState>,
    Form(body): Form<ModuleCreateShema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let query_result = sqlx::query_as!(
        Module,
        "INSERT INTO module (title) VALUES ($1) RETURNING *",
        body.title.to_string(),
    )
    .fetch_one(&*state.db)
    .await;

    match query_result {
        Ok(module) => {
            let module_response = json!({
                "status": "success",
                "data": json!({
                    "module": module,
                }),
            });

            return Ok((StatusCode::CREATED, Json(module_response)));
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "This module already exists",
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
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

pub async fn get_modulo(
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

    let dados_formatados: Vec<Value> = res
        .data.clone()
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
