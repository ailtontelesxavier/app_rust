use crate::utils::{serde_utils::bool_from_str, validator::EMAIL_RX};
use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

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
/*
utilizado para saber os documento utilizados pelo tipo de contato se obrigatorio ou nao

*/
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentoRequerido {
    pub id: &'static str,
    pub nome: &'static str,
    pub obrigatorio: bool,
}

/*
Documento para pronaf
*/
pub static DOC_PRONAF: &[DocumentoRequerido; 14] = &[
    DocumentoRequerido {
        id: "rg_frente_solicitante",
        nome: "RG FRENTE SOLICITANTE",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "cpf_frente_solicitante",
        nome: "CPF FRENTE DO SOLICITANTE",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "rg_verso_solicitante",
        nome: "RG VERSO DO SOLICITANTE",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "cpf_verso_solicitante",
        nome: "CPF VERSO DO SOLICITANTE",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "rg_frente_conjuge",
        nome: "RG FRENTE DO CÔNJUGE",
        obrigatorio: false,
    },
    DocumentoRequerido {
        id: "cpf_frente_conjuge",
        nome: "CPF FRENTE DO CÔNJUGE",
        obrigatorio: false,
    },
    DocumentoRequerido {
        id: "rg_verso_conjuge",
        nome: "RG VERSO DO CÔNJUGE",
        obrigatorio: false,
    },
    DocumentoRequerido {
        id: "cpf_verso_conjuge",
        nome: "CPF VERSO DO CÔNJUGE",
        obrigatorio: false,
    },
    DocumentoRequerido {
        id: "caf",
        nome: "CAF - CADASTRO DA AGRICULTURA FAMILIAR",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "comprovante_endereco",
        nome: "COMPROVANTE DE ENDEREÇO",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "foto_agricultor",
        nome: "FOTO DO AGRICULTOR NO EMPREENDIMENTO",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "selfie_documento",
        nome: "SELFIE COM O DOCUMENTO DE IDENTIFICAÇÃO (RG OU CNH)",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "certidao_nascimento_casamento_divorcio",
        nome: "CERTIDÃO DE NASCIMENTO/CASAMENTO/DIVÓRCIO",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "outros",
        nome: "OUTROS DOCUMENTOS",
        obrigatorio: false,
    },
];

/*
documentos para micro credito online
*/
pub static DOC_MICRO_CREDITO: &[DocumentoRequerido; 1] = &[DocumentoRequerido {
    id: "rg_frente_solicitante",
    nome: "RG FRENTE SOLICITANTE",
    obrigatorio: true,
}];
/*
documento capital de giro turismo
*/
pub static DOC_CAPITAL: &[DocumentoRequerido; 1] = &[DocumentoRequerido {
    id: "rg_frente_solicitante",
    nome: "RG FRENTE SOLICITANTE",
    obrigatorio: true,
}];
/*
documento para credito popular
*/
pub static DOC_POPULAR: &[DocumentoRequerido; 2] = &[
    DocumentoRequerido {
        id: "rg_frente_solicitante",
        nome: "RG FRENTE SOLICITANTE",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "cpf_frente_solicitante",
        nome: "CPF FRENTE DO SOLICITANTE",
        obrigatorio: true,
    },
];

/*
documento para agricultura familiar
*/
pub static DOC_AGRICULTURA: &[DocumentoRequerido; 2] = &[
    DocumentoRequerido {
        id: "rg_frente_solicitante",
        nome: "RG FRENTE SOLICITANTE",
        obrigatorio: true,
    },
    DocumentoRequerido {
        id: "cpf_frente_solicitante",
        nome: "CPF FRENTE DO SOLICITANTE",
        obrigatorio: true,
    },
];

/*
documentos para emergencial
*/
pub static DOC_EMERGINCIAL: &[DocumentoRequerido; 1] = &[DocumentoRequerido {
    id: "foto_agricultor",
    nome: "FOTO DO AGRICULTOR NO EMPREENDIMENTO",
    obrigatorio: true,
}];
/*
documento para maos que criam
*/
pub static DOC_MAOS_QUE: &[DocumentoRequerido; 1] = &[DocumentoRequerido {
    id: "comprovante_endereco",
    nome: "COMPROVANTE DE ENDEREÇO",
    obrigatorio: true,
}];
/*
documentos credito online
*/
pub static DOC_ONLINE: &[DocumentoRequerido; 1] = &[DocumentoRequerido {
    id: "rg_frente_solicitante",
    nome: "RG FRENTE SOLICITANTE",
    obrigatorio: true,
}];

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
    pub dados_imports: Option<Value>,
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
pub struct CreateContatoSchema {
    #[validate(length(min = 1, message = " não pode ser vazio"))]
    pub cpf_cnpj: String,
    #[validate(length(min = 1, message = "nome não pode ser vazio"))]
    pub nome: String,
    #[validate(length(min = 1, message = "telefone não pode ser vazio"))]
    pub telefone: String,
    #[validate(regex(path=*EMAIL_RX, message = "Invalid email"))]
    pub email: String,
    #[validate(range(min = 1, message = "cidade_id não localizada"))]
    pub cidade_id: i64,
    pub val_solicitado: BigDecimal,
}

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct ContatoSchema {
    pub cpf_cnpj: String,
    pub nome: String,
    pub telefone: String,
    pub email: String,
    pub cidade_id: i64,
    pub val_solicitado: BigDecimal,

    #[serde(flatten)]
    pub tipo: TipoContatoExtra,
    //pub documentos: Vec<DocumentoRequerido>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type_contato", rename_all = "snake_case")]
pub enum TipoContatoExtra {
    PronafB(PronafB),
    MicroCreditoOnline(MicroCreditoOnline),
    CreditoPopular(CreditoPopular),
    MaosQueCriam(MaosQueCriam),
    CreditoOnline(CreditoOnline),
}

#[derive(Debug, Validate, Default, Clone, Serialize, Deserialize)]
pub struct PronafB {
    pub nome_tecnico: String,
    pub orgao_associacao_tecnico: String,
    pub telefone_whatsapp_tecnico: String,
    pub apelido: Option<String>, //do solicitante
    pub estado_civil: i32,
    #[validate(length(min = 9, max = 9, message = "Cep invalido"))]
    pub cep: String, //len 9
    pub endereco: String,
    pub prev_aumento_fat: BigDecimal,
    pub cpf_conj: Option<String>,
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

/*
Aplicacao de recursos
*/
#[derive(Debug, Validate, Clone, Serialize, Deserialize)]
pub struct AplicacaoRecursos {
    pub descricao: String,
    pub quantidade: i32,
    pub valor_unitario: BigDecimal,
    pub valor_total: BigDecimal,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRegiaoSchema {
    pub name: String,
    pub municipio_id: i32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRegiaoSchema {
    pub id: i32,
    pub name: String,
    pub municipio_id: i32
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRegiaoCidades {
    pub regiao_id: i32,
    pub municipio_id: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRegiaoCidades {
    pub id: i32,
    pub regiao_id: i32,
    pub municipio_id: i32,
}


