use sqlx::PgPool;

use crate::core::{
    model::{Municipio},
    schema::{CreateMunicipio, MunicipioWithUf, UpdateMunicipio},
};

pub struct MunicipioService {
    db: PgPool,
}

impl MunicipioService {
    pub fn new(db: PgPool) -> Self {
        MunicipioService { db }
    }

    pub async fn create(&self, municipio: CreateMunicipio) -> Result<Municipio, sqlx::Error> {
        let municipio = sqlx::query_as!(
            Municipio,
            r#"
            INSERT INTO municipio (nome, uf_id)
            VALUES ($1, $2)
            RETURNING id, nome, uf_id
            "#,
            municipio.nome,
            municipio.uf_id
        )
        .fetch_one(&self.db)
        .await?;

        Ok(municipio)
    }

    pub async fn find_all(&self) -> Result<Vec<Municipio>, sqlx::Error> {
        let municipios = sqlx::query_as!(Municipio, r#"SELECT id, nome, uf_id FROM municipio"#)
            .fetch_all(&self.db)
            .await?;

        Ok(municipios)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<Option<Municipio>, sqlx::Error> {
        let municipio = sqlx::query_as!(
            Municipio,
            r#"SELECT id, nome, uf_id FROM municipio WHERE id = $1"#,
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(municipio)
    }

    pub async fn update(
        &self,
        id: i64,
        municipio: UpdateMunicipio,
    ) -> Result<Option<Municipio>, sqlx::Error> {
        let municipio = sqlx::query_as!(
            Municipio,
            r#"
            UPDATE municipio
            SET nome = $1, uf_id = $2
            WHERE id = $3
            RETURNING id, nome, uf_id
        "#,
            municipio.nome,
            municipio.uf_id,
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(municipio)
    }

    pub async fn delete(&self, id: i64) -> Result<Option<Municipio>, sqlx::Error> {
        let municipio = sqlx::query_as!(
            Municipio,
            r#"
            DELETE FROM municipio
            WHERE id = $1
            RETURNING id, nome, uf_id
        "#,
            id
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(municipio)
    }

    pub async fn get_municipio_with_uf(
        pool: &PgPool,
        municipio_id: i64,
    ) -> Result<MunicipioWithUf, sqlx::Error> {
        let row = sqlx::query_as!(
            MunicipioWithUf,
            r#"
            SELECT m.id, m.nome, m.uf_id, u.sigla AS uf_sigla, u.nome AS uf_nome
            FROM municipio m
            INNER JOIN uf u ON m.uf_id = u.id
            WHERE m.id = $1
            "#,
            municipio_id
        )
        .fetch_one(pool)
        .await?;

        Ok(row)
    }
}
