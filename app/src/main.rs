mod filters;
mod middlewares;
mod permissao;

use std::{
    collections::HashMap, env, fmt::Display, sync::Arc, time::{Instant, SystemTime, UNIX_EPOCH}
};

use axum::{
    body::Body, extract::{Query, State}, http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE},
        HeaderValue, Method, Request, Response, StatusCode,
    }, middleware::{self, Next}, response::{Html, IntoResponse, Redirect}, routing::{get, post}, Form, Router
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
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing::{debug, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use dotenv::dotenv;
use sqlx::postgres::PgPoolOptions;

use permissao::router as router_permissao;
use shared::{helpers, AppState, FlashStatus, MessageResponse, SharedState};

use crate::{filters::register_filters, permissao::{ModuleRepository, UserService}};


async fn hello_world() -> &'static str {
    "Welcome!"
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
        .route("/", get(index))
        .route("/privado", get(rota_privada))
        .route("/logout", get(logout))
        .nest("/permissao", router_permissao())
        .layer(middleware::from_fn(middlewares::autenticar));

    let app = Router::new()
        .route("/hello", get(hello_world))
        .route("/login", get(get_login).post(login))
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

async fn index(State(state): State<SharedState>,) -> Result<Html<String>, impl IntoResponse> {
    match state.templates.get_template("principal.html") {
        Ok(template) => match template.render({}) {
            Ok(html) => Ok(Html(html)),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao renderizar template: {}", err),
            )
                .into_response()),
        },
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao carregar template: {}", err),
        )
            .into_response()),
    }
}

async fn get_login(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse>{
    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .get("msg")
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.get("status").and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    let context = minijinja::context! {
        flash_message => flash_message,
        flash_status => flash_status,
    };

    match state.templates.get_template("login.html") {
        Ok(template) => match template.render(context) {
            Ok(html) => Ok(Html(html)),
            Err(err) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Erro ao renderizar template: {}", err),
            )
                .into_response()),
        },
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Erro ao carregar template: {}", err),
        )
            .into_response()),
    }
}

async fn login(
    State(state): State<SharedState>,
    Form(playload): Form<LoginPayload>
) -> Response<Body> {

    println!("{:?}", playload);

    match UserService::get_by_username(&state.db, &playload.username).await {
        Ok(user) => {

            if !user.is_active {

                let flash_url = helpers::create_flash_url(
                    "/login",
                    "Incorrect username or password",
                    FlashStatus::Error,
                );
                return Redirect::to(&flash_url).into_response();

            }

            if let Ok(false)| Err(_) = UserService::verify_password(&playload.password, &user.password){

                let flash_url = helpers::create_flash_url(
                    "/login",
                    "Incorrect username or password",
                    FlashStatus::Error,
                );
                return Redirect::to(&flash_url).into_response();
            }

            let access_token = middlewares::gerar_token(&user.username);


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
            //let mut response = Response::new(Body::empty());
            let mut response = Redirect::to("/").into_response();

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
        Err(err) => {

            let flash_url = helpers::create_flash_url(
                "/login",
                &format!("Senha não atualizada: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }


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
