use serde::{Deserialize, Serialize};
use sqlx::postgres::{PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};

use crate::core::StatusOpt;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum StatusChamado {
    Aberto = 0 as i32,
    EmAtendimento = 1 as i32,
    Pausado = 2 as i32,
    Resolvido = 3 as i32,
    Fechado = 4 as i32,
}

impl TryFrom<i32> for StatusChamado {
    type Error = &'static str;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(StatusChamado::Aberto),
            1 => Ok(StatusChamado::EmAtendimento),
            2 => Ok(StatusChamado::Pausado),
            3 => Ok(StatusChamado::Resolvido),
            4 => Ok(StatusChamado::Fechado),
            _ => Err("Status inválido"),
        }
    }
}

impl StatusChamado {
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => StatusChamado::Aberto,
            1 => StatusChamado::EmAtendimento,
            2 => StatusChamado::Pausado,
            3 => StatusChamado::Resolvido,
            4 => StatusChamado::Fechado,
            _ => StatusChamado::Aberto,
        }
    }

    pub fn to_i32(&self) -> i32 {
        *self as i32
    }
}

impl<'r> Decode<'r, Postgres> for StatusChamado {
    fn decode(value: PgValueRef<'r>) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let v: i32 = <i32 as Decode<Postgres>>::decode(value)?;
        Ok(StatusChamado::try_from(v)?)
    }
}

impl<'q> Encode<'q, Postgres> for StatusChamado {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<sqlx::encode::IsNull, Box<dyn std::error::Error + Send + Sync>> {
        <i32 as sqlx::Encode<'_, Postgres>>::encode_by_ref(&(*self as i32), buf)
    }
}

impl Type<Postgres> for StatusChamado {
    fn type_info() -> PgTypeInfo {
        <i32 as Type<Postgres>>::type_info()
    }
}

impl std::fmt::Display for StatusChamado {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            StatusChamado::Aberto => "Aberto",
            StatusChamado::EmAtendimento => "Em Atendimento",
            StatusChamado::Pausado => "Pausado",
            StatusChamado::Resolvido => "Resolvido",
            StatusChamado::Fechado => "Fechado",
        };
        write!(f, "{}", str)
    }
}

impl StatusChamado {
    pub fn all() -> Vec<Self> {
        vec![
            StatusChamado::Aberto,
            StatusChamado::EmAtendimento,
            StatusChamado::Pausado,
            StatusChamado::Resolvido,
            StatusChamado::Fechado,
        ]
    }

    /*
    para o select do html
    */
    pub fn status_options() -> Vec<StatusOpt> {
        StatusChamado::all()
            .into_iter()
            .map(|s| StatusOpt {
                value: s as i32,      // valor do <option>
                label: s.to_string(), // usa impl Display p/ rótulo
            })
            .collect()
    }
}