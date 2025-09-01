use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, from_str};
use sqlx::FromRow;

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
