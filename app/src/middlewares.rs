use serde::{Deserialize, Serialize};
use shared::{AppError, AppState, SharedState};
use std::{
    sync::Arc, time::{Instant, SystemTime, UNIX_EPOCH}
};
use tracing::{debug, info};
use axum::{
    extract::Request as RequestExtract,
    body::Body, extract::State, http::{
        header::{AUTHORIZATION, COOKIE},
        Request, Response, StatusCode,
    }, middleware::Next, response::{IntoResponse, Redirect},
    response::{Html},
};
use axum::response::Response as ResponseExt;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use crate::permissao::{User, UserService};


static SECRET: &[u8] = b"chave_secreta_super_segura";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CurrentUser {
    pub current_user: User,
}

// Middleware de autenticação JWT
pub async fn autenticar(
    State(state): State<SharedState>,
    //Extension(state): Extension<Arc<SharedState>>,
    req: Request<Body>,
    next: Next
) -> Response<Body> {
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
                    req.extensions_mut().insert(data.claims.clone());
                    
                    
                    //busca usuario:
                    let user = UserService::get_by_username(&*state.db, &data.claims.sub)
                    .await
                    .map_err(|_| {
                        Redirect::to("/login").into_response()
                    });
                    

                    // Adiciona o usuário logado às extensões
                    req.extensions_mut().insert(CurrentUser {
                        current_user: user.unwrap(),
                    });
                    
                    next.run(req).await
                }
                Err(e) => {
                    debug!("Erro ao decodificar token: {}", e);
                    //(StatusCode::UNAUTHORIZED, "Token inválido").into_response()
                    Redirect::to("/login").into_response()
                }
            }
        }
        None => {
            debug!("Token não encontrado no header Authorization nem no cookie");
            //(StatusCode::UNAUTHORIZED, "Token ausente").into_response()
            Redirect::to("/login").into_response()
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
async fn log_middleware(
    req: Request<Body>, next: Next
) -> Response<Body> {
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

pub async fn role_check(
    req: Request<Body>,
    next: Next,
    required_roles: Vec<String>,
) -> Response<Body> {
    // Tenta pegar o usuário atual das extensões da requisição
    let current_user = req
        .extensions()
        .get::<CurrentUser>()
        .cloned();

    match current_user {
        Some(user_data) => {
            // Aqui você precisaria verificar se o usuário tem as roles necessárias
            // Por exemplo, se você tiver um campo `roles` no User:

            
            // Simples verificação - você pode ajustar conforme sua estrutura de roles
            let user_has_required_role = required_roles.is_empty() || 
                required_roles.iter().any(|role| {
                    // Assumindo que você tem um método para verificar roles
                    // ou um campo roles no User
                    match role.as_str() {
                        "admin" => user_data.current_user.is_superuser,
                        "user" => true, // qualquer usuário autenticado
                        _ => false,
                    }
                });
            
            // verifica se super user
            if user_data.current_user.is_superuser {
                next.run(req).await
            }
            else if user_has_required_role {
                next.run(req).await
            } else {
                debug!("Usuário não tem permissão para acessar este recurso");
                (StatusCode::FORBIDDEN, "Acesso negado").into_response()
            }
        }
        None => {
            debug!("Usuário não autenticado");
            Redirect::to("/login").into_response()
        }
    }
}

// Função helper para criar middleware de role check
pub fn require_roles(roles: Vec<&str>) -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response<Body>> + Send>> + Clone {
    let required_roles: Vec<String> = roles.iter().map(|s| s.to_string()).collect();
    
    move |req: Request<Body>, next: Next| {
        let roles = required_roles.clone();
        Box::pin(async move {
            role_check(req, next, roles).await
        })
    }
}


// Middleware simplificado para capturar 403
pub async fn handle_forbidden(req: Request<Body>, next: Next) -> ResponseExt {
    let res = next.run(req).await;

    if res.status() == StatusCode::FORBIDDEN {
        return (
            StatusCode::FORBIDDEN,
            Html(r#"
                <!DOCTYPE html>
                <html>
                <head>
                    <title>403 - Acesso Negado</title>
                </head>
                <body>
                    <h1>403 - Acesso Negado</h1>
                    <p>Você não tem permissão para acessar esta página.</p>
                    <a href="/">Voltar para a página inicial</a>
                </body>
                </html>
            "#),
        ).into_response();
    }

    res
}