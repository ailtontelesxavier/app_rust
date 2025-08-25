use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Representa uma Unidade Federativa (UF)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Uf {
    pub id: i64,
    pub sigla: String,
    pub nome: String,
}

/// Representa um Município
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Municipio {
    pub id: i64,
    pub nome: String,
    pub uf_id: i64,
}
