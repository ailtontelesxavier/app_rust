use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{FromRow, PgPool, QueryBuilder, postgres::PgRow};
use std::sync::Arc;

use crate::{model::module::{Module, Permission}, schema::{CreateModuleSchema, PermissionCreateSchema, PermissionUpdateSchema, UpdateModuleSchema}};

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total_records: i64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
}

#[async_trait]
pub trait Repository<T>
where
    T: for<'r> FromRow<'r, PgRow> + Send + Unpin + Serialize + 'static,
{
    type CreateInput: DeserializeOwned + Send + Sync;
    type UpdateInput: DeserializeOwned + Send + Sync;

    fn table_name(&self) -> &str;
    fn searchable_fields(&self) -> &[&str];
    fn select_clause(&self) -> &str;
    fn from_clause(&self) -> &str;

    fn extra_where(&self) -> Option<&str> {
        None
    }

    async fn get_paginated(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: u32,
        page_size: u32,
    ) -> Result<PaginatedResponse<T>, sqlx::Error> {
        let page = page.max(1);
        let offset = (page - 1) * page_size;
        let limit = page_size.min(100);

        // === WHERE clause builder ===
        let mut where_parts = vec![];
        let mut bindings = vec![];

        if let Some(find) = find {
            let like = format!("%{}%", find);
            let mut filter_parts = vec![];

            for field in self.searchable_fields() {
                filter_parts.push(format!(r#"{field} ILIKE ?"#));
                bindings.push(like.clone());
            }

            if !filter_parts.is_empty() {
                where_parts.push(format!("({})", filter_parts.join(" OR ")));
            }
        }

        if let Some(extra) = self.extra_where() {
            where_parts.push(format!("({})", extra));
        }

        let where_clause = if !where_parts.is_empty() {
            format!("WHERE {}", where_parts.join(" AND "))
        } else {
            "".to_string()
        };

        // === COUNT ===
        let count_sql = format!(
            "SELECT COUNT(*) FROM {} {}",
            self.from_clause(),
            where_clause
        );

        let mut count_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(&count_sql);
        for val in &bindings {
            count_builder.push_bind(val);
        }

        let total: (i64,) = count_builder.build_query_as().fetch_one(pool).await?;

        // === SELECT data ===
        let data_sql = format!(
            "SELECT {} FROM {} {} ORDER BY 1 DESC OFFSET {} LIMIT {}",
            self.select_clause(),
            self.from_clause(),
            where_clause,
            offset,
            limit
        );

        let mut data_builder: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(&data_sql);
        for val in &bindings {
            data_builder.push_bind(val);
        }

        let data = data_builder.build_query_as::<T>().fetch_all(pool).await?;

        let total_pages = if total.0 == 0 {
            1
        } else {
            ((total.0 as f32) / (limit as f32)).ceil() as u32
        };

        Ok(PaginatedResponse {
            data,
            total_records: total.0,
            page,
            page_size: limit,
            total_pages,
        })
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<T, sqlx::Error>;

    async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: Self::UpdateInput,
    ) -> Result<T, sqlx::Error>;

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<(), sqlx::Error>;
}

//#[derive(Debug, Serialize, FromRow)]
//pub struct Module {
//    pub id: i32,
//   pub title: String,
//}


pub struct ModuleRepository;

#[async_trait]
impl Repository<Module> for ModuleRepository {
    type CreateInput = CreateModuleSchema;
    type UpdateInput = UpdateModuleSchema;

    fn table_name(&self) -> &str {
        "module"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["m.title"]
    }

    fn select_clause(&self) -> &str {
        "m.id, m.title, m.created_at, m.updated_at"
    }

    fn from_clause(&self) -> &str {
        "module m"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<Module, sqlx::Error> {
        sqlx::query_as!(
            Module,
            r#"INSERT INTO module (title) VALUES ($1) RETURNING id, title, created_at, updated_at"#,
            input.title
        )
        .fetch_one(pool)
        .await
    }

    async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: Self::UpdateInput,
    ) -> Result<Module, sqlx::Error> {
        sqlx::query_as!(
            Module,
            r#"UPDATE module SET title = $1 WHERE id = $2 RETURNING id, title, created_at, updated_at"#,
            input.title,
            id
        )
        .fetch_one(pool)
        .await
    }

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM module WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}


pub struct PermissionRepository;

#[async_trait]
impl Repository<Permission> for PermissionRepository {
    type CreateInput = PermissionCreateSchema;
    type UpdateInput = PermissionUpdateSchema;

    fn table_name(&self) -> &str {
        "permission"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["p.name", "p.description"]
    }

    fn select_clause(&self) -> &str {
        "p.id, p.name, p.description, p.module_id, p.created_at, p.updated_at"
    }

    fn from_clause(&self) -> &str {
        "permission p"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<Permission, sqlx::Error> {
        sqlx::query_as!(
            Permission,
            r#"INSERT INTO permission (name, description, module_id) 
               VALUES ($1, $2, $3) 
               RETURNING id, name, description, module_id, created_at, updated_at"#,
            input.name,
            input.description,
            input.module_id
        )
        .fetch_one(pool)
        .await
    }

    async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: Self::UpdateInput,
    ) -> Result<Permission, sqlx::Error> {
        sqlx::query_as!(
            Permission,
            r#"UPDATE permission 
               SET name = $1, description = $2, module_id = $3 
               WHERE id = $4 
               RETURNING id, name, description, module_id, created_at, updated_at"#,
            input.name,
            input.description,
            input.module_id,
            id
        )
        .fetch_one(pool)
        .await
    }

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM permission WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}