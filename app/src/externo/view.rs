use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

use anyhow::Result;
use axum::Json;
use axum::{
    Extension, Form,
    extract::{Multipart, Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect, Response},
};
use bigdecimal::BigDecimal;
use minijinja::context;
use regex::Regex;
use serde_json::{Map, Value};
use shared::{FlashStatus, ListParams, IdParams, PaginatedResponse, PaginationQuery, SharedState, helpers};
use tracing::debug;
use uuid::Uuid;
use validator::Validate;

use crate::externo::model::Regiao;
use crate::externo::schema::{AplicacaoRecursos, CreateRegiaoCidades, CreateRegiaoSchema, CreateUserLinha, CreateUserRegiao, TipoContatoExtra, UpdateRegiaoSchema};
use crate::externo::service::{RegiaoCidadesService, RegiaoService, UserLinhaService, UserRegiaoService};
use crate::permissao::UserService;
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
    let service_linha = LinhaService::new();

    let mut data_contato = CreateContatoSchema {
        cpf_cnpj: String::new(),
        nome: String::new(),
        telefone: String::new(),
        email: String::new(),
        cidade_id: 0,
        val_solicitado: BigDecimal::from(0),
    };

    // Estruturas para armazenar os dados do form
    let mut form_data = PronafB {
        nome_tecnico: "".to_string(),
        orgao_associacao_tecnico: "".to_string(),
        telefone_whatsapp_tecnico: "".to_string(),
        apelido: None,
        estado_civil: 0,
        cep: "".to_string(),
        endereco: "".to_string(),
        prev_aumento_fat: BigDecimal::from(0),
        cpf_conj: None,
        nome_conj: None,
        telefone_conj: None,
        email_conj: None,
        email: None,
        valor_estimado_imovel: None,
        desc_atividade: "".to_string(),
        finalidade_credito: "".to_string(),
    };

    let mut item_recurso = AplicacaoRecursos {
        descricao: "".to_string(),
        quantidade: 0,
        valor_unitario: BigDecimal::from(0),
        valor_total: BigDecimal::from(0),
    };

    let mut list_item_recurso = vec![];

    // Vetor para armazenar arquivos
    let mut arquivos: Vec<(String, (String, Vec<u8>))> = vec![];

    // Itera pelos campos do multipart
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("").to_string();

        if let Some(file_name) = field.file_name().map(|s| s.to_string()) {
            // lê os bytes do arquivo
            let data = field.bytes().await.unwrap().to_vec();
            arquivos.push((name, (file_name, data)));
        } else {
            // campo de texto
            let text = field.text().await.unwrap_or_default();

            // Preencher os campos específicos de AplicacaoRecursos lista
            if name.starts_with("descricao_") {
                /* let idx = name
                    .trim_start_matches("descricao_")
                    .parse::<usize>()
                    .unwrap_or(0);
                println!("descricao {} => {}", idx, text);
                // aqui você pode inserir em um vetor de itens, ex:
                // itens[idx].descricao = text; */
                item_recurso.descricao = text
            } else if name.starts_with("quantidade_") {
                item_recurso.quantidade = text.parse().unwrap_or(0);
            } else if name.starts_with("valor_unitario_") {
                item_recurso.valor_unitario =
                    BigDecimal::from_str(&text.replace(".", "").replace(",", "."))
                        .unwrap_or(BigDecimal::from(0));
            } else if name.starts_with("valor_total_") {
                item_recurso.valor_total =
                    BigDecimal::from_str(&text.replace(".", "").replace(",", "."))
                        .unwrap_or(BigDecimal::from(0));

                list_item_recurso.push(item_recurso.clone());
                item_recurso = AplicacaoRecursos {
                    descricao: "".to_string(),
                    quantidade: 0,
                    valor_unitario: BigDecimal::from(0),
                    valor_total: BigDecimal::from(0),
                };
            } else {
                // Preencher os campos específicos de PronafB
                match name.as_str() {
                    //dados contato
                    "cpf_cnpj" => data_contato.cpf_cnpj = text,
                    "nome" => data_contato.nome = text,
                    "telefone" => data_contato.telefone = text,
                    "email" => data_contato.email = text,
                    "cidade_id" => data_contato.cidade_id = text.parse().unwrap_or(0),
                    "val_solicitado" => {
                        data_contato.val_solicitado =
                            BigDecimal::from_str(&text.replace(".", "").replace(",", "."))
                                .unwrap_or(BigDecimal::from(0))
                    }

                    //dados pronaf_b
                    "nome_tecnico" => form_data.nome_tecnico = text,
                    "orgao_associacao_tecnico" => form_data.orgao_associacao_tecnico = text,
                    "telefone_whatsapp_tecnico" => form_data.telefone_whatsapp_tecnico = text,
                    "apelido" => {
                        form_data.apelido = if text.is_empty() { None } else { Some(text) }
                    }
                    "estado_civil" => form_data.estado_civil = text.parse().unwrap_or(0),
                    "cep" => form_data.cep = text,
                    "endereco" => form_data.endereco = text,
                    "prev_aumento_fat" => {
                        form_data.prev_aumento_fat =
                            BigDecimal::from_str(&text.replace(".", "").replace(",", "."))
                                .unwrap_or(BigDecimal::from(0))
                    }
                    "cpf_conj" => {
                        form_data.cpf_conj = if text.is_empty() { None } else { Some(text) }
                    }
                    "nome_conj" => {
                        form_data.nome_conj = if text.is_empty() { None } else { Some(text) }
                    }
                    "telefone_conj" => {
                        form_data.telefone_conj = if text.is_empty() { None } else { Some(text) }
                    }
                    "email_conj" => {
                        form_data.email_conj = if text.is_empty() { None } else { Some(text) }
                    }
                    "email" => form_data.email = if text.is_empty() { None } else { Some(text) },
                    "valor_estimado_imovel" => {
                        form_data.valor_estimado_imovel = if text.is_empty() {
                            None
                        } else {
                            Some(
                                BigDecimal::from_str(&text.replace(".", "").replace(",", "."))
                                    .unwrap_or(BigDecimal::from(0)),
                            )
                        }
                    }
                    "desc_atividade" => form_data.desc_atividade = text,
                    "finalidade_credito" => form_data.finalidade_credito = text,

                    // Preencher os campos específicos de AplicacaoRecursos
                    "descricao" => item_recurso.descricao = text,
                    "quantidade" => item_recurso.quantidade = text.parse().unwrap_or(0),
                    "valor_unitario" => {
                        item_recurso.valor_unitario =
                            BigDecimal::from_str(&text.replace(".", "").replace(",", "."))
                                .unwrap_or(BigDecimal::from(0));
                    }
                    "valor_total" => {
                        item_recurso.valor_total =
                            BigDecimal::from_str(&text.replace(".", "").replace(",", "."))
                                .unwrap_or(BigDecimal::from(0));
                        list_item_recurso.push(item_recurso.clone());
                    }

                    _ => {}
                }
            }
        }
    }

    match data_contato.validate() {
        Ok(_) => {}
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "detail": errors
                })),
            );
        }
    }

    if data_contato.val_solicitado > BigDecimal::from_str("15000").unwrap() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "status": "error",
                "detail": "Valor solicitado não pode ser superior a R$ 15.000,00"
            })),
        );
    }

    match form_data.validate() {
        Ok(_) => {}
        Err(errors) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "detail": errors
                })),
            );
        }
    }

    //validar documentos importante enviados obrigatorios foram enviados
    for (name, (file_name, data)) in &arquivos {
        if let Some(doc) = DOC_PRONAF.iter().find(|d| d.id == name) {
            if doc.obrigatorio {
                if data.is_empty() {
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "status": "error",
                            "errors": format!("Documento {} é obrigatório", doc.id)
                        })),
                    );
                }
            }
        }
    }

    /* println!("valor: {:?}", Some(&form_data.valor_estimado_imovel));
    println!("Form Data: {:?}", form_data); */

    //obten linha
    let linha;

    match service_linha.get_by_id(&*state.db, 8).await{
        Ok(l) => linha = l,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "detail": format!("Linha Pronaf não encontrada: {}", err)
                })),
            );
        }
    }

    match service
        .create(
            &*state.db,
            TipoContatoExtra::PronafB(form_data),
            linha,
            data_contato,
            Some(list_item_recurso),
            Some(arquivos),
        )
        .await
    {
        Ok(contato) => {
            return (
                StatusCode::CREATED,
                Json(serde_json::json!({
                    "code": 200,
                    "status": "success",
                    "data": contato
                })),
            );
        }
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "status": "error",
                    "detail": err.to_string()
                })),
            );
        }
    }

    /*
    let resultado = agrupa_items(itens);
    println!("{}", serde_json::to_string_pretty(&resultado).unwrap());
     */
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

/*
==========================================

----------------- Regiao ---------------
==========================================

*/

pub async fn list_regiao(
    State(state): State<SharedState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {
    let service = RegiaoService::new();

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

            match state.templates.get_template("externo/regiao_list.html") {
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
            debug!("Erro ao buscar regiões: {}", err);
            let redirect_url = helpers::create_flash_url(
                "/",
                &format!("Erro ao carregar regiões: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&redirect_url).into_response()
        }
    }
}


pub async fn regiao_form(
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

    match state.templates.get_template("externo/regiao_form.html") {
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

pub async fn create_regiao(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Form(mut body): Form<CreateRegiaoSchema>,
) -> impl IntoResponse {
    if !current_user.current_user.is_superuser {
        let flash_url = helpers::create_flash_url(
            &format!("/externo/regiao"),
            &"Você não tem permissão para criar uma região".to_string(),
            FlashStatus::Error,
        );
        return Redirect::to(&flash_url).into_response();
    }

    let service = RegiaoService::new();

    match service.create(&*state.db, body).await {
        Ok(regiao) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/regiao-form/{}", regiao.id),
                "Região criada com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/regiao-form",
                &format!("Erro ao criar região: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}


pub async fn get_regiao(
    State(state): State<SharedState>,
    Path(id): Path<i32>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Html<String>, impl IntoResponse> {
    let service = RegiaoService::new();

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
    let template = match state.templates.get_template("externo/regiao_form.html") {
        Ok(t) => t,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Falha ao carregar template: {}", err),
            )
                .into_response());
        }
    };

    let regiao = match service.get_by_id(&state.db, id).await {
        Ok(p) => p,
        Err(e) => {
            debug!("Erro ao buscar regiao: {}", e);
            let flash_url = helpers::create_flash_url(
                "/externo/regiao-form",
                &format!("regiao não encontrada: {}", e),
                FlashStatus::Error,
            );
            return Err(Redirect::to(&flash_url).into_response());
        }
    };

    // Preparar o contexto
    let ctx = context! {
        row => regiao,
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

pub async fn update_regiao(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
    Form(input): Form<UpdateRegiaoSchema>,
) -> impl IntoResponse {
    let service = RegiaoService::new();

    match service.update(&*state.db, id, input).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/regiao-form/{}", id),
                &format!("Região atualizada com sucesso!"),
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/regiao"),
                &format!("Erro ao atualizar região: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_regiao(
    State(state): State<SharedState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let service = RegiaoService::new();

    match service.delete(&*state.db, id).await {
        Ok(()) => {
            let flash_url = helpers::create_flash_url(
                "/externo/regiao",
                "Região excluída com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/regiao",
                &format!("Erro ao excluir região: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}


/*
api regiao

*/
pub async fn regiao_api(
    Query(q): Query<PaginationQuery>,
    State(state): State<SharedState>,
) -> Result<Json<PaginatedResponse<Regiao>>, StatusCode> {
    let service = RegiaoService::new();
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

//===========================
// Gestão Regiao
//===========================

/* 
id: id da regiao regiao
*/
pub async fn get_gestao_regiao(
    Query(params): Query<ListParams>,
    Query(form): Query<IdParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let service_regiao = RegiaoService::new();
    let service = RegiaoCidadesService::new();

    let regiao;

    let mut context = minijinja::context! {};

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

    match form.id {
        Some(id) => {
            regiao = Some(
                service_regiao
                    .get_by_id(&*state.db, id as i32)
                    .await
                    .unwrap(),
            );
            let result = service
                .get_paginated_by_regiao_id(
                    &state.db,
                    regiao.clone().unwrap().id,
                    params.page.unwrap_or(1),
                    params.page_size.unwrap_or(10),
                )
                .await;

            match result {
                Ok(paginated_response) => {
                    context = minijinja::context! {
                        regiao => Some(regiao),
                        rows => paginated_response.data,
                        current_page => paginated_response.page,
                        total_pages => paginated_response.total_pages,
                        page_size => paginated_response.page_size,
                        total_records => paginated_response.total_records,
                        find => params.find.unwrap_or_default(),
                        flash_message => flash_message,
                        flash_status => flash_status,
                    };
                }
                Err(_err) => {
                    context = minijinja::context! {
                        regiao => Some(regiao)
                    };
                }
            }
        }
        None => (),
    }

    match state.templates.get_template("externo/regiao_gestao.html") {
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

pub async fn create_gestao_regiao(
    State(state): State<SharedState>,
    Form(body): Form<CreateRegiaoCidades>,
) -> Response {
    let service = RegiaoCidadesService::new();

    let id: i32 = body.regiao_id;

    match service.create(&state.db, body).await {
        Ok(regiao_cidades) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/regiao-gestao?id={}", id.to_string()),
                "Municipio adicionada com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/regiao-gestao",
                &format!("Erro ao adicionar municipio: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_gestao_regiao(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Response {
    let service = RegiaoCidadesService::new();

    match service.delete(&state.db, id.try_into().unwrap()).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                "/externo/regiao-gestao",
                "Municipio removido com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/regiao-gestao",
                &format!("Erro ao remover região: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}


//===========================
// Regiao por usuario
//===========================

/* 
id: id do usuario
*/
pub async fn get_regiao_por_usuario(
    Query(params): Query<ListParams>,
    Query(form): Query<IdParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let service_user = UserService::new();
    let service = UserRegiaoService::new();

    let usuario;

    let mut context = minijinja::context! {};

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

    match form.id {
        Some(id) => {
            usuario = Some(
                service_user
                    .get_by_id(&*state.db, id)
                    .await
                    .unwrap(),
            );
            let result = service
                .get_paginated_by_user_id(
                    &state.db,
                    usuario.clone().unwrap().id as i32,
                    params.page.unwrap_or(1),
                    params.page_size.unwrap_or(10),
                )
                .await;

            match result {
                Ok(paginated_response) => {
                    context = minijinja::context! {
                        usuario => Some(usuario),
                        rows => paginated_response.data,
                        current_page => paginated_response.page,
                        total_pages => paginated_response.total_pages,
                        page_size => paginated_response.page_size,
                        total_records => paginated_response.total_records,
                        find => params.find.unwrap_or_default(),
                        flash_message => flash_message,
                        flash_status => flash_status,
                    };
                }
                Err(_err) => {
                    context = minijinja::context! {
                        usuario => Some(usuario)
                    };
                }
            }
        }
        None => (),
    }

    match state.templates.get_template("externo/regiao_user.html") {
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

pub async fn create_regiao_por_usuario(
    State(state): State<SharedState>,
    Form(body): Form<CreateUserRegiao>,
) -> Response {
    let service = UserRegiaoService::new();

    let id: i32 = body.regiao_id;

    match service.create(&state.db, body).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/regiao-user?id={}", id.to_string()),
                "Região adicionada com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/regiao-user",
                &format!("Erro ao adicionar região: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_regiao_por_usuario(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Response {
    let service = UserRegiaoService::new();

    match service.delete(&state.db, id.try_into().unwrap()).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                "/externo/regiao-user",
                "Região removida com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/regiao-user",
                &format!("Erro ao remover região: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}


//===========================
// linha por usuario
//===========================

/* 
id: id do usuario
*/
pub async fn get_linha_por_usuario(
    Query(params): Query<ListParams>,
    Query(form): Query<IdParams>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let service_user = UserService::new();
    let service = UserLinhaService::new();

    let usuario;

    let mut context = minijinja::context! {};

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

    match form.id {
        Some(id) => {
            usuario = Some(
                service_user
                    .get_by_id(&*state.db, id)
                    .await
                    .unwrap(),
            );
            let result = service
                .get_paginated_by_user_id(
                    &state.db,
                    usuario.clone().unwrap().id as i32,
                    params.page.unwrap_or(1),
                    params.page_size.unwrap_or(10),
                )
                .await;

            match result {
                Ok(paginated_response) => {
                    context = minijinja::context! {
                        usuario => Some(usuario),
                        rows => paginated_response.data,
                        current_page => paginated_response.page,
                        total_pages => paginated_response.total_pages,
                        page_size => paginated_response.page_size,
                        total_records => paginated_response.total_records,
                        find => params.find.unwrap_or_default(),
                        flash_message => flash_message,
                        flash_status => flash_status,
                    };
                }
                Err(_err) => {
                    context = minijinja::context! {
                        usuario => Some(usuario)
                    };
                }
            }
        }
        None => (),
    }

    match state.templates.get_template("externo/linha_user.html") {
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

pub async fn create_linha_por_usuario(
    State(state): State<SharedState>,
    Form(body): Form<CreateUserLinha>,
) -> Response {
    let service = UserLinhaService::new();

    let id: i32 = body.linha_id;

    match service.create(&state.db, body).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                &format!("/externo/linha-user?id={}", id.to_string()),
                "Linha adicionada com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/linha-user",
                &format!("Erro ao adicionar linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

pub async fn delete_linha_por_usuario(
    State(state): State<SharedState>,
    Path(id): Path<i64>,
) -> Response {
    let service = UserLinhaService::new();

    match service.delete(&state.db, id.try_into().unwrap()).await {
        Ok(_) => {
            let flash_url = helpers::create_flash_url(
                "/externo/linha-user",
                "Linha removida com sucesso!",
                FlashStatus::Success,
            );
            Redirect::to(&flash_url).into_response()
        }
        Err(err) => {
            let flash_url = helpers::create_flash_url(
                "/externo/linha-user",
                &format!("Erro ao remover linha: {}", err),
                FlashStatus::Error,
            );
            Redirect::to(&flash_url).into_response()
        }
    }
}

