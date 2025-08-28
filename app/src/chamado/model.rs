use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};
use sqlx::FromRow;
use sqlx::postgres::{PgTypeInfo, PgValueRef};
use sqlx::{Decode, Encode, Postgres, Type};

use crate::core::StatusOpt;

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

#[derive(Debug)]
pub struct ImagemChamado {
    pub url: String,
    pub caption: String,
    pub with_border: bool,
    pub stretched: bool,
    pub with_background: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
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

impl Chamado {
    /// Extrai apenas as imagens da descrição EditorJS
    pub fn extrair_imagens(&self) -> Result<Vec<ImagemChamado>> {
        let mut imagens = Vec::new();

        // Verifica se a descrição é um objeto JSON válido
        let descricao_obj: Value = if let Value::String(s) = &self.descricao {
            from_str(s).map_err(|e| anyhow!("Erro ao parsear JSON: {}", e))?
        } else {
            // Se já for Value, usa diretamente
            self.descricao.clone()
        };

        // Obtém o array de blocks
        let blocks = descricao_obj
            .get("blocks")
            .ok_or_else(|| anyhow!("Campo 'blocks' não encontrado"))?
            .as_array()
            .ok_or_else(|| anyhow!("'blocks' não é um array"))?;

        // Itera por todos os blocks
        for block in blocks {
            if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                if block_type == "image" {
                    if let Some(data) = block.get("data") {
                        let imagem = ImagemChamado {
                            url: data
                                .get("file")
                                .and_then(|f| f.get("url"))
                                .and_then(|u| u.as_str())
                                .unwrap_or("")
                                .to_string(),
                            caption: data
                                .get("caption")
                                .and_then(|c| c.as_str())
                                .unwrap_or("")
                                .to_string(),
                            with_border: data
                                .get("withBorder")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(false),
                            stretched: data
                                .get("stretched")
                                .and_then(|s| s.as_bool())
                                .unwrap_or(false),
                            with_background: data
                                .get("withBackground")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(false),
                        };

                        // Só adiciona se tiver URL
                        if !imagem.url.is_empty() {
                            imagens.push(imagem);
                        }
                    }
                }
            }
        }

        Ok(imagens)
    }
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
