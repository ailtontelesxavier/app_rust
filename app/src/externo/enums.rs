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

impl StatusCivil {
    pub fn all() -> Vec<Self> {
        vec![
            StatusCivil::Solteiro,
            StatusCivil::Casado,
            StatusCivil::Divorciado,
            StatusCivil::Viuvo,
            StatusCivil::Separado,
            StatusCivil::UniaoEstavel,
        ]
    }

    /*
    para o select do html
    */
    pub fn estado_civil_options() -> Vec<StatusOpt> {
        StatusCivil::all()
            .into_iter()
            .map(|s| StatusOpt {
                value: s as i32,      // valor do <option>
                label: s.to_string(), // usa impl Display p/ rótulo
            })
            .collect()
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
    AguardandoAtendimento = 1 as i32,
    VoceEstaQuaseLaEnvioDocumentos = 2 as i32,
    EmCaminhadoPProspector = 3 as i32,
    FaltandoDocumento = 4 as i32,
    Cadastro = 5 as i32,
    Aprovado = 6 as i32,
    Contratacao = 7 as i32,
    Reprovado = 8 as i32,
    EmAtendimento = 9 as i32,
    EmAnaliseFinanceira = 10 as i32,
}

impl std::fmt::Display for StatusTramitacaoEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StatusTramitacaoEnum::AguardandoAtendimento => "Aguardando Atendimento",
            StatusTramitacaoEnum::VoceEstaQuaseLaEnvioDocumentos => {
                "Você está quase lá, envie documentos"
            }
            StatusTramitacaoEnum::EmCaminhadoPProspector => "Em caminhado para prospector",
            StatusTramitacaoEnum::FaltandoDocumento => "Faltando documento",
            StatusTramitacaoEnum::Cadastro => "Cadastro",
            StatusTramitacaoEnum::Aprovado => "Aprovado",
            StatusTramitacaoEnum::Contratacao => "Contratação",
            StatusTramitacaoEnum::Reprovado => "Reprovado",
            StatusTramitacaoEnum::EmAtendimento => "Em Atendimento",
            StatusTramitacaoEnum::EmAnaliseFinanceira => "Em Análise Financeira",
        };
        write!(f, "{}", str)
    }
}

impl StatusTramitacaoEnum {
    pub fn from_i32(value: i32) -> Self {
        match value {
            1 => StatusTramitacaoEnum::AguardandoAtendimento,
            2 => StatusTramitacaoEnum::VoceEstaQuaseLaEnvioDocumentos,
            3 => StatusTramitacaoEnum::EmCaminhadoPProspector,
            4 => StatusTramitacaoEnum::FaltandoDocumento,
            5 => StatusTramitacaoEnum::Cadastro,
            6 => StatusTramitacaoEnum::Aprovado,
            7 => StatusTramitacaoEnum::Contratacao,
            8 => StatusTramitacaoEnum::Reprovado,
            9 => StatusTramitacaoEnum::EmAtendimento,
            10 => StatusTramitacaoEnum::EmAnaliseFinanceira,
            _ => StatusTramitacaoEnum::AguardandoAtendimento,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }

    pub fn color(&self) -> &'static str {
        match self {
            StatusTramitacaoEnum::AguardandoAtendimento => "bg-orange-300",
            StatusTramitacaoEnum::Cadastro => "bg-green-300",
            StatusTramitacaoEnum::Contratacao => "bg-blue-500",
            StatusTramitacaoEnum::FaltandoDocumento => "bg-red-300",
            StatusTramitacaoEnum::VoceEstaQuaseLaEnvioDocumentos => "bg-blue-400",
            _ => "",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum StatusAtendimentoEnum {
    Pendente = 0 as i32,
    EmAtendimento = 1 as i32,
    AguardandoDocumento = 2 as i32,
    Aprovado = 3 as i32,
    Concluido = 4 as i32,
    Cancelado = 5 as i32,
}

impl std::fmt::Display for StatusAtendimentoEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StatusAtendimentoEnum::Pendente => "Pendente",
            StatusAtendimentoEnum::EmAtendimento => "Em Atendimento",
            StatusAtendimentoEnum::AguardandoDocumento => "Aguardando Documento",
            StatusAtendimentoEnum::Aprovado => "Aprovado",
            StatusAtendimentoEnum::Concluido => "Concluído",
            StatusAtendimentoEnum::Cancelado => "Cancelado",
        };
        write!(f, "{}", str)
    }
}

impl StatusAtendimentoEnum {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => StatusAtendimentoEnum::Pendente,
            1 => StatusAtendimentoEnum::EmAtendimento,
            2 => StatusAtendimentoEnum::AguardandoDocumento,
            3 => StatusAtendimentoEnum::Aprovado,
            4 => StatusAtendimentoEnum::Concluido,
            5 => StatusAtendimentoEnum::Cancelado,
            _ => StatusAtendimentoEnum::Pendente,
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
    Pendente = 0 as i32,
    EmAnalise = 1 as i32,
    Aprovado = 2 as i32,
    Reprovado = 3 as i32,
}

impl std::fmt::Display for StatusDocumentoEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StatusDocumentoEnum::Pendente => "Pendente",
            StatusDocumentoEnum::EmAnalise => "Em Análise",
            StatusDocumentoEnum::Aprovado => "Aprovado",
            StatusDocumentoEnum::Reprovado => "Reprovado",
        };
        write!(f, "{}", str)
    }
}

impl StatusDocumentoEnum {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => StatusDocumentoEnum::Pendente,
            1 => StatusDocumentoEnum::EmAnalise,
            2 => StatusDocumentoEnum::Aprovado,
            3 => StatusDocumentoEnum::Reprovado,
            _ => StatusDocumentoEnum::Pendente,
        }
    }
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

/*
 tipo Contato
    1 = Micro Crédito Online
    2 = Capital de Giro Turismo
    3 = Crédito Popular
    4 = Agricultura Familiar
    5 = Emergencial
    6 = Mãos que Criam
    7 = Crédito Online
    8 = Pronaf B
*/
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum TypeContato {
    MicroCreditoOnline = 1 as i32,
    CapitalDeGiroTurismo = 2 as i32,
    CreditoPopular = 3 as i32,
    AgriculturaFamiliar = 4 as i32,
    Emergencial = 5 as i32,
    MaosQueCriam = 6 as i32,
    CreditoOnline = 7 as i32,
    PronafB = 8 as i32,
}

impl std::fmt::Display for TypeContato {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            TypeContato::MicroCreditoOnline => "Micro Crédito Online",
            TypeContato::CapitalDeGiroTurismo => "Capital de Giro Turismo",
            TypeContato::CreditoPopular => "Crédito Popular",
            TypeContato::AgriculturaFamiliar => "Agricultura Familiar",
            TypeContato::Emergencial => "Emergencial",
            TypeContato::MaosQueCriam => "Mãos que Criam",
            TypeContato::CreditoOnline => "Crédito Online",
            TypeContato::PronafB => "Pronaf B",
        };
        write!(f, "{}", str)
    }
}

impl TypeContato {
    pub fn from_i32(value: i32) -> Option<Self> {
        match value {
            1 => Some(TypeContato::MicroCreditoOnline),
            2 => Some(TypeContato::CapitalDeGiroTurismo),
            3 => Some(TypeContato::CreditoPopular),
            4 => Some(TypeContato::AgriculturaFamiliar),
            5 => Some(TypeContato::Emergencial),
            6 => Some(TypeContato::MaosQueCriam),
            7 => Some(TypeContato::CreditoOnline),
            8 => Some(TypeContato::PronafB),
            _ => None,
        }
    }
    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

impl TypeContato {
    pub fn all() -> Vec<Self> {
        vec![
            TypeContato::MicroCreditoOnline,
            TypeContato::CapitalDeGiroTurismo,
            TypeContato::CreditoPopular,
            TypeContato::AgriculturaFamiliar,
            TypeContato::Emergencial,
            TypeContato::MaosQueCriam,
            TypeContato::CreditoOnline,
            TypeContato::PronafB,
        ]
    }
    /*
    para o select do html
    */
    pub fn tipo_contato_options() -> Vec<StatusOpt> {
        TypeContato::all()
            .into_iter()
            .map(|s| StatusOpt {
                value: s as i32,      // valor do <option>
                label: s.to_string(), // usa impl Display p/ rótulo
            })
            .collect()
    }
}
