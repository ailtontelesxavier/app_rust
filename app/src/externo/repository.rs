use async_trait::async_trait;
use minijinja::Value;
use shared::Repository;
use sqlx::PgPool;

use anyhow::Ok;
use anyhow::Result;
use uuid::Uuid;

use crate::externo::model::Contato;
use crate::externo::model::Regiao;
use crate::externo::schema::CreateContato;
use crate::externo::schema::CreateRegiaoSchema;
use crate::externo::schema::UpdateContato;
use crate::externo::schema::UpdateRegiaoSchema;
use crate::externo::{
    model::Linha,
    schema::{CreateLinhaSchema, UpdateLinhaSchema},
};

pub struct LinhaRepository;

#[async_trait]
impl Repository<Linha, i32> for LinhaRepository {
    type CreateInput = CreateLinhaSchema;
    type UpdateInput = UpdateLinhaSchema;

    fn table_name(&self) -> &str {
        "linha"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["m.nome"]
    }

    fn select_clause(&self) -> &str {
        "m.id, m.nome, m.permite_cnpj, m.permite_cpf, m.permite_avalista, m.valor_maximo"
    }

    fn from_clause(&self) -> &str {
        "linha m"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<Linha> {
        Ok(sqlx::query_as!(
            Linha,
            "INSERT INTO linha (nome, permite_cnpj, permite_cpf, permite_avalista, valor_maximo)
            VALUES ($1, $2, $3, $4, $5) RETURNING *",
            input.nome.to_string(),
            input.permite_cnpj,
            input.permite_cpf,
            input.permite_avalista,
            input.valor_maximo
        )
        .fetch_one(pool)
        .await?)
    }

    async fn update(&self, pool: &PgPool, id: i32, input: Self::UpdateInput) -> Result<Linha> {
        Ok(sqlx::query_as!(
            Linha,
            r#"
            UPDATE linha
            SET
                nome = COALESCE($1, nome),
                permite_cnpj = COALESCE($2, permite_cnpj),
                permite_cpf = COALESCE($3, permite_cpf),
                permite_avalista = COALESCE($4, permite_avalista),
                valor_maximo = COALESCE($5, valor_maximo)
            WHERE id = $6
            RETURNING *"#,
            input.nome,
            input.permite_cnpj,
            input.permite_cpf,
            input.permite_avalista,
            input.valor_maximo,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM linha WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct ContatoRepository;

#[async_trait]
impl Repository<Contato, Uuid> for ContatoRepository {
    type CreateInput = CreateContato;
    type UpdateInput = UpdateContato;

    fn table_name(&self) -> &str {
        "contato"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["m.nome"]
    }

    fn select_clause(&self) -> &str {
        "m.id, m.linha_id, m.protocolo, m.status_atendimento, m.cpf_cnpj, m.nome, m.telefone, m.email,
            m.cidade_id, m.val_solicitado, m.status_tramitacao, m.campos, m.dados_imports, m.created_at, m.updated_at"
    }

    fn from_clause(&self) -> &str {
        "contato m"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<Contato> {
        Ok(sqlx::query_as!(
            Contato,
            "INSERT INTO contato (linha_id, protocolo, status_atendimento, cpf_cnpj, nome, telefone, email,
            cidade_id, val_solicitado, status_tramitacao, campos, dados_imports, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, NOW(), NOW()) RETURNING *",
            input.linha_id,
            input.protocolo,
            input.status_atendimento,
            input.cpf_cnpj,
            input.nome,
            input.telefone,
            input.email,
            input.cidade_id,
            input.val_solicitado,
            input.status_tramitacao,
            input.campos,
            input.dados_imports
        )
        .fetch_one(pool)
        .await?)
    }

    async fn update(&self, pool: &PgPool, id: Uuid, input: Self::UpdateInput) -> Result<Contato> {
        Ok(sqlx::query_as!(
            Contato,
            r#"
            UPDATE contato
            SET
                linha_id = COALESCE($1, linha_id),
                status_atendimento = COALESCE($2, status_atendimento),
                cpf_cnpj = COALESCE($3, cpf_cnpj),
                nome = COALESCE($4, nome),
                telefone = COALESCE($5, telefone),
                email = COALESCE($6, email),
                cidade_id = COALESCE($7, cidade_id),
                val_solicitado = COALESCE($8, val_solicitado),
                status_tramitacao = COALESCE($9, status_tramitacao),
                campos = COALESCE($10, campos),
                dados_imports = COALESCE($11, dados_imports),
                updated_at = NOW()
            WHERE id = $12
            RETURNING *"#,
            input.linha_id,
            input.status_atendimento,
            input.cpf_cnpj,
            input.nome,
            input.telefone,
            input.email,
            input.cidade_id,
            input.val_solicitado,
            input.status_tramitacao,
            input.campos,
            input.dados_imports,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: Uuid) -> Result<()> {
        sqlx::query!("DELETE FROM contato WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct RegiaoRepository;

#[async_trait]
impl Repository<Regiao, i32> for RegiaoRepository {
    type CreateInput = CreateRegiaoSchema;
    type UpdateInput = UpdateRegiaoSchema;

    fn table_name(&self) -> &str {
        "emprestimo_regiao"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["r.name"]
    }

    fn select_clause(&self) -> &str {
        "r.id, r.name, r.municipio_id, m.nome as municipio_nome"
    }

    fn from_clause(&self) -> &str {
        "emprestimo_regiao r
        INNER JOIN municipio m ON r.municipio_id = m.id
        "
    }

    fn id_column(&self) -> &str {
        "r.id"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<Regiao> {
        Ok(sqlx::query_as!(
            Regiao,
            r#"
            INSERT INTO emprestimo_regiao (name, municipio_id)
            VALUES ($1, $2)
            RETURNING id, name, municipio_id, NULL as "municipio_nome?"
            "#,
            input.name,
            input.municipio_id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn update(&self, pool: &PgPool, id: i32, input: Self::UpdateInput) -> Result<Regiao> {
        Ok(sqlx::query_as!(
            Regiao,
            r#"
            UPDATE emprestimo_regiao
            SET
                name = COALESCE($1, name),
                municipio_id = COALESCE($2, municipio_id)
            WHERE id = $3
            RETURNING id, name, municipio_id, NULL as "municipio_nome?"
            "#,
            input.name,
            input.municipio_id,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM emprestimo_regiao WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
