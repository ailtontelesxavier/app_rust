// Teste simples para verificar como usar tower_sessions
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    routing::get,
    Router,
};
use tower_sessions::Session;

// Teste simples de função que usa Session
pub async fn test_session(
    State(state): State<SharedState>,
    session: Session,
) -> impl IntoResponse {
    Html("test").into_response()
}
