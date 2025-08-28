use serde::{Deserialize, Serialize};

/// Schema para criar uma UF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUf {
    pub sigla: String,
    pub nome: String,
}

/// Schema para criar um Município
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMunicipio {
    pub nome: String,
    pub uf_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMunicipio {
    pub nome: Option<String>,
    pub uf_id: i64,
}

/// Resposta de Município com UF (join)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MunicipioWithUf {
    pub id: i64,
    pub nome: String,
    pub uf_id: i64,
    pub uf_sigla: String,
    pub uf_nome: String,
}

/// Estrutura retornada pela API do IBGE para UF
#[derive(Debug, Deserialize)]
#[warn(unused_imports)]
pub struct MunicipioIbge {
    pub id: i64,
    pub nome: String,
    pub microrregiao: Option<Microrregiao>,
}

#[derive(Debug, Deserialize)]
#[warn(unused_imports)]
pub struct Microrregiao {
    pub id: i64,
    pub nome: String,
    pub mesorregiao: Mesorregiao,
}

#[derive(Debug, Deserialize)]
#[warn(unused_imports)]
pub struct Mesorregiao {
    pub id: i64,
    pub nome: String,
    #[serde(rename = "UF")]
    pub uf: UfIbge,
}

#[derive(Debug, Deserialize)]
#[warn(unused_imports)]
pub struct UfIbge {
    pub id: i64,
    pub sigla: String,
    pub nome: String,
    pub regiao: Regiao,
}

#[derive(Debug, Deserialize)]
#[warn(unused_imports)]
pub struct Regiao {
    pub id: i64,
    pub sigla: String,
    pub nome: String,
}



/*
 utilizado para gerar option para o html
*/
#[derive(serde::Serialize)]
pub struct StatusOpt {
    pub value: i32,
    pub label: String,
}
