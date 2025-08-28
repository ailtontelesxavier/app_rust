use axum::{
    Json,
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
};
use reqwest::StatusCode;
use shared::{FlashStatus, ListParams, SharedState, helpers};
use tracing::debug;

use crate::core::{
    repository::{fetch_municipios, fetch_ufs, upsert_municipios, upsert_ufs},
    service::MunicipioService,
};

/*
    utilizado para buscar e atualizar cidade4s e estados do ibge
*/
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

/*
    listagem de cidades(Municipios)
*/
pub async fn list_municipio(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = MunicipioService::new();

    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .msg
        .as_ref()
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());
    let flash_status = params.status.as_ref().and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    let permissions_result = service
        .get_paginated(
            &state.db,
            params.find.as_deref(),
            params.page.unwrap_or(1),
            params.page_size.unwrap_or(10),
        )
        .await;

    match permissions_result {
        Ok(paginated_response) => {
            let context = minijinja::context! {
                rows => paginated_response.data,
                current_page => paginated_response.page,
                total_pages => paginated_response.total_pages,
                page_size => paginated_response.page_size,
                total_records => paginated_response.total_records,
                find => params.find.unwrap_or_default(),
                flash_message => flash_message,
                flash_status => flash_status,
            };

            match state.templates.get_template("core/municipio_list.html") {
                Ok(template) => match template.render(context) {
                    Ok(html) => Html(html).into_response(),
                    Err(err) => {
                        debug!("Erro ao renderizar template: {}", err);
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    }
                },
                Err(err) => {
                    debug!("Erro ao carregar template: {}", err);
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
        Err(err) => {
            debug!("Erro ao buscar cidades: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar cidades: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}
