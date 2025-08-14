use crate::{
    model::module::{Perfil, Permission, PermissionWithModule},
    repository::{self, Repository},
    schema::{PerfilCreateSchema, PerfilUpdateSchema, PermissionCreateSchema, PermissionUpdateSchema},
};
use anyhow::Result;
use sqlx::PgPool;

use axum::{extract::State, response::Html};
use shared::SharedState;

use crate::{
    model::module::Module,
    schema::{CreateModuleSchema, UpdateModuleSchema},
};

pub struct ModuleService {
    repo: repository::ModuleRepository,
}

impl ModuleService {
    pub fn new() -> Self {
        Self {
            repo: repository::ModuleRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i32) -> Result<Module> {
        Ok(self.repo.get_by_id(pool, id).await?)
    }

    pub async fn create(&self, pool: &PgPool, input: CreateModuleSchema) -> Result<Module> {
        Ok(self.repo.create(pool, input).await?)
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: UpdateModuleSchema,
    ) -> Result<Module> {
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
    ) -> Result<repository::PaginatedResponse<Module>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }
}

pub struct PermissionService {
    repo: repository::PermissionRepository,
}

impl PermissionService {
    pub fn new() -> Self {
        Self {
            repo: repository::PermissionRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i32) -> Result<Permission> {
        Ok(self.repo.get_by_id(pool, id).await?)
    }

    pub async fn create(&self, pool: &PgPool, input: PermissionCreateSchema) -> Result<Permission> {
        Ok(self.repo.create(pool, input).await?)
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: PermissionUpdateSchema,
    ) -> Result<Permission> {
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
    ) -> Result<repository::PaginatedResponse<Permission>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }
}

pub struct PerfilService {
    repo: repository::PerfilRepository,
}

impl PerfilService {
    pub fn new() -> Self {
        Self {
            repo: repository::PerfilRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i32) -> Result<Perfil> {
        Ok(self.repo.get_by_id(pool, id).await?)
    }

    pub async fn create(&self, pool: &PgPool, input: PerfilCreateSchema) -> Result<Perfil> {
        Ok(self.repo.create(pool, input).await?)
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: PerfilUpdateSchema,
    ) -> Result<Perfil> {
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
    ) -> Result<repository::PaginatedResponse<Perfil>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }
}

pub async fn home(State(state): State<SharedState>) -> Html<String> {
    let template = state.templates.get_template("index.html").unwrap();
    let html = template.render(()).unwrap();
    Html(html)
}

pub async fn saudacao(State(state): State<SharedState>) -> Html<String> {
    let context = [("nome", "João")];
    let template = state.templates.get_template("saudacao.html").unwrap();
    let html = template.render(context).unwrap();
    Html(html)
}
