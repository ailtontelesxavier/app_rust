use axum::{
    extract::State,
    response::Html,
    routing::get,
    Router,
};
use tower_sessions::Session;
use shared::SharedState;

// Função simples para testar Session
pub async fn test_with_session(
    State(state): State<SharedState>,
    session: Session,
) -> Html<String> {
    Html("Test session working".to_string())
}

pub fn test_router() -> Router<SharedState> {
    Router::new()
        .route("/test", get(test_with_session))
}
