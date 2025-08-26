use axum::{
    extract::{Path, State},
    http::{Response, StatusCode},
    response::IntoResponse,
};
use shared::AppState;
use tokio::fs::File;
use tokio_util::io::ReaderStream;
use std::path::{PathBuf};
use std::sync::Arc;


/* 
Utilizado para gerencar acesso as pasta e arquivos 
da pasta upload
*/
pub async fn serve_upload(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>, // captura o que vem depois de /uploads/
) -> impl IntoResponse {

    let upload_dir: String = "uploads".into();
    // Normalizar caminho para evitar path traversal
    let mut file_path = PathBuf::from(&upload_dir);
    file_path.push(&path);

    // Exemplo: verificar permissão baseado na pasta raiz
    if let Some(folder) = file_path.components().nth(1) {
        let folder_str = folder.as_os_str().to_string_lossy();
        if !user_has_permission(&folder_str).await {
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
async fn user_has_permission(folder: &str) -> bool {
    match folder {
        "chamado" => true,     // permitir
        "arquivos" => false,   // negar
        _ => false,
    }
}