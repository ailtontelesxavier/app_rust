mod filters;

use std::{
    env,
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    body::Body,
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, SET_COOKIE, COOKIE},
        HeaderValue, Method, Request, Response, StatusCode,
    },
    middleware::{self, Next},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use axum::extract::FromRequestParts;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use minijinja::{path_loader, Environment};
use percent_encoding::{percent_encode, NON_ALPHANUMERIC};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::{format_description::well_known::Rfc2822, Duration, OffsetDateTime};
use tokio;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};
use tower_sessions::{cookie::CookieJar, MemoryStore, SessionManagerLayer};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

use permissao::router as router_permissao;
use shared::{AppState, MessageResponse};

use crate::filters::register_filters;


async fn hello_world() -> &'static str {
    "Welcome!"
}

static SECRET: &[u8] = b"chave_secreta_super_segura";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn gerar_token(usuario: &str) -> String {
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

// Middleware de autenticação JWT
async fn autenticar(req: Request<Body>, next: Next) -> Response<Body> {
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

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    // Carrega os templates
    // Crie o ambiente MiniJinja
    let mut env = Environment::new();
    env.set_loader(path_loader("templates"));
    // Registre os filtros
    register_filters(&mut env);

    let templates = Arc::new(env);

    let state = Arc::new(AppState {
        db: Arc::new(db_pool),
        templates,
        message: Arc::new(MessageResponse {
            status: "info".to_string(),
            message: "Sistema iniciado".to_string(),
        }),
    });

    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            "info,debug,tower_http=debug",
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:2000").await.unwrap();

    let server_dir = ServeDir::new("static");

    let store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(store).with_secure(false); // true em produção

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:2000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let rotas_privadas = Router::new()
        .route("/privado", get(rota_privada))
        .route("/logout", get(logout))
        .nest("/permissao", router_permissao())
        .layer(middleware::from_fn(autenticar));

    let app = Router::new()
        .route("/hello", get(hello_world))
        .route("/login", get(login))
        .nest_service("/static", server_dir)
        .layer(session_layer) // Sessões devem vir antes do CORS
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .merge(rotas_privadas)
        .with_state(state.clone());


    info!("Starting server on http://0.0.0.0:2000");
    debug!("Server running");
    //println!("Server running on http://0.0.0.0:2000");
    axum::serve(listener, app).await.unwrap();
}

async fn rota_privada() -> &'static str {
    "Acesso privado: você está autenticado!"
}

async fn login() -> Response<Body> {
    let access_token = gerar_token("usuario_demo");

    // Converte os módulos para JSON
    //let json_data = json!(modules.iter().map(|m| m.as_dict()).collect::<Vec<_>>()).to_string();
    
    // Configura os cookies
    let access_token_expire_minutes = env::var("ACCESS_TOKEN_EXPIRE_MINUTES")
        .unwrap_or_else(|_| "3600".to_string())
        .parse::<i64>()
        .unwrap_or(3600);
    
    let max_age = Duration::minutes(access_token_expire_minutes);
    let expires = OffsetDateTime::now_utc() + Duration::hours(1);

    // Formata a data de expiração no formato RFC2822
    let expires_formatted = expires.format(&Rfc2822).unwrap();
    
    // Cria a resposta
    let mut response = Response::new(Body::empty());

    // Adiciona os cookies
    /* let modules_cookie = format!(
        "modules={}; Max-Age={}; Path=/",
        percent_encoding::percent_encode(json_data.as_bytes(), percent_encoding::NON_ALPHANUMERIC),
        max_age.whole_seconds()
    ); */

    // Adiciona os cookies ao cabeçalho da resposta
    /* response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&modules_cookie).unwrap()
    ); */

    // Cria o cookie de access_token
    let access_token_cookie = format!(
        "access_token={}; HttpOnly; SameSite=Strict; Max-Age={}; Path=/; Expires={}",
        percent_encode(access_token.as_bytes(), NON_ALPHANUMERIC),
        max_age.whole_seconds(),
        expires_formatted
    );

    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&access_token_cookie).unwrap()
    );

    response
}

async fn logout() -> impl IntoResponse {
    // Cria uma resposta de sucesso
    let mut response = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .unwrap();

    // Invalida o cookie de access_token definindo uma data no passado
    let expired_cookie = format!(
        "access_token=; HttpOnly; SameSite=Strict; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Max-Age=0"
    );

    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(&expired_cookie).unwrap()
    );

    // Se você tiver outros cookies para limpar, adicione aqui
    // Exemplo para limpar o cookie 'usuario':
    let expired_usuario_cookie = "usuario=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Max-Age=0";
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(expired_usuario_cookie).unwrap()
    );

    response
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_soma() {
        assert_eq!(1 + 1, 2);
    }
}
