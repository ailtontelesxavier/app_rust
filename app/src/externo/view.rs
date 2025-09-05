use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use axum::{
    Extension, Form,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use bigdecimal::BigDecimal;
use minijinja::context;
use regex::Regex;
use serde_json::{Map, Value};
use shared::{FlashStatus, ListParams, SharedState, helpers};
use tracing::debug;
use uuid::Uuid;

use crate::{
    externo::{
        LinhaService, StatusCivil, TypeContato,
        schema::{
            ContatoSchema, CreateContatoSchema, CreateLinhaSchema, DOC_AGRICULTURA, DOC_CAPITAL,
            DOC_EMERGINCIAL, DOC_MAOS_QUE, DOC_MICRO_CREDITO, DOC_ONLINE, DOC_POPULAR, DOC_PRONAF,
            DocumentoRequerido, PronafB, UpdateContato, UpdateLinhaSchema,
        },
        service::ContatoService,
    },
    middlewares::CurrentUser,
};

pub async fn list_linha(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = LinhaService::new();

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

            match state.templates.get_template("externo/linha_list.html") {
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
            debug!("Erro ao buscar linhas: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar linhas: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

pub async fn linha_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
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

    match state.templates.get_template("externo/linha_form.html") {
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

pub async fn create_linha(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Form(mut body): Form<CreateLinhaSchema>,
) -> impl IntoResponse {
    if !current_user.current_user.is_superuser {
        let flash_url = helpers::create_flash_url(
            &format!("/externo/linha"),
            &"Você não tem permissão para criar uma linha".to_string(),
            FlashStatus::Error,
        );
        return Redirect::to(&flash_url).into_response();
    }

    let service = LinhaService::new();

    match service.create(&*state.db, body).await {
        Ok(linha) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/linha-form/{}", linha.id),
                "Linha criada com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/linha-form",
                &format!("Erro ao criar linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn get_linha(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let service = LinhaService::new();

    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .get("msg")
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());

    let flash_status = params.get("status").and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    // Carregar o template
    let template = match state.templates.get_template("externo/linha_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    let linha = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar linha: {}", e);
            let flash_url = helpers::create_flash_url(
                "/externo/linha-form",
                &format!("linha não encontrada: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    // Preparar o contexto
    let ctx = context! {
        row => linha,
        flash_message => flash_message,
        flash_status => flash_status,
    };

    // Renderizar o template
    match template.render(&ctx) {
        Ok(html) => Ok(Html(html)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Falha ao renderizar template: {}", err),
        )
            .into_response()),
    }
}

pub async fn update_linha(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Form(input): Form<UpdateLinhaSchema>,
) -> impl IntoResponse {
    let service = LinhaService::new();

    match service.update(&*state.db, id, input).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/linha"),
                &format!("Linha atualizada com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/linha-form/{}", id),
                &format!("Erro ao atualizar linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_linha(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let service = LinhaService::new();

    match service.delete(&*state.db, id).await {
        Ok(()) => {
            let flash_url = helpers::create_flash_url(
                "/externo/linha",
                "Linha excluída com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/linha",
                &format!("Erro ao excluir linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

/*
==========================================

----------------- Contato ---------------
==========================================
*/

pub async fn list_contato(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = ContatoService::new();

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
                tipo_contato => TypeContato::tipo_contato_options(),
                flash_message => flash_message,
                flash_status => flash_status,
            };

            match state.templates.get_template("externo/contato_list.html") {
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
            debug!("Erro ao buscar contatos: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar contatos: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}

/*
 formulario para contato
*/
pub async fn contato_form(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let type_contato = params.get("type_contato");

    let template;
    let documentos;

    if let Some(tipo) = type_contato
        .and_then(|s| s.parse::<i32>().ok())
        .and_then(TypeContato::from_i32)
    {
        template = template_contato(tipo as i32);
        documentos = get_list_documento(tipo as i32);
    } else {
        eprintln!("Tipo contato inválido");
        let flash_url = helpers::create_flash_url(
            "http://fomento.to.gov.br",
            &format!("Tipo de Contato não permitido"),
            FlashStatus::Error,
        );
        return Err(Redirect::to(&flash_url).into_response());
    }

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
        type_contato => type_contato.unwrap().parse::<i32>().ok(),
        estado_civil => StatusCivil::estado_civil_options(),
        tipo_contato => TypeContato::tipo_contato_options(),
        documentos => documentos,
    };

    match state.templates.get_template(template) {
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

fn template_contato(type_contato: i32) -> &'static str {
    match TypeContato::from_i32(type_contato) {
        Some(TypeContato::PronafB) => "externo/contato_form_pronaf_b.html",
        Some(TypeContato::AgriculturaFamiliar) => "externo/contato_form_agricultura.html",
        Some(TypeContato::CapitalDeGiroTurismo) => "externo/contato_form_capital.html",
        Some(TypeContato::CreditoOnline) => "externo/contato_form_credito.html",
        Some(TypeContato::CreditoPopular) => "externo/contato_form_credito_popular.html",
        Some(TypeContato::Emergencial) => "externo/contato_form_emergencial.html",
        Some(TypeContato::MaosQueCriam) => "externo/contato_form_maos_que_criam.html",
        Some(TypeContato::MicroCreditoOnline) => "externo/contato_form_micro_credito.html",
        _ => "externo/contato_form.html", // Template padrão ou de erro
    }
}

/*
Retorna lista de documentos para cada linha de contato
*/
fn get_list_documento(type_contato: i32) -> Option<&'static [DocumentoRequerido]> {
    match TypeContato::from_i32(type_contato) {
        Some(TypeContato::PronafB) => Some(DOC_PRONAF),
        Some(TypeContato::AgriculturaFamiliar) => Some(DOC_AGRICULTURA),
        Some(TypeContato::CapitalDeGiroTurismo) => Some(DOC_CAPITAL),
        Some(TypeContato::CreditoOnline) => Some(DOC_ONLINE),
        Some(TypeContato::CreditoPopular) => Some(DOC_POPULAR),
        Some(TypeContato::Emergencial) => Some(DOC_EMERGINCIAL),
        Some(TypeContato::MaosQueCriam) => Some(DOC_MAOS_QUE),
        Some(TypeContato::MicroCreditoOnline) => Some(DOC_MICRO_CREDITO),
        _ => None,
    }
}

/*
 cria contato interno pronaf
*/
pub async fn create_contato_pronaf(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let service = ContatoService::new();
    // Estruturas para armazenar os dados do form
    let mut form_data = CreateContatoSchema {
        cpf_cnpj: "".to_string(),
        nome: "".to_string(),
        telefone: "".to_string(),
        email: "".to_string(),
        cidade_id: 0,
        val_solicitado: BigDecimal::from(0),
    };

    // Vetor para armazenar arquivos
    let mut arquivos: Vec<(String, Vec<u8>)> = vec![];

    // Itera pelos campos do multipart
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();

        if let Some(file_name) = field.file_name().map(|s| s.to_string()) {
            // lê os bytes do arquivo
            let data = field.bytes().await.unwrap().to_vec();
            arquivos.push((file_name, data));
        } else {
            // campo de texto
            let text = field.text().await.unwrap_or_default();
            match name.as_str() {
                "cpf_cnpj" => form_data.cpf_cnpj = text,
                "nome" => form_data.nome = text,
                "telefone" => form_data.telefone = text,
                "email" => form_data.email = text,
                "cidade_id" => form_data.cidade_id = text.parse().unwrap_or(0),
                "val_solicitado" => {
                    form_data.val_solicitado =
                        BigDecimal::from_str(&text).unwrap_or(BigDecimal::from(0))
                }
                _ => {}
            }
        }
    }

    /* 
    let resultado = agrupa_items(itens);
    println!("{}", serde_json::to_string_pretty(&resultado).unwrap());
     */

    let flash_url = helpers::create_flash_url(
        &format!("/externo/contato-form/"),
        "Contato criado com sucesso!",
        FlashStatus::Success,
    );
    Redirect::to(&flash_url).into_response()

    /* match service.create(&*state.db, body).await {
        Ok(contato) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/contato-form/{}", contato.id),
                "Contato criado com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/contato-form",
                &format!("Erro ao criar contato: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    } */
}

/* 
para pronaf b juntar os arquivos
*/
/// Agrupa itens no formato `"descricao_0": "carro", "quantidade_0": "1", ...`
pub fn agrupa_items(itens: &str) -> Vec<Map<String, Value>> {
    // Parse do JSON
    let raw_itens: Value = match serde_json::from_str(itens) {
        Ok(val) => val,
        Err(_) => return vec![],
    };

    let raw_map = match raw_itens.as_object() {
        Some(map) => map,
        None => return vec![],
    };

    let re = Regex::new(r"(\w+)_([0-9]+)").unwrap();
    let mut grouped: BTreeMap<usize, Map<String, Value>> = BTreeMap::new();

    for (key, value) in raw_map.iter() {
        if let Some(caps) = re.captures(key) {
            let field = caps.get(1).unwrap().as_str();
            let idx: usize = caps.get(2).unwrap().as_str().parse().unwrap_or(0);

            let entry = grouped.entry(idx).or_insert_with(Map::new);
            entry.insert(field.to_string(), value.clone());
        }
    }

    grouped.into_values().collect()
}

/*
 cria contato interno
*/
pub async fn create_contato(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Form(mut body): Form<ContatoSchema>,
) -> impl IntoResponse {
    let service = ContatoService::new();

    let flash_url = helpers::create_flash_url(
        &format!("/externo/contato-form/"),
        "Contato criado com sucesso!",
        FlashStatus::Success,
    );
    Redirect::to(&flash_url).into_response()

    /* match service.create(&*state.db, body).await {
        Ok(contato) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/contato-form/{}", contato.id),
                "Contato criado com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/contato-form",
                &format!("Erro ao criar contato: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    } */
}

pub async fn get_contato(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let service = ContatoService::new();

    // Extrair mensagens flash dos parâmetros da query
    let flash_message = params
        .get("msg")
        .map(|msg| urlencoding::decode(msg).unwrap_or_default().to_string());

    let flash_status = params.get("status").and_then(|s| match s.as_str() {
        "success" => Some("success"),
        "error" => Some("error"),
        _ => None,
    });

    // Carregar o template
    let template = match state.templates.get_template("externo/contato_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    let contato = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar contato: {}", e);
            let flash_url = helpers::create_flash_url(
                "/externo/contato-form",
                &format!("contato não encontrado: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    // Preparar o contexto
    let ctx = context! {
        row => contato,
        flash_message => flash_message,
        flash_status => flash_status,
    };

    // Renderizar o template
    match template.render(&ctx) {
        Ok(html) => Ok(Html(html)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Falha ao renderizar template: {}", err),
        )
            .into_response()),
    }
}

pub async fn update_contato(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
    Form(input): Form<UpdateContato>,
) -> impl IntoResponse {
    let service = ContatoService::new();

    match service.update(&*state.db, id, input).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/contato"),
                &format!("Contato atualizado com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/linha-form/{}", id),
                &format!("Erro ao atualizar linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_contato(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let service = ContatoService::new();

    match service.delete(&*state.db, id).await {
        Ok(()) => {
            let flash_url = helpers::create_flash_url(
                "/externo/contato",
                "Contato excluído com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/contato",
                &format!("Erro ao excluir contato: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}
