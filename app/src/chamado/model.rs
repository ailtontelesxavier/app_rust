use sqlx::FromRow;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TipoChamado {
    pub id: i64,
    pub nome: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct ServicoChamado {
    pub id: i64,
    pub nome: String,
    pub tipo_id: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CategoriaChamado {
    pub id: i64,
    pub nome: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Chamado {
    pub id: i64,
    pub titulo: String,
    pub descricao: String,
    pub status: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub user_solic_id: i64,
    pub servico_id: i64,
    pub tipo_id: i64,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct GerenciamentoChamado {
    pub id: i64,
    pub descricao: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub categoria_id: Option<i64>,
    pub chamado_id: i64,
    pub user_atend_id: i64,
    pub observacao_chamado: Option<String>,
}
