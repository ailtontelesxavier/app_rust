use anyhow::Ok;
use anyhow::Result;
use async_trait::async_trait;
use serde_json::json;
use serde_json::Value;
use shared::Repository;
use sqlx::PgPool;

use crate::chamado::model::{CategoriaChamado, Chamado, ServicoChamado, TipoChamado};
use crate::chamado::schema::CreateCategoriaChamadoSchema;
use crate::chamado::schema::CreateChamado;
use crate::chamado::schema::CreateServicoChamadoSchema;
use crate::chamado::schema::CreateTipoChamadoSchema;
use crate::chamado::schema::UpdateCategoriaChamadoSchema;
use crate::chamado::schema::UpdateChamado;
use crate::chamado::schema::UpdateServicoChamadoSchema;
use crate::chamado::schema::UpdateTipoChamadoSchema;

pub struct TipoChamadoRepository;

#[async_trait]
impl Repository<TipoChamado, i64> for TipoChamadoRepository {
    type CreateInput = CreateTipoChamadoSchema;
    type UpdateInput = UpdateTipoChamadoSchema;

    fn table_name(&self) -> &str {
        "chamado_tipos_chamado"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["m.nome"]
    }

    fn select_clause(&self) -> &str {
        "m.id, m.nome"
    }

    fn from_clause(&self) -> &str {
        "chamado_tipos_chamado m"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<TipoChamado> {
        Ok(sqlx::query_as!(
            TipoChamado,
            "INSERT INTO chamado_tipos_chamado (nome) VALUES ($1) RETURNING *",
            input.nome.to_string()
        )
        .fetch_one(pool)
        .await?)
    }

    async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        input: Self::UpdateInput,
    ) -> Result<TipoChamado> {
        Ok(sqlx::query_as!(
            TipoChamado,
            r#"UPDATE chamado_tipos_chamado SET nome = $1 WHERE id = $2 RETURNING *"#,
            input.nome,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM chamado_tipos_chamado WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct CategoriaChamadoRepository;

#[async_trait]
impl Repository<CategoriaChamado, i64> for CategoriaChamadoRepository {
    type CreateInput = CreateCategoriaChamadoSchema;
    type UpdateInput = UpdateCategoriaChamadoSchema;

    fn table_name(&self) -> &str {
        "chamado_categoria_chamado"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["m.nome"]
    }

    fn select_clause(&self) -> &str {
        "m.id, m.nome"
    }

    fn from_clause(&self) -> &str {
        "chamado_categoria_chamado m"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<CategoriaChamado> {
        let query = format!(
            "INSERT INTO {} (nome) VALUES ($1) RETURNING *",
            self.table_name()
        );

        Ok(sqlx::query_as(&query)
            .bind(input.nome.to_string())
            .fetch_one(pool)
            .await?)
    }

    async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        input: Self::UpdateInput,
    ) -> Result<CategoriaChamado> {
        Ok(sqlx::query_as!(
            CategoriaChamado,
            r#"UPDATE chamado_categoria_chamado SET nome = $1 WHERE id = $2 RETURNING *"#,
            input.nome,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM chamado_categoria_chamado WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct ServicoChamadoRepository;

#[async_trait]
impl Repository<ServicoChamado, i64> for ServicoChamadoRepository {
    type CreateInput = CreateServicoChamadoSchema;
    type UpdateInput = UpdateServicoChamadoSchema;

    fn table_name(&self) -> &str {
        "chamado_servico_chamado"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["m.nome"]
    }

    fn select_clause(&self) -> &str {
        "m.id, m.nome, m.tipo_id"
    }

    fn from_clause(&self) -> &str {
        "chamado_servico_chamado m"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<ServicoChamado> {
        let query = format!(
            "INSERT INTO {} (nome, tipo_id) VALUES ($1, $2) RETURNING *",
            self.table_name()
        );

        Ok(sqlx::query_as(&query)
            .bind(input.nome.to_string())
            .bind(input.tipo_id)
            .fetch_one(pool)
            .await?)
    }

    async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        input: Self::UpdateInput,
    ) -> Result<ServicoChamado> {
        Ok(sqlx::query_as!(
            ServicoChamado,
            r#"UPDATE chamado_servico_chamado SET nome = $1, tipo_id = $2  WHERE id = $3 RETURNING *"#,
            input.nome,
            input.tipo_id,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM chamado_servico_chamado WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct ChamadoRepository;

#[async_trait]
impl Repository<Chamado, i64> for ChamadoRepository {
    type CreateInput = CreateChamado;
    type UpdateInput = UpdateChamado;

    fn table_name(&self) -> &str {
        "chamado_chamados"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["m.titulo", "m.descricao"]
    }

    fn select_clause(&self) -> &str {
        "m.id, m.titulo, m.descricao, m.status, m.created_at, m.updated_at, m.user_solic_id, m.servico_id, m.tipo_id"
    }

    fn from_clause(&self) -> &str {
        "chamado_chamados m"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<Chamado> {
        // Conteúdo vazio inicial
        let editor_data: Value = json!({
            "time": 0,
            "blocks": [],
            "version": "2.31.0-rc.7"
        });
        let query = format!(
            "INSERT INTO {} 
            (
            titulo,
            descricao,
            status,
            user_solic_id,
            servico_id,
            tipo_id,
            created_at,
            updated_at
            ) VALUES (
            $1, $2, $3, $4, $5, $6, NOW(), NOW()) RETURNING *",
            self.table_name()
        );

        Ok(sqlx::query_as(&query)
            .bind(input.titulo.to_string())
            .bind(editor_data)
            .bind(input.status)
            .bind(input.user_solic_id)
            .bind(input.servico_id)
            .bind(input.tipo_id)
            .fetch_one(pool)
            .await?)
    }

    async fn update(&self, pool: &PgPool, id: i64, input: Self::UpdateInput) -> Result<Chamado> {
        Ok(sqlx::query_as!(
            Chamado,
            r#"UPDATE chamado_chamados SET titulo = $1, descricao = $2, servico_id = $3, tipo_id = $4, updated_at = NOW() WHERE id = $5 RETURNING *"#,
            input.titulo,
            input.descricao,
            input.servico_id,
            input.tipo_id,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM chamado_chamados WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}
