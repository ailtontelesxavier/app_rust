use anyhow::Ok;
use anyhow::Result;
use async_trait::async_trait;
use shared::Repository;
use sqlx::PgPool;

use crate::chamado::model::CategoriaChamado;
use crate::chamado::model::TipoChamado;
use crate::chamado::schema::CreateCategoriaChamadoSchema;
use crate::chamado::schema::CreateTipoChamadoSchema;
use crate::chamado::schema::UpdateCategoriaChamadoSchema;
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

        Ok(
            sqlx::query_as(&query)
            .bind(input.nome.to_string())
            .fetch_one(pool)
            .await?
        )
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
