
use anyhow::Ok;
use anyhow::Result;
use async_trait::async_trait;
use shared::Repository;
use sqlx::PgPool;

use crate::chamado::model::TipoChamado;
use crate::chamado::schema::CreateTipoChamadoSchema;
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

    async fn update(&self, pool: &PgPool, id: i64, input: Self::UpdateInput) -> Result<TipoChamado> {
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


