use axum::{Json, extract::State, response::IntoResponse};
use shared::SharedState;

use crate::core::repository::{fetch_municipios, fetch_ufs, upsert_municipios, upsert_ufs};

pub async fn atualizar_ibge(State(state): State<SharedState>) -> impl IntoResponse {
    let pool = &*state.db;

    // Atualiza UFs
    match fetch_ufs().await {
        Ok(ufs) => {
            if let Err(e) = upsert_ufs(pool, &ufs).await {
                return Json(
                    serde_json::json!({ "status": "error", "message": format!("Erro ao salvar UFs: {}", e) }),
                );
            }

            // Para cada UF, atualiza os municípios
            for uf in ufs {
                if let Ok(municipios) = fetch_municipios(&uf.sigla).await {
                    //debug!("Fetched {} municipios for UF {}", municipios.len(), uf.sigla);
                    if let Err(e) = upsert_municipios(pool, municipios, &uf.id).await {
                        return Json(
                            serde_json::json!({ "status": "error", "message": format!("Erro ao salvar municípios de {}: {}", uf.sigla, e) }),
                        );
                    }
                }
            }

            Json(
                serde_json::json!({ "status": "ok", "message": "Base de UFs e Municípios atualizada com sucesso!" }),
            )
        }
        Err(e) => Json(
            serde_json::json!({ "status": "error", "message": format!("Erro ao buscar UFs: {}", e) }),
        ),
    }
}
