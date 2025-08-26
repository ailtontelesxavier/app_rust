use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::FromRow;
use sqlx::postgres::{PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};

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

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Chamado {
    pub id: i64,
    pub titulo: String,
    pub descricao: Value,
    pub status: Option<i32>,
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
