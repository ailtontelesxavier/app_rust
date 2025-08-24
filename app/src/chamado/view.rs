use axum::{extract::{Query, State}, http::StatusCode, response::{Html, IntoResponse, Redirect}};
use shared::{helpers, FlashStatus, ListParams, SharedState};
use tracing::debug;

use crate::chamado::service::TipoChamadoService;




pub async fn list_tipo_chamado(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>
) -> impl IntoResponse {
    let service = TipoChamadoService::new();

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

    // Usar o PermissionService para buscar dados paginados
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

            match state
                .templates
                .get_template("chamado/tipo_list.html")
            {
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
            debug!("Erro ao buscar permissions: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar permissions: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}