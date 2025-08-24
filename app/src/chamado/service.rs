use shared::{PaginatedResponse, Repository};
use sqlx::PgPool;
use anyhow::Result;

use crate::chamado::{model::TipoChamado, repository::TipoChamadoRepository, schema::{CreateTipoChamadoSchema, UpdateTipoChamadoSchema}};

pub struct TipoChamadoService {
    repo: TipoChamadoRepository,
}

impl TipoChamadoService {
    pub fn new() -> Self {
        Self {
            repo: TipoChamadoRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i64) -> Result<TipoChamado> {
        Repository::<TipoChamado, i64>::get_by_id(&self.repo, pool, id).await
    }

    pub async fn create(&self, pool: &PgPool, input: CreateTipoChamadoSchema) -> Result<TipoChamado> {
        self.repo.create(pool, input).await
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        input: UpdateTipoChamadoSchema,
    ) -> Result<TipoChamado> {
        self.repo.update(pool, id, input).await
    }

    pub async fn delete(&self, pool: &PgPool, id: i64) -> Result<()> {
        self.repo.delete(pool, id).await
    }

    pub async fn get_paginated(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<TipoChamado>> {
        Repository::<TipoChamado, i64>::get_paginated(&self.repo, pool, find, page, page_size).await
    }
}

