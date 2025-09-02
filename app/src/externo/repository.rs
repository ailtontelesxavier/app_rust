use async_trait::async_trait;
use shared::Repository;
use sqlx::PgPool;

use anyhow::Ok;
use anyhow::Result;

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
