use anyhow::Result;
use shared::PaginatedResponse;
use sqlx::{PgPool, Pool};

use crate::core::{
    model::{Municipio, Uf},
    schema::{CreateMunicipio, MunicipioWithUf, UpdateMunicipio},
};

pub struct UfService {}

impl UfService {
    pub fn new() -> Self {
        UfService {}
    }

    /*
       retorna lista de Ufs
    */
    pub async fn get_paginated(
        &self,
        db: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<Uf>> {
        let page = page.max(1) as i32;
        let page_size = page_size.min(100) as i32;
        let offset = (page - 1) * page_size;

        let like_term = match find {
            Some(search_term) => format!("%{search_term}%"),
            None => "%".to_string(),
        };

        // Busca total de registros para paginação
        let total: (i64,) = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(p) FROM uf p
            WHERE p.sigla ILIKE $1 and p.nome ILIKE $1
            "#,
        )
        .bind(&like_term)
        .fetch_one(db)
        .await?;

        let total_records = total.0;
        let total_pages = if total_records == 0 {
            1
        } else {
            ((total_records as f64) / (page_size as f64)).ceil() as i32
        };

        // Busca os registros paginados
        let items: Vec<Uf> = sqlx::query_as!(
            Uf,
            r#"
            SELECT 
                *
            FROM uf p
            WHERE p.sigla ILIKE $1 and p.nome ILIKE $1
            ORDER BY p.id DESC
            LIMIT $2 OFFSET $3
            "#,
            like_term,
            page_size as i64,
            offset as i64
        )
        .fetch_all(db)
        .await?;

        Ok(PaginatedResponse {
            data: items,
            total_records,
            page,
            page_size,
            total_pages,
        })
    }
}

pub struct MunicipioService {}

impl MunicipioService {
    pub fn new() -> Self {
        MunicipioService {}
    }

    pub async fn create(
        &self,
        db: &PgPool,
        municipio: CreateMunicipio,
    ) -> Result<Municipio, sqlx::Error> {
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
        .fetch_one(db)
        .await?;

        Ok(municipio)
    }

    pub async fn find_all(&self, db: &PgPool) -> Result<Vec<Municipio>, sqlx::Error> {
        let municipios = sqlx::query_as!(Municipio, r#"SELECT id, nome, uf_id FROM municipio"#)
            .fetch_all(db)
            .await?;

        Ok(municipios)
    }

    pub async fn find_by_id(&self, db: &PgPool, id: i64) -> Result<Option<Municipio>, sqlx::Error> {
        let municipio = sqlx::query_as!(
            Municipio,
            r#"SELECT id, nome, uf_id FROM municipio WHERE id = $1"#,
            id
        )
        .fetch_optional(db)
        .await?;

        Ok(municipio)
    }

    pub async fn find_municipio_with_uf_by_id(
        &self,
        db: &PgPool,
        ibge_id: i64,
    ) -> Result<Option<MunicipioWithUf>, sqlx::Error> {
        let municipio = sqlx::query_as!(
            MunicipioWithUf,
            r#"
            SELECT m.id, m.nome, m.uf_id, u.sigla as uf_sigla, u.nome as uf_nome
            FROM municipio m
            JOIN uf u ON u.id = m.uf_id
            WHERE m.id = $1"#,
            ibge_id
        )
        .fetch_optional(db)
        .await?;

        Ok(municipio)
    }

    /*
       retorna lista de Municipios
    */
    pub async fn get_paginated(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<MunicipioWithUf>> {
        let page = page.max(1) as i32;
        let page_size = page_size.min(100) as i32;
        let offset = (page - 1) * page_size;

        let like_term = match find {
            Some(search_term) => format!("%{search_term}%"),
            None => "%".to_string(),
        };

        // Busca total de registros para paginação
        let total: (i64,) = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(p) FROM municipio p
            WHERE p.nome ILIKE $1
            "#,
        )
        .bind(&like_term)
        .fetch_one(pool)
        .await?;

        let total_records = total.0;
        let total_pages = if total_records == 0 {
            1
        } else {
            ((total_records as f64) / (page_size as f64)).ceil() as i32
        };

        // Busca os registros paginados
        let items: Vec<MunicipioWithUf> = sqlx::query_as!(
            MunicipioWithUf,
            r#"
            SELECT m.id, m.nome, m.uf_id, u.sigla AS uf_sigla, u.nome AS uf_nome
            FROM municipio m
            INNER JOIN uf u ON m.uf_id = u.id
            WHERE m.nome ILIKE $1
            ORDER BY m.nome, u.sigla DESC
            LIMIT $2 OFFSET $3
            "#,
            like_term,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse {
            data: items,
            total_records,
            page,
            page_size,
            total_pages,
        })
    }

    pub async fn update(
        &self,
        db: &PgPool,
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
        .fetch_optional(db)
        .await?;

        Ok(municipio)
    }

    pub async fn delete(&self, db: &PgPool, id: i64) -> Result<Option<Municipio>, sqlx::Error> {
        let municipio = sqlx::query_as!(
            Municipio,
            r#"
            DELETE FROM municipio
            WHERE id = $1
            RETURNING id, nome, uf_id
        "#,
            id
        )
        .fetch_optional(db)
        .await?;

        Ok(municipio)
    }

    pub async fn get_municipio_with_uf(
        db: &PgPool,
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
        .fetch_one(db)
        .await?;

        Ok(row)
    }

    /*
       retorna lista de Municipios
    */
    pub async fn get_paginated_cidades_to(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<MunicipioWithUf>> {
        let page = page.max(1) as i32;
        let page_size = page_size.min(100) as i32;
        let offset = (page - 1) * page_size;

        let like_term = match find {
            Some(search_term) => format!("%{search_term}%"),
            None => "%".to_string(),
        };

        // Busca total de registros para paginação
        let total: (i64,) = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COUNT(m) FROM municipio m
            INNER JOIN uf u ON m.uf_id = u.id
            WHERE u.id = 17 AND m.nome ILIKE $1
            "#,
        )
        .bind(&like_term)
        .fetch_one(pool)
        .await?;

        let total_records = total.0;
        let total_pages = if total_records == 0 {
            1
        } else {
            ((total_records as f64) / (page_size as f64)).ceil() as i32
        };

        // Busca os registros paginados
        let items: Vec<MunicipioWithUf> = sqlx::query_as!(
            MunicipioWithUf,
            r#"
            SELECT m.id, m.nome, m.uf_id, u.sigla AS uf_sigla, u.nome AS uf_nome
            FROM municipio m
            INNER JOIN uf u ON m.uf_id = u.id
            WHERE m.nome ILIKE $1 and u.id = 17
            ORDER BY m.nome, u.sigla DESC
            LIMIT $2 OFFSET $3
            "#,
            like_term,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        Ok(PaginatedResponse {
            data: items,
            total_records,
            page,
            page_size,
            total_pages,
        })
    }
}
