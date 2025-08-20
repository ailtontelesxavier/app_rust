use axum::{
    body::Body, http::StatusCode, response::{Html, IntoResponse, Response}, Json
};
use serde::Serialize;
use std::fmt;

#[derive(Debug)]
pub enum AppError {
    InvalidInput(String),
    NotFound,
    InternalServerError,
    SessionError(String),
    InvalidSecret,
    VerificationFailed,
    PermissionDenied,
    UserNotAuthenticated,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidInput(msg) => write!(f, "Input inválido: {}", msg),
            AppError::NotFound => write!(f, "Não encontrado"),
            AppError::InternalServerError => write!(f, "Erro interno do servidor"),
            AppError::SessionError(msg) => write!(f, "Erro de sessão: {}", msg),
            AppError::InvalidSecret => write!(f, "Invalid base32 secret"),
            AppError::VerificationFailed => write!(f, "OTP verification failed"),
            AppError::PermissionDenied => write!(f,"You are not allowed to perform this action"),
            AppError::UserNotAuthenticated => write!(f,"Authentication required. Please log in."),
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
        match self {
            AppError::InvalidInput(msg) => {
                let body = Json(ErrorResponse { error: msg });
                (StatusCode::BAD_REQUEST, body).into_response()
            },
            AppError::NotFound => {
                let body = Json(ErrorResponse { error: "Not found".to_string() });
                (StatusCode::NOT_FOUND, body).into_response()
            },
            AppError::InternalServerError => {
                server_error("Internal server error".to_string()).1
            },
            AppError::SessionError(msg) => {
                server_error(format!("Session error: {}", msg)).1
            },
            AppError::InvalidSecret => {
                let body = Json(ErrorResponse { error: "".to_string() });
                (StatusCode::NOT_FOUND, body).into_response()
            },
            AppError::VerificationFailed => {
                let body = Json(ErrorResponse { error: "".to_string() });
                (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
            },
            AppError::PermissionDenied => {
                let body = Json(ErrorResponse { error: "You are not allowed to perform this action".to_string() });
                (StatusCode::FORBIDDEN, body).into_response()
            },
            AppError::UserNotAuthenticated => {
                let body = Json(ErrorResponse { error: "Authentication required. Please log in.".to_string() });
                (StatusCode::UNAUTHORIZED, body).into_response()
            },
        }
    }
}

// Implementação para converter erros de tower_sessions para AppError
impl From<tower_sessions::session::Error> for AppError {
    fn from(err: tower_sessions::session::Error) -> Self {
        AppError::SessionError(err.to_string())
    }
}

fn server_error(e: String) -> (StatusCode, Response<Body>) {
    eprintln!("Server error: {}", e);

    let html_string = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Erro Interno do Servidor</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        body {{ font-family: Arial, sans-serif; text-align: center; padding: 50px; }}
        .error-container {{ max-width: 600px; margin: 0 auto; }}
        h1 {{ color: #d32f2f; }}
        p {{ color: #666; }}
        .error-code {{ font-size: 4em; font-weight: bold; color: #d32f2f; margin: 20px 0; }}
    </style>
</head>
<body>
    <div class="error-container">
        <div class="error-code">500</div>
        <h1>Erro Interno do Servidor</h1>
        <p>Ocorreu um erro interno no servidor. Por favor, tente novamente mais tarde.</p>
        <p>Se o problema persistir, entre em contato com o administrador do sistema.</p>
        <a href="/">Voltar à página inicial</a>
    </div>
</body>
</html>"#
    );

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Html(html_string).into_response(),
    )
}
