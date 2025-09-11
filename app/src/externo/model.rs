use bigdecimal::BigDecimal;
use chrono::Datelike;
use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

use crate::externo::enums;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Linha {
    pub id: i32,
    pub nome: String,
    pub permite_cnpj: bool,
    pub permite_cpf: bool,
    pub permite_avalista: bool,
    //#[serde(with = "bigdecimal::serde::json_num")]
    pub valor_maximo: BigDecimal,
}
/*


*/
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Contato {
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

impl Contato {
    pub fn gerar_codigo_protocolo() -> String {
        let ano_atual = Local::now().year();
        let numero_protocolo = format!("{}{:06}", ano_atual, rand::random::<u32>() % 1_000_000);
        numero_protocolo
    }
}

/*

1 = Micro Crédito Online
2 = Capital de Giro Turismo
3 = Crédito Popular
4 = Agricultura Familiar
5 = Emergencial
6 = Mãos que Criam
7 = Crédito Online

*/
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Campos {
    MicroCreditoOnline {
        faturamento_bruto_mensal: BigDecimal,
        porte_empresa: i32,
        custo_mensal: BigDecimal,
        tipo_conta: enums::TipoContaEnum,
        banco: String,
        agencia: String,
        conta_corrente: String,
        finalidade_credito: String,
        atividade: String,
        descricao_despesa: String,
    },
    CapitalGiroTurismo {},
    CreditoPopular {
        faturamento_bruto_mensal: BigDecimal,
        porte_empresa: i32,
        custo_mensal: BigDecimal,
        tipo_conta: enums::TipoContaEnum,
        banco: String,
        agencia: String,
        conta_corrente: String,
        finalidade_credito: String,
        atividade: String,
        descricao_despesa: String,
    },
    AgriculturaFamiliar {},
    Emergencial {},
    MaosQueCriam {
        faturamento_bruto_mensal: BigDecimal,
        porte_empresa: i32,
        custo_mensal: BigDecimal,
        tipo_conta: enums::TipoContaEnum,
        banco: String,
        agencia: String,
        conta_corrente: String,
        finalidade_credito: String,
        nome_conj: String,
        telefone_conj: String,
        email_conj: String,
    },
    CreditoOnline {
        faturamento_bruto_mensal: BigDecimal,
        porte_empresa: i32,
        custo_mensal: BigDecimal,
        tipo_conta: enums::TipoContaEnum,
        banco: String,
        agencia: String,
        conta_corrente: String,
        finalidade_credito: String,
        atividade: String,
        descricao_despesa: String,
    },
    PronafB {
        nome_tecnico: String,
        orgao_associacao_tecnico: String,
        telefone_whatsapp_tecnico: String,
        apelido: Option<String>, //do solicitante
        estado_civil: i32,
        cidade_id: i32,
        cep: String, //len 9
        endereco: String,
        prev_aumento_fat: BigDecimal,
        nome_conj: Option<String>,
        telefone_conj: Option<String>,
        email_conj: Option<String>,
        email: Option<String>,
        valor_estimado_imovel: Option<BigDecimal>,
        desc_atividade: String,
        finalidade_credito: String,
    },
    //#[serde(other)]
    //Outro(Value), // pega qualquer coisa não mapeada
}

/*
utilizado para o credito pronaf b
*/
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AplicacaoRecurso {
    pub id: i64,
    pub descricao: String,
    pub quantidade: i32,
    pub valor_unitario: BigDecimal,
    pub valor_total: BigDecimal,
    pub contato_id: Uuid,
}

/*
armazenar arquivos dos documentos enviados das solicitacoes de contato
*/
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocSolicitante {
    pub id: i64,
    pub contato_id: Uuid,
    pub arquivo: String,
    pub status_arquivo: String,
    pub observacao: String,
    pub tipo: String,
}

/*
 Tipos de documentos cliente enviar contato
*/
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TipoDocumento {
    pub id: i32,
    pub nome: String,
    pub descricao: String,
}

/*
 utilizado para o tipo de documento de contato
 quais documentos obrigatorios enviar
*/
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TipoDocContato {
    pub id: i32,
    pub tipo_documento_id: i32,
    pub contato_id: Uuid,
    pub obrigatorio: bool,
    pub ativo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContatoSocio {
    pub id: i64,
    pub contato_id: Uuid,
    pub nome: String,
    pub cpf: String,
    pub telefone: String,
    pub email: String,
    pub exporta_politicamente: bool,
    pub nome_conj: String,
    pub cpf_conj: String,
    pub telefone_conj: String,
    pub email_conj: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ContatoAvalista {
    pub id: i64,
    pub contato_id: Uuid,
    pub nome: String,
    pub cpf: String,
    pub telefone: String,
    pub email: String,
    pub exporta_politicamente: bool,
    pub nome_conj: String,
    pub cpf_conj: String,
    pub telefone_conj: String,
    pub email_conj: String,
}
