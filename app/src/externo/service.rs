use anyhow::{Ok, Result};
use std::fs;

use chrono::Datelike;
use chrono::{DateTime, Local, Utc};
use shared::{PaginatedResponse, Repository};
use sqlx::{PgPool, Transaction};
use uuid::Uuid;

use crate::externo::schema::{AplicacaoRecursos, CreateContatoSchema, PronafB, TipoContatoExtra};
use crate::externo::{
    LinhaRepository,
    model::{Contato, Linha},
    repository::ContatoRepository,
    schema::{CreateContato, CreateLinhaSchema, UpdateContato, UpdateLinhaSchema},
};
use crate::externo::{StatusDocumentoEnum, StatusTramitacaoEnum};

pub struct LinhaService {
    repo: LinhaRepository,
}

impl LinhaService {
    pub fn new() -> Self {
        Self {
            repo: LinhaRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i32) -> Result<Linha> {
        Ok(self.repo.get_by_id(pool, id).await?)
    }

    pub async fn create(&self, pool: &PgPool, input: CreateLinhaSchema) -> Result<Linha> {
        Ok(self.repo.create(pool, input).await?)
    }

    pub async fn update(&self, pool: &PgPool, id: i32, input: UpdateLinhaSchema) -> Result<Linha> {
        Ok(self.repo.update(pool, id, input).await?)
    }

    pub async fn delete(&self, pool: &PgPool, id: i32) -> Result<()> {
        Ok(self.repo.delete(pool, id).await?)
    }

    pub async fn get_paginated(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<Linha>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }
}

/*
==========================================

----------------- Contato ---------------
==========================================

*/
pub struct ContatoService {
    repo: ContatoRepository,
}

impl ContatoService {
    pub fn new() -> Self {
        Self {
            repo: ContatoRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: Uuid) -> Result<Contato> {
        Ok(self.repo.get_by_id(pool, id).await?)
    }

    /*
       pool: conexao do banco de dados
       data_contato: CreateContatoSchema,
       form_data: PronafB,
       list_item_recurso: Vec<AplicacaoRecursos>,
       arquivos: Vec<(String, (String, Vec<u8>))>, // (id tipo do arquivo(categria do arquivo), (nome do arquivo, dados))
    */
    pub async fn create(
        &self,
        pool: &PgPool,
        tipo: TipoContatoExtra,
        linha: Linha,
        data_contato: CreateContatoSchema,
        list_item_recurso: Option<Vec<AplicacaoRecursos>>,
        arquivos: Option<Vec<(String, (String, Vec<u8>))>>,
    ) -> Result<Contato> {
        let mut tx: Transaction<'_, sqlx::Postgres> = pool.begin().await?;

        let result = async {

            let campos = match &tipo {
                TipoContatoExtra::PronafB(p) => serde_json::to_value(p).unwrap(),
                TipoContatoExtra::MicroCreditoOnline(m) => serde_json::to_value(m).unwrap(),
                TipoContatoExtra::CreditoPopular(c) => serde_json::to_value(c).unwrap(),
                TipoContatoExtra::MaosQueCriam(m) => serde_json::to_value(m).unwrap(),
                TipoContatoExtra::CreditoOnline(c) => serde_json::to_value(c).unwrap()
            };

            //gerar protocolo
            let mut protocolo = Contato::gerar_codigo_protocolo();
            loop {
                if !Self::exists_by_protocolo(&self, pool, &protocolo).await? {
                    break;
                } else {
                    protocolo = Contato::gerar_codigo_protocolo();
                }
            }

            //inserir contato
            let contato = sqlx::query_as!(
                Contato,
                "INSERT INTO contato (linha_id, protocolo, status_atendimento, cpf_cnpj, nome, telefone, email,
                cidade_id, val_solicitado, status_tramitacao, campos, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, NOW(), NOW()) RETURNING *",
                linha.id,
                protocolo,
                false,
                data_contato.cpf_cnpj,
                data_contato.nome,
                data_contato.telefone,
                data_contato.email,
                data_contato.cidade_id,
                data_contato.val_solicitado,
                StatusTramitacaoEnum::AguardandoAtendimento.to_i32(),
                campos
            )
            .fetch_one(pool)
            .await?;

            //inserir recursos
            for recurso in &list_item_recurso.unwrap_or_default() {
                sqlx::query!(
                    r#"
                    INSERT INTO aplicacao_recurso(
	                descricao, quantidade, valor_unitario, valor_total, contato_id)
	                    VALUES ($1, $2, $3, $4, $5);
                    "#,
                    recurso.descricao.clone(),
                    recurso.quantidade,
                    recurso.valor_unitario.clone(),
                    recurso.valor_total.clone(),
                    contato.id
                )
                .execute(&mut *tx)
                .await?;
            }

            //inserir arquivos
            for (tipo_arquivo, (nome_arquivo, dados_arquivo)) in arquivos.unwrap_or_default() {
                // upload do arquivo
                let end_arquivo = Self::upload_arquivo(contato.id, nome_arquivo, dados_arquivo).await;
                sqlx::query!(
                    r#"
                    INSERT INTO doc_solicitante(
	                contato_id, arquivo, status_arquivo, observacao, tipo)
	                VALUES ($1, $2, $3, $4, $5);
                    "#,
                    contato.id,
                    end_arquivo,
                    StatusDocumentoEnum::Pendente.to_i32(),
                    "",
                    tipo_arquivo
                )
                .execute(&mut *tx)
                .await?;
            }
            Ok(contato)

        }.await;

        match result {
            std::result::Result::Ok(result) => {
                tx.commit().await?;
                Ok(result)
            }
            Err(e) => {
                tx.rollback().await?;
                return Err(e);
            }
        }
    }

    pub async fn update(&self, pool: &PgPool, id: Uuid, input: UpdateContato) -> Result<Contato> {
        Ok(self.repo.update(pool, id, input).await?)
    }

    pub async fn delete(&self, pool: &PgPool, id: Uuid) -> Result<()> {
        Ok(self.repo.delete(pool, id).await?)
    }

    pub async fn exists_by_protocolo(
        &self,
        pool: &PgPool,
        protocolo: &str,
    ) -> anyhow::Result<bool> {
        let count: i64 = sqlx::query_scalar!(
            r#"
        SELECT COUNT(*)
        FROM contato
        WHERE protocolo = $1
        "#,
            protocolo
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        Ok(count > 0)
    }

    pub async fn get_paginated(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<Contato>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }

    /*
    (nome, arquivo bytes)
     */
    pub async fn upload_arquivo(contato_id: Uuid, nome_arq: String, arq: Vec<u8>) -> String {
        let mut file_url = String::new();

        let name = nome_arq;
        let data = arq;

        let agora = Local::now();
        let ano = agora.year();
        let mes = agora.month();

        // Inclui o ID do contato no path para organização
        let path_file = format!("uploads/contato/{}/{}/{}", ano, mes, contato_id.to_string());

        // Cria o diretório se não existir
        if let Err(e) = fs::create_dir_all(&path_file) {
            eprintln!("Erro ao criar diretório {}: {}", path_file, e);
        }

        // Obtém o nome do arquivo e extensão
        let ext = name.rsplit('.').next().unwrap_or("png").to_lowercase();

        // Gera nome único para o arquivo
        let filename = format!("{}/{}.{}", path_file, Uuid::new_v4(), ext);

        // Salva o arquivo
        match fs::write(&filename, &data) {
            std::result::Result::Ok(_) => {
                file_url = format!("/{}", filename);
            }
            Err(e) => {
                eprintln!("Erro ao salvar arquivo {}: {}", filename, e);
            }
        }

        file_url
    }
}
