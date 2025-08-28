use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTipoChamadoSchema {
    pub nome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTipoChamadoSchema {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct ServicoTipoViewSchema {
    pub id: i64,
    pub nome: String,
    pub tipo_id: i64,
    pub nome_tipo: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateCategoriaChamadoSchema {
    pub nome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateCategoriaChamadoSchema {
    pub nome: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChamado {
    pub titulo: String,
    pub status: Option<i32>,
    pub user_solic_id: Option<i64>,
    pub servico_id: i64,
    pub tipo_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateChamado {
    pub titulo: Option<String>,
    pub descricao: Value,
    pub servico_id: Option<i64>,
    pub tipo_id: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGerenciamentoChamado {
    pub descricao: Option<String>,
    pub categoria_id: Option<i64>,
    pub chamado_id: i64,
    pub user_atend_id: i64,
    pub observacao_chamado: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGerenciamentoChamado {
    pub descricao: Option<String>, //descriçao interna
    pub status: i32,               //status para alterar no chamado
    pub categoria_id: i64,
    pub observacao_chamado: Option<String>, //observaçao para o usuario do chamado
}
