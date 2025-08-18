use serde::{Deserialize, Serialize};
use std::{
    time::{Instant, SystemTime, UNIX_EPOCH}
};
use tracing::{debug, info};
use axum::{
    body::Body, http::{
        header::{AUTHORIZATION, COOKIE},
        Request, Response, StatusCode,
    }, middleware::{Next}, response::{IntoResponse},
};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use crate::{
    permissao::User,
};


static SECRET: &[u8] = b"chave_secreta_super_segura";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrentUser {
    pub current_user: User,
}

// Middleware de autenticação JWT
pub async fn autenticar(req: Request<Body>, next: Next) -> Response<Body> {
    // Primeiro tenta pegar o token do header Authorization
    let auth_header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    // Se não encontrou no header, tenta pegar do cookie
    let cookie_token = req
        .headers()
        .get(COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookie_str| {
            // Parse manual dos cookies
            cookie_str
                .split(';')
                .find_map(|cookie| {
                    let cookie = cookie.trim();
                    if cookie.starts_with("access_token=") {
                        Some(cookie.trim_start_matches("access_token=").to_string())
                    } else {
                        None
                    }
                })
        });

    // Usa o token do header ou do cookie
    let token = auth_header.or(cookie_token);

    match token {
        Some(token) => {
            // Decodifica o token percent-encoded se necessário
            let decoded_token = percent_encoding::percent_decode_str(&token)
                .decode_utf8()
                .unwrap_or_default()
                .to_string();

            match decode::<Claims>(&decoded_token, &DecodingKey::from_secret(SECRET), &Validation::default()) {
                Ok(data) => {
                    // Adiciona as claims do usuário às extensões da requisição
                    let mut req = req;
                    req.extensions_mut().insert(data.claims);

                    // Adicionar usuario logado
                    /* req.extensions_mut().insert(CurrentUser {
                        current_user: user.clone(),
                    });
                    */
                    next.run(req).await
                }
                Err(e) => {
                    debug!("Erro ao decodificar token: {}", e);
                    (StatusCode::UNAUTHORIZED, "Token inválido").into_response()
                }
            }
        }
        None => {
            debug!("Token não encontrado no header Authorization nem no cookie");
            (StatusCode::UNAUTHORIZED, "Token ausente").into_response()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

pub fn gerar_token(usuario: &str) -> String {
    let expiracao = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600;

    let claims = Claims {
        sub: usuario.to_string(),
        exp: expiracao as usize,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET),
    )
    .unwrap()
}

// Middleware de log
async fn log_middleware(req: Request<Body>, next: Next) -> Response<Body> {
    let start = Instant::now();
    let method = req.method().clone();
    let uri = req.uri().clone();

    let response = next.run(req).await;

    let duration = start.elapsed();
    info!("{} {} - {:?}", method, uri, duration);

    response
}

// "Usuários cadastrados" (fake)
fn verificar_credenciais(username: &str, password: &str) -> bool {
    username == "admin" && password == "1234"
}