use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLinhaSchema {
    pub nome: String,
    pub permite_cnpj: bool,
    pub permite_cpf: bool,
    pub permite_avalista: bool,
    pub valor_maximo: BigDecimal,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLinhaSchema {
    pub id: i32,
    pub nome: String,
    pub permite_cnpj: bool,
    pub permite_cpf: bool,
    pub permite_avalista: bool,
    pub valor_maximo: BigDecimal,
}
