

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