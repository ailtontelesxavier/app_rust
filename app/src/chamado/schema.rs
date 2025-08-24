
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTipoChamadoSchema {
    pub nome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateServicoChamadoSchema {
    pub nome: String,
    pub tipo_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateServicoChamadoSchema {
    pub nome: Option<String>,
    pub tipo_id: Option<i64>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoriaChamadoSchema {
    pub nome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChamadoSchema {
    pub titulo: String,
    pub descricao: String,
    pub status: i32,
    pub user_solic_id: i64,
    pub servico_id: i64,
    pub tipo_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChamadoSchema {
    pub titulo: Option<String>,
    pub descricao: Option<String>,
    pub status: Option<i32>,
    pub user_solic_id: Option<i64>,
    pub servico_id: Option<i64>,
    pub tipo_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGerenciamentoChamadoSchema {
    pub descricao: Option<String>,
    pub categoria_id: Option<i64>,
    pub chamado_id: i64,
    pub user_atend_id: i64,
    pub observacao_chamado: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGerenciamentoChamadoSchema {
    pub descricao: Option<String>,
    pub categoria_id: Option<i64>,
    pub chamado_id: Option<i64>,
    pub user_atend_id: Option<i64>,
    pub observacao_chamado: Option<String>,
}