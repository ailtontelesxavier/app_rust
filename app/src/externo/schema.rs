use crate::utils::serde_utils::bool_from_str;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLinhaSchema {
    pub nome: String,
    #[serde(default, deserialize_with = "bool_from_str")]
    pub permite_cnpj: bool,
    #[serde(default, deserialize_with = "bool_from_str")]
    pub permite_cpf: bool,
    #[serde(default, deserialize_with = "bool_from_str")]
    pub permite_avalista: bool,
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
    pub valor_maximo: BigDecimal,
}
