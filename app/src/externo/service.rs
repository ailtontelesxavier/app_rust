use anyhow::Result;
use shared::{PaginatedResponse, Repository};
use sqlx::PgPool;
use uuid::Uuid;

use crate::externo::{
    LinhaRepository,
    model::{Contato, Linha},
    repository::ContatoRepository,
    schema::{CreateContato, CreateLinhaSchema, UpdateContato, UpdateLinhaSchema},
};

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

    pub async fn create(&self, pool: &PgPool, input: CreateContato) -> Result<Contato> {
        Ok(self.repo.create(pool, input).await?)
    }

    pub async fn update(&self, pool: &PgPool, id: Uuid, input: UpdateContato) -> Result<Contato> {
        Ok(self.repo.update(pool, id, input).await?)
    }

    pub async fn delete(&self, pool: &PgPool, id: Uuid) -> Result<()> {
        Ok(self.repo.delete(pool, id).await?)
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
}
