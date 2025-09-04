use axum::{
    Json,
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
};
use regex::Regex;
use reqwest::StatusCode;
use serde_json::{Value, json};
use shared::{FlashStatus, ListParams, PaginatedResponse, PaginationQuery, SharedState, helpers};
use tracing::debug;

use crate::core::{
    model::Municipio,
    repository::{fetch_municipios, fetch_ufs, upsert_municipios, upsert_ufs},
    schema::{CepQuery, CidadeParams, MunicipioWithUf},
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

/* APIS */
pub async fn buscar_cep(Query(params): Query<CepQuery>) -> impl IntoResponse {
    // Remove caracteres não numéricos
    let re = Regex::new(r"\D").unwrap();
    let cep = re.replace_all(&params.cep, "").to_string();

    // Valida se tem exatamente 8 dígitos
    if !Regex::new(r"^\d{8}$").unwrap().is_match(&cep) {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            "CEP inválido. Deve conter 8 dígitos numéricos.".to_string(),
        )
            .into_response();
    }

    let url_api = format!("https://viacep.com.br/ws/{}/json/", cep);

    let resp = match reqwest::get(&url_api).await {
        Ok(r) => r,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "Erro ao acessar serviço de CEP".to_string(),
            )
                .into_response();
        }
    };

    if !resp.status().is_success() {
        return (
            axum::http::StatusCode::BAD_GATEWAY,
            format!("Erro ao buscar CEP: {}", resp.status()),
        )
            .into_response();
    }

    let dados_json: Value = match resp.json().await {
        Ok(v) => v,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_GATEWAY,
                "Erro ao processar resposta do ViaCEP".to_string(),
            )
                .into_response();
        }
    };

    Json(dados_json).into_response()
}

/*
utilizado no cadastro externo API de acesso publico
não precisa ser usuario do sistema para consulta
*/
pub async fn read_cidade_por_ibge(
    State(state): State<SharedState>,
    Query(params): Query<CidadeParams>,
) -> impl IntoResponse {
    let service = MunicipioService::new();
    let row = service
        .find_municipio_with_uf_by_id(&state.db, params.ibge_id as i64)
        .await;

    match row {
        Ok(Some(cidade)) => Json(json!({
            "id": cidade.id,
            "nome": cidade.nome,
            "uf": cidade.uf_sigla,
            "uf_id": cidade.uf_id,
        }))
        .into_response(),

        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "Cidade não encontrada").into_response(),

        Err(_) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            "Erro ao buscar cidade",
        )
            .into_response(),
    }
}

/*
api publica retorna cidades do brasil
*/
pub async fn cidade_br_list_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<MunicipioWithUf>>, StatusCode> {
    let service = MunicipioService::new();
    let res = service
        .get_paginated(
            &state.db,
            q.find.as_deref(),
            q.page.unwrap_or(1) as i32,
            q.page_size.unwrap_or(10) as i32,
        )
        .await
        .map_err(|err| {
            debug!("error:{}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(res))
}

/*
api publica retorna cidades to tocantins

*/
pub async fn cidades_to_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<MunicipioWithUf>>, StatusCode> {
    let service = MunicipioService::new();
    let res = service
        .get_paginated_cidades_to(
            &state.db,
            q.find.as_deref(),
            q.page.unwrap_or(1) as i32,
            q.page_size.unwrap_or(10) as i32,
        )
        .await
        .map_err(|err| {
            debug!("error:{}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(res))
}
