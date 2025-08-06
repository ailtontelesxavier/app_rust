use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, FromRow, postgres::PgRow};
use std::collections::HashMap;

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
    type CreateInput: Deserialize<'static> + Send + Sync;
    type UpdateInput: Deserialize<'static> + Send + Sync;

    fn table_name(&self) -> &str;
    fn searchable_fields(&self) -> &[&str];
    fn select_clause(&self) -> &str;   // SELECT com JOINs e alias
    fn from_clause(&self) -> &str;     // FROM + JOIN

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
        let offset = (page.max(1) - 1) * page_size;
        let page_size = page_size.min(100);

        let mut filters = vec![];
        let mut args: Vec<String> = vec![];
        let mut param_idx = 1;

        if let Some(find) = find {
            let like = format!("%{}%", find);
            for field in self.searchable_fields() {
                filters.push(format!(r#"{} ILIKE ${}"#, field, param_idx));
                args.push(like.clone());
                param_idx += 1;
            }
        }

        let mut where_clause = if !filters.is_empty() {
            format!("({})", filters.join(" OR "))
        } else {
            "TRUE".to_string()
        };

        if let Some(extra) = self.extra_where() {
            where_clause = format!("{} AND ({})", where_clause, extra);
        }

        // Total
        let count_sql = format!("SELECT COUNT(*) FROM {} WHERE {}", self.from_clause(), &where_clause);
        let total: (i64,) = sqlx::query_as_with(&count_sql, args.clone())
            .fetch_one(pool)
            .await?;

        // Dados
        args.push(offset.to_string());
        args.push(page_size.to_string());

        let data_sql = format!(
            "SELECT {} FROM {} WHERE {} ORDER BY 1 DESC OFFSET ${} LIMIT ${}",
            self.select_clause(),
            self.from_clause(),
            where_clause,
            param_idx,
            param_idx + 1
        );

        let data = sqlx::query_as_with::<_, T, _>(&data_sql, args)
            .fetch_all(pool)
            .await?;

        let total_pages = if total.0 == 0 {
            1
        } else {
            ((total.0 as f32) / (page_size as f32)).ceil() as u32
        };

        Ok(PaginatedResponse {
            data,
            total_records: total.0,
            page,
            page_size,
            total_pages,
        })
    }

    async fn create(
        &self,
        pool: &PgPool,
        input: Self::CreateInput,
    ) -> Result<T, sqlx::Error>;

    async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: Self::UpdateInput,
    ) -> Result<T, sqlx::Error>;

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<(), sqlx::Error>;
}

#[derive(Debug, Serialize, FromRow)]
pub struct Module {
    pub id: i32,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateModule {
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModule {
    pub title: String,
}


pub struct ModuleRepository;

#[async_trait]
impl Repository<Module> for ModuleRepository {
    type CreateInput = CreateModule;
    type UpdateInput = UpdateModule;

    fn table_name(&self) -> &str {
        "module"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["m.title"]
    }

    fn select_clause(&self) -> &str {
        "m.id, m.title"
    }

    fn from_clause(&self) -> &str {
        "module m"
    }

    async fn create(
        &self,
        pool: &PgPool,
        input: Self::CreateInput,
    ) -> Result<Module, sqlx::Error> {
        sqlx::query_as!(
            Module,
            r#"INSERT INTO module (title) VALUES ($1) RETURNING id, title"#,
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
            r#"UPDATE module SET title = $1 WHERE id = $2 RETURNING id, title"#,
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
