use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};

use crate::core::StatusOpt;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum StatusCivil {
    Solteiro = 1 as i32,
    Casado = 2 as i32,
    Divorciado = 3 as i32,
    Viuvo = 4 as i32,
    Separado = 5 as i32,
    UniaoEstavel = 6 as i32,
}

impl std::fmt::Display for StatusCivil {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StatusCivil::Solteiro => "Solteiro",
            StatusCivil::Casado => "Casado",
            StatusCivil::Divorciado => "Divorciado",
            StatusCivil::Viuvo => "Viuvo",
            StatusCivil::Separado => "Separado",
            StatusCivil::UniaoEstavel => "Uniao Estavel",
        };
        write!(f, "{}", str)
    }
}

impl StatusCivil {
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => StatusCivil::Solteiro,
            2 => StatusCivil::Casado,
            3 => StatusCivil::Divorciado,
            4 => StatusCivil::Viuvo,
            5 => StatusCivil::Separado,
            6 => StatusCivil::UniaoEstavel,
            _ => StatusCivil::Solteiro,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

/*
    MEI = 1, "MEI"
    ME = 2, "ME"
    AUTONOMO_PROFISSIONAL_LIBERAL = 3, "Autônomo/Profissional Liberal"
    OUTROS = 4, "Outros"
*/
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum PorteEmpresaEnum {
    MEI = 1 as i32,
    ME = 2 as i32,
    OUTROS = 4 as i32,
}

impl std::fmt::Display for PorteEmpresaEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            PorteEmpresaEnum::MEI => "MEI",
            PorteEmpresaEnum::ME => "ME",
            //PorteEmpresaEnum::AUTONOMO_PROFISSIONAL_LIBERAL => "Autônomo/Profissional Liberal",
            PorteEmpresaEnum::OUTROS => "Outros",
        };
        write!(f, "{}", str)
    }
}

impl PorteEmpresaEnum {
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => PorteEmpresaEnum::MEI,
            2 => PorteEmpresaEnum::ME,
            //3 => PorteEmpresaEnum::AUTONOMO_PROFISSIONAL_LIBERAL,
            4 => PorteEmpresaEnum::OUTROS,
            _ => PorteEmpresaEnum::OUTROS,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TipoContaEnum {
    Fisica = 1 as i32,
    Juridica = 2 as i32,
}

impl std::fmt::Display for TipoContaEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TipoContaEnum::Fisica => "Pessoa Física",
            TipoContaEnum::Juridica => "Pessoa Jurídica",
        };
        write!(f, "{}", str)
    }
}

impl TipoContaEnum {
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => TipoContaEnum::Fisica,
            2 => TipoContaEnum::Juridica,
            _ => TipoContaEnum::Fisica,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum StatusTramitacaoEnum {
    AGUARDANDO_ATENDIMENTO = 1 as i32,
    VOCE_ESTA_QUASE_LA_ENVIO_DOCUMENTOS = 2 as i32,
    EM_CAMINHADO_P_PROSPECTOR = 3 as i32,
    FALTANDO_DOCUMENTO = 4 as i32,
    CADASTRO = 5 as i32,
    APROVADO = 6 as i32,
    CONTRATACAO = 7 as i32,
    REPROVADO = 8 as i32,
}

impl std::fmt::Display for StatusTramitacaoEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StatusTramitacaoEnum::AGUARDANDO_ATENDIMENTO => "Aguardando Atendimento",
            StatusTramitacaoEnum::VOCE_ESTA_QUASE_LA_ENVIO_DOCUMENTOS => {
                "Você está quase lá, envie documentos"
            }
            StatusTramitacaoEnum::EM_CAMINHADO_P_PROSPECTOR => "Em caminhado para prospector",
            StatusTramitacaoEnum::FALTANDO_DOCUMENTO => "Faltando documento",
            StatusTramitacaoEnum::CADASTRO => "Cadastro",
            StatusTramitacaoEnum::APROVADO => "Aprovado",
            StatusTramitacaoEnum::CONTRATACAO => "Contratação",
            StatusTramitacaoEnum::REPROVADO => "Reprovado",
        };
        write!(f, "{}", str)
    }
}

impl StatusTramitacaoEnum {
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => StatusTramitacaoEnum::AGUARDANDO_ATENDIMENTO,
            2 => StatusTramitacaoEnum::VOCE_ESTA_QUASE_LA_ENVIO_DOCUMENTOS,
            3 => StatusTramitacaoEnum::EM_CAMINHADO_P_PROSPECTOR,
            4 => StatusTramitacaoEnum::FALTANDO_DOCUMENTO,
            5 => StatusTramitacaoEnum::CADASTRO,
            6 => StatusTramitacaoEnum::APROVADO,
            7 => StatusTramitacaoEnum::CONTRATACAO,
            8 => StatusTramitacaoEnum::REPROVADO,
            _ => StatusTramitacaoEnum::AGUARDANDO_ATENDIMENTO,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum StatusAtendimentoEnum {
    PENDENTE = 0 as i32,
    EM_ATENDIMENTO = 1 as i32,
    AGUARDANDO_DOCUMENTO = 2 as i32,
    APROVADO = 3 as i32,
    CONCLUIDO = 4 as i32,
    CANCELADO = 5 as i32,
}

impl std::fmt::Display for StatusAtendimentoEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StatusAtendimentoEnum::PENDENTE => "Pendente",
            StatusAtendimentoEnum::EM_ATENDIMENTO => "Em Atendimento",
            StatusAtendimentoEnum::AGUARDANDO_DOCUMENTO => "Aguardando Documento",
            StatusAtendimentoEnum::APROVADO => "Aprovado",
            StatusAtendimentoEnum::CONCLUIDO => "Concluído",
            StatusAtendimentoEnum::CANCELADO => "Cancelado",
        };
        write!(f, "{}", str)
    }
}

impl StatusAtendimentoEnum {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => StatusAtendimentoEnum::PENDENTE,
            1 => StatusAtendimentoEnum::EM_ATENDIMENTO,
            2 => StatusAtendimentoEnum::AGUARDANDO_DOCUMENTO,
            3 => StatusAtendimentoEnum::APROVADO,
            4 => StatusAtendimentoEnum::CONCLUIDO,
            5 => StatusAtendimentoEnum::CANCELADO,
            _ => StatusAtendimentoEnum::PENDENTE,
        }
    }
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

/*
utilizando nos documentos enviados
na solicitação de crédito

*/
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum StatusDocumentoEnum {
    PENDENTE = 0 as i32,
    EM_ANALISE = 1 as i32,
    APROVADO = 2 as i32,
    REPROVADO = 3 as i32,
}

impl std::fmt::Display for StatusDocumentoEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StatusDocumentoEnum::PENDENTE => "Pendente",
            StatusDocumentoEnum::EM_ANALISE => "Em Análise",
            StatusDocumentoEnum::APROVADO => "Aprovado",
            StatusDocumentoEnum::REPROVADO => "Reprovado",
        };
        write!(f, "{}", str)
    }
}

impl StatusDocumentoEnum {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => StatusDocumentoEnum::PENDENTE,
            1 => StatusDocumentoEnum::EM_ANALISE,
            2 => StatusDocumentoEnum::APROVADO,
            3 => StatusDocumentoEnum::REPROVADO,
            _ => StatusDocumentoEnum::PENDENTE,
        }
    }
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

/*
 tipo documentos
*/
