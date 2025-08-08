

Macro para verificar permissão (simples via função decoradora)
Você pode criar uma função estilo wrap_handler para aplicar checagem personalizada:

rust
Copy
Edit
fn with_permission<F, Fut>(
    required_role: &'static str,
    handler: F,
) -> impl Fn(axum::extract::State<Arc<Permissions>>) -> Fut + Clone
where
    F: Fn() -> Fut + Clone + Send + 'static,
    Fut: std::future::Future<Output = impl IntoResponse> + Send + 'static,
{
    move |State(permissions): axum::extract::State<Arc<Permissions>>| {
        let has_permission = permissions.roles.contains(&required_role.to_string());
        let handler = handler.clone();
        async move {
            if has_permission {
                handler().await.into_response()
            } else {
                axum::http::StatusCode::FORBIDDEN.into_response()
            }
        }
    }
}
Uso:

rust
Copy
Edit
let app = Router::new().route(
    "/admin",
    get(with_permission("admin", || async { "Admin OK" })),
);






sqlx migrate add -r "init"
sqlx migrate run
sqlx migrate add create_permissao_module_table


https://editorjs.io/



//api
pub async fn api_create_model(
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