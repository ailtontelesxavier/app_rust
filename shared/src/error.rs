use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    InvalidInput(String),
    NotFound,
    InternalServerError,
    SessionError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidInput(msg) => write!(f, "Input inválido: {}", msg),
            AppError::NotFound => write!(f, "Não encontrado"),
            AppError::InternalServerError => write!(f, "Erro interno do servidor"),
            AppError::SessionError(msg) => write!(f, "Erro de sessão: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_msg) = match self {
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::InternalServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal server error".to_string(),
            ),
            AppError::SessionError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Session error: {}", msg),
            ),
        };

        let body = Json(ErrorResponse { error: error_msg });

        (status, body).into_response()
    }
}

// Implementação para converter erros de tower_sessions para AppError
impl From<tower_sessions::session::Error> for AppError {
    fn from(err: tower_sessions::session::Error) -> Self {
        AppError::SessionError(err.to_string())
    }
}
