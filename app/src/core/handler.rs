use async_trait::async_trait;
use axum::{
    Extension,
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use shared::AppState;
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::{
    chamado::ChamadoService, middlewares::CurrentUser, permissao::{User}
};

#[async_trait]
pub trait PermissionChecker {
    async fn can_access(&self, user: &User, object_id: String, db: Arc<PgPool>) -> bool;
}

pub struct ChamadoPermissionChecker;
pub struct ProjetoPermissionChecker;
pub struct DocumentoPermissionChecker;

#[async_trait]
impl PermissionChecker for ChamadoPermissionChecker {
    async fn can_access(&self, user: &User, object_id: String, db: Arc<PgPool>) -> bool {
        ChamadoService::can_access(user, object_id.parse().unwrap_or(0), &db).await
    }
}

#[async_trait]
impl PermissionChecker for ProjetoPermissionChecker {
    async fn can_access(&self, user: &User, object_id: String, db: Arc<PgPool>) -> bool {
        // Lógica de verificação de permissão para "projeto"
        true
    }
}

#[async_trait]
impl PermissionChecker for DocumentoPermissionChecker {
    async fn can_access(&self, user: &User, object_id: String, db: Arc<PgPool>) -> bool {
        // Lógica de verificação de permissão para "documento"
        true
    }
}

/*
Utilizado para gerencar acesso as pasta e arquivos
da pasta upload
padrão pasta/pasta/ano/mes/id/nome_arquivo.ext
//"uploads/chamado/2025/8/3/e981a626-d5d3-45d2-8178-c509d4702c2b.jpg"

*/
pub async fn serve_upload(
    Extension(current_user): Extension<CurrentUser>,
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>, // captura o que vem depois de /uploads/
) -> impl IntoResponse {
    // obter usuario
    let user = current_user.current_user.clone();

    let upload_dir: String = "uploads".into();
    // Normalizar caminho para evitar path traversal
    let mut file_path = PathBuf::from(&upload_dir);
    file_path.push(&path);

    // Exemplo: verificar permissão baseado na pasta raiz
    if let Some(folder) = file_path.components().nth(1) {
        let folder_str = folder.as_os_str().to_string_lossy();

        //pegar id do objeto
        if let Some(id_objeto) = file_path.components().nth(1) {
            let id_objeto = id_objeto.as_os_str().to_string_lossy();
            if !user_has_permission(&folder_str, &user, id_objeto.to_string(), state.db.clone()).await {
                return StatusCode::FORBIDDEN.into_response();
            }
        } else {
            return StatusCode::FORBIDDEN.into_response();
        }
    }

    // Tentar abrir o arquivo
    match File::open(&file_path).await {
        Ok(file) => {
            let stream = ReaderStream::new(file);
            Response::builder()
                .status(StatusCode::OK)
                .body(axum::body::Body::from_stream(stream))
                .unwrap()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

// Simulação de verificação de permissão
async fn user_has_permission(
    folder: &str,
    user: &User,
    id_objeto: String,
    db: Arc<PgPool>,
) -> bool {
    match folder {
        "chamado" => ChamadoPermissionChecker.can_access(user, id_objeto, db).await,
        "arquivos" => false,
        _ => false,
    }
}
