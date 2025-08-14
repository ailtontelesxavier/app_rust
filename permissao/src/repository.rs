use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{Encode, FromRow, PgPool, Postgres, QueryBuilder, Type, postgres::PgRow};
use std::fmt::Display;
use tracing::{debug, info};

use crate::{
    model::module::{Module, Perfil, Permission, PermissionWithModule},
    schema::{
        CreateModuleSchema, PerfilCreateSchema, PerfilUpdateSchema, PermissionCreateSchema, PermissionUpdateSchema, UpdateModuleSchema
    },
};

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total_records: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[async_trait]
pub trait Repository<T, ID>
where
    T: for<'r> FromRow<'r, PgRow> + Send + Unpin + Serialize + 'static,
    ID: Type<Postgres> + for<'q> Encode<'q, Postgres> + Send + Sync + Display + 'static,
{
    /*
    use uuid::Uuid;
    impl Repository<User, Uuid> for UserRepository
    async fn get_by_id(&self, pool: &PgPool, id: Uuid) -> Result<User, sqlx::Error>

    impl Repository<Module, i32> for ModuleRepository
    async fn get_by_id(&self, pool: &PgPool, id: i32) -> Result<Module, sqlx::Error>

     */
    type CreateInput: DeserializeOwned + Send + Sync;
    type UpdateInput: DeserializeOwned + Send + Sync;

    fn table_name(&self) -> &str;
    fn searchable_fields(&self) -> &[&str];
    fn select_clause(&self) -> &str;
    fn from_clause(&self) -> &str;
    fn id_column(&self) -> &str {
        "id"
    }

    fn extra_where(&self) -> Option<&str> {
        None
    }

    async fn get_paginated(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<T>, sqlx::Error>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let page = page.max(1);
        let page_size = page_size.min(100);
        let offset = (page - 1) * page_size;

        // WHERE builder
        let mut where_parts = Vec::new();

        if let Some(term) = find {
            let like_term = format!("%{}%", term);
            let search_fields = self.searchable_fields();

            if !search_fields.is_empty() {
                let mut field_parts = Vec::new();
                for field in search_fields {
                    field_parts.push(format!("{} ILIKE '{}'", field, like_term));
                }
                where_parts.push(format!("({})", field_parts.join(" OR ")));
            }
        }

        if let Some(extra) = self.extra_where() {
            where_parts.push(extra.to_string());
        }

        let where_clause = if where_parts.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_parts.join(" AND "))
        };

        // === COUNT ===
        let mut count_builder = QueryBuilder::new(format!(
            "SELECT COUNT(*) FROM {} {}",
            self.from_clause(),
            where_clause
        ));

        let total: (i64,) = count_builder.build_query_as().fetch_one(pool).await?;

        // === DATA ===
        let mut data_builder = QueryBuilder::new(format!(
            "SELECT {} FROM {} {} ORDER BY 1 DESC",
            self.select_clause(),
            self.from_clause(),
            where_clause
        ));

        data_builder
            .push(" OFFSET ")
            .push_bind(offset as i64)
            .push(" LIMIT ")
            .push_bind(page_size as i64);

        let data = data_builder.build_query_as::<T>().fetch_all(pool).await?;

        let total_pages: i32 = if total.0 == 0 {
            1
        } else {
            ((total.0 as f32) / (page_size as f32)).ceil() as i32
        };

        Ok(PaginatedResponse {
            data,
            total_records: total.0,
            page,
            page_size,
            total_pages,
        })
    }

    async fn get_by_id(&self, pool: &PgPool, id: ID) -> Result<T, sqlx::Error> {
        let query = format!(
            "SELECT * FROM {} WHERE {} = $1 LIMIT 1",
            self.table_name(),
            self.id_column()
        );

        sqlx::query_as(&query).bind(id).fetch_one(pool).await
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<T, sqlx::Error>;

    async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: Self::UpdateInput,
    ) -> Result<T, sqlx::Error>;

    async fn delete(&self, pool: &PgPool, id: ID) -> Result<(), sqlx::Error>;
}

//#[derive(Debug, Serialize, FromRow)]
//pub struct Module {
//    pub id: i32,
//   pub title: String,
//}

pub struct ModuleRepository;

#[async_trait]
impl Repository<Module, i32> for ModuleRepository {
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
            "INSERT INTO module (title) VALUES ($1) RETURNING id, title, created_at, updated_at",
            input.title.to_string()
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
impl Repository<Permission, i32> for PermissionRepository {
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

    async fn create(
        &self,
        pool: &PgPool,
        input: Self::CreateInput,
    ) -> Result<Permission, sqlx::Error> {
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

pub struct PerfilRepository;

#[async_trait]
impl Repository<Perfil, i32> for PerfilRepository {
    type CreateInput = PerfilCreateSchema;
    type UpdateInput = PerfilUpdateSchema;

    fn table_name(&self) -> &str {
        "roles"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["p.name",]
    }

    fn select_clause(&self) -> &str {
        "p.id, p.name"
    }

    fn from_clause(&self) -> &str {
        "roles p"
    }

    async fn create(
        &self,
        pool: &PgPool,
        input: Self::CreateInput,
    ) -> Result<Perfil, sqlx::Error> {
        sqlx::query_as!(
            Perfil,
            r#"INSERT INTO roles (name) 
               VALUES ($1) 
               RETURNING id, name"#,
            input.name,

        )
        .fetch_one(pool)
        .await
    }

    async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: Self::UpdateInput,
    ) -> Result<Perfil, sqlx::Error> {
        sqlx::query_as!(
            Perfil,
            r#"UPDATE roles 
               SET name = $1
               WHERE id = $2
               RETURNING id, name"#,
            input.name,
            id
        )
        .fetch_one(pool)
        .await
    }

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM roles WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}