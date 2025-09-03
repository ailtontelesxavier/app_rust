use crate::{
    externo::enums,
    utils::{serde_utils::bool_from_str, validator::EMAIL_RX},
};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLinhaSchema {
    pub nome: String,
    #[serde(default, deserialize_with = "bool_from_str")]
    pub permite_cnpj: bool,
    #[serde(default, deserialize_with = "bool_from_str")]
    pub permite_cpf: bool,
    #[serde(default, deserialize_with = "bool_from_str")]
    pub permite_avalista: bool,
    #[serde(deserialize_with = "crate::utils::serde_utils::brl_to_bigdecimal")]
    pub valor_maximo: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLinhaSchema {
    pub id: i32,
    pub nome: String,
    #[serde(default, deserialize_with = "bool_from_str")]
    pub permite_cnpj: bool,
    #[serde(default, deserialize_with = "bool_from_str")]
    pub permite_cpf: bool,
    #[serde(default, deserialize_with = "bool_from_str")]
    pub permite_avalista: bool,
    #[serde(deserialize_with = "crate::utils::serde_utils::brl_to_bigdecimal")]
    pub valor_maximo: BigDecimal,
}

/*
==========================================

----------------- Contato ---------------
==========================================
*/

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateContato {
    pub linha_id: i32,
    pub protocolo: String,
    pub status_atendimento: bool,
    pub cpf_cnpj: String,
    pub nome: String,
    pub telefone: String,
    pub email: String,
    pub cidade_id: i64,
    pub val_solicitado: BigDecimal,
    pub status_tramitacao: i32,
    pub campos: Value,
    pub dados_imports: Option<Value>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateContato {
    pub id: Uuid,
    pub linha_id: i32,
    pub protocolo: String,
    pub status_atendimento: bool,
    pub cpf_cnpj: String,
    pub nome: String,
    pub telefone: String,
    pub email: String,
    pub cidade_id: i64,
    pub val_solicitado: BigDecimal,
    pub status_tramitacao: i32,
    pub campos: Value,
    pub dados_imports: Option<Value>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct Contato {
    pub cpf_cnpj: String,
    pub nome: String,
    pub telefone: String,
    pub email: String,
    pub cidade_id: i64,
    pub val_solicitado: BigDecimal,
    pub campos: Value,
    pub dados_imports: Option<Value>,
}

#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct PronafB {
    pub nome_tecnico: String,
    pub orgao_associacao_tecnico: String,
    pub telefone_whatsapp_tecnico: String,
    pub apelido: Option<String>, //do solicitante
    #[validate(length(min = 9, max = 9, message = "Cep invalido"))]
    pub cep: String, //len 9
    pub endereco: String,
    pub prev_aumento_fat: BigDecimal,
    pub nome_conj: Option<String>,
    pub telefone_conj: Option<String>,
    pub email_conj: Option<String>,
    #[validate(regex(path=*EMAIL_RX, message = "Invalid email"))]
    pub email: Option<String>,
    pub valor_estimado_imovel: Option<BigDecimal>,
    #[validate(length(min = 8, message = "Detalhe a atividade melhor"))]
    pub desc_atividade: String,
    pub finalidade_credito: String,
}

#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct MicroCreditoOnline {
    pub faturamento_bruto_mensal: BigDecimal,
    pub porte_empresa: i32,
    pub custo_mensal: BigDecimal,
    pub tipo_conta: i32,
    pub banco: String,
    pub agencia: String,
    pub conta_corrente: String,
    pub finalidade_credito: String,
    pub atividade: String,
    pub descricao_despesa: String,
}

#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct CreditoPopular {
    faturamento_bruto_mensal: BigDecimal,
    porte_empresa: i32,
    custo_mensal: BigDecimal,
    tipo_conta: i32,
    banco: String,
    agencia: String,
    conta_corrente: String,
    finalidade_credito: String,
    atividade: String,
    descricao_despesa: String,
}
#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct MaosQueCriam {
    faturamento_bruto_mensal: BigDecimal,
    porte_empresa: i32,
    custo_mensal: BigDecimal,
    tipo_conta: i32,
    banco: String,
    agencia: String,
    conta_corrente: String,
    finalidade_credito: String,
    nome_conj: String,
    telefone_conj: String,
    email_conj: String,
}
#[derive(Debug, Validate, Clone, Serialize, Deserialize)]
pub struct CreditoOnline {
    faturamento_bruto_mensal: BigDecimal,
    porte_empresa: i32,
    custo_mensal: BigDecimal,
    tipo_conta: i32,
    banco: String,
    agencia: String,
    conta_corrente: String,
    finalidade_credito: String,
    atividade: String,
    descricao_despesa: String,
}
