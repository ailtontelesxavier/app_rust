use std::sync::Arc;

use axum::{
    Json,
    extract::State,
    extract::{Path, Query, Form},
    http::StatusCode,
    response::Html,
    response::IntoResponse,
    Extension,
};
use serde_json::json;
use shared::SharedState;
use sqlx::FromRow;

use crate::handler;
use crate::model::Module;
use crate::schema::ModuleCreateShema;

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
    Extension(pool): Extension<Arc<PgPool>>,
) -> Result<Json<PaginatedResponse<Module>>, StatusCode> {
    let repo = ModuleRepository;
    let res = repo
        .get_paginated(&pool, q.find.as_deref(), q.page.unwrap_or(1), q.page_size.unwrap_or(10))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(res))
}

pub async fn saudacao(State(state): State<SharedState>) -> Html<String> {
    let context = [("nome", "João")];
    let template = state.templates.get_template("saudacao.html").unwrap();
    let html = template.render(context).unwrap();
    Html(html)
}
