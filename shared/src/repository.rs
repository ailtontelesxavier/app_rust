use anyhow::Ok;
use anyhow::Result;
use async_trait::async_trait;
use serde::Deserialize;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{Encode, FromRow, PgPool, Postgres, Type, postgres::PgRow};
use std::fmt::Display;

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total_records: i64,
    pub page: i32,
    pub page_size: i32,
    pub total_pages: i32,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub find: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Deserialize)]
pub struct ListParams {
    pub page: Option<i32>,
    pub find: Option<String>,
    pub msg: Option<String>,
    pub status: Option<String>,
    pub page_size: Option<i32>,
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

    fn select_clause_view(&self) -> &str {
        self.select_clause()
    }
    fn from_clause_view(&self) -> &str {
        self.from_clause()
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
    ) -> Result<PaginatedResponse<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let page = page.max(1);
        let page_size = page_size.min(100);
        let offset = (page - 1) * page_size;

        // Construir WHERE clause com parâmetros seguros
        let (where_clause, params) = if let Some(term) = find {
            let search_fields = self.searchable_fields();
            if !search_fields.is_empty() {
                let mut field_parts = Vec::new();
                for field in search_fields {
                    field_parts.push(format!("{} ILIKE $1", field));
                }
                let where_str = format!("WHERE ({})", field_parts.join(" OR "));
                (where_str, vec![format!("%{}%", term)])
            } else {
                (String::new(), vec![])
            }
        } else {
            (String::new(), vec![])
        };

        // === COUNT ===
        let count_query = format!(
            "SELECT COUNT(*) FROM {} {}",
            self.from_clause(),
            where_clause
        );

        let total: (i64,) = if params.is_empty() {
            sqlx::query_as(&count_query).fetch_one(pool).await?
        } else {
            sqlx::query_as(&count_query)
                .bind(&params[0])
                .fetch_one(pool)
                .await?
        };

        // === DATA ===
        let data_query = format!(
            "SELECT {} FROM {} {} ORDER BY {} DESC LIMIT ${} OFFSET ${}",
            self.select_clause(),
            self.from_clause(),
            where_clause,
            self.id_column(),
            if params.is_empty() { 1 } else { 2 },
            if params.is_empty() { 2 } else { 3 }
        );

        let data = if params.is_empty() {
            sqlx::query_as::<_, T>(&data_query)
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_as::<_, T>(&data_query)
                .bind(&params[0])
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(pool)
                .await?
        };

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

    async fn get_by_id(&self, pool: &PgPool, id: ID) -> anyhow::Result<T> {
        let query = format!(
            "SELECT {} FROM {} WHERE {} = $1 LIMIT 1",
            self.select_clause(),
            self.from_clause(),
            self.id_column()
        );

        Ok(sqlx::query_as(&query).bind(id).fetch_one(pool).await?)
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<T>;

    async fn update(&self, pool: &PgPool, id: ID, input: Self::UpdateInput) -> Result<T>;

    async fn delete(&self, pool: &PgPool, id: ID) -> Result<()>;

    async fn get_paginated_view(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<T>>
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
    {
        let page = page.max(1);
        let page_size = page_size.min(100);
        let offset = (page - 1) * page_size;

        // Construir WHERE clause com parâmetros seguros
        let (where_clause, params) = if let Some(term) = find {
            let search_fields = self.searchable_fields();
            if !search_fields.is_empty() {
                let mut field_parts = Vec::new();
                for field in search_fields {
                    field_parts.push(format!("{} ILIKE $1", field));
                }
                let where_str = format!("WHERE ({})", field_parts.join(" OR "));
                (where_str, vec![format!("%{}%", term)])
            } else {
                (String::new(), vec![])
            }
        } else {
            (String::new(), vec![])
        };

        // === COUNT ===
        let count_query = format!(
            "SELECT COUNT(*) FROM {} {}",
            self.from_clause(),
            where_clause
        );

        let total: (i64,) = if params.is_empty() {
            sqlx::query_as(&count_query).fetch_one(pool).await?
        } else {
            sqlx::query_as(&count_query)
                .bind(&params[0])
                .fetch_one(pool)
                .await?
        };

        // === DATA ===
        let data_query = format!(
            "SELECT {} FROM {} {} ORDER BY {} DESC LIMIT ${} OFFSET ${}",
            self.select_clause_view(),
            self.from_clause_view(),
            where_clause,
            self.id_column(),
            if params.is_empty() { 1 } else { 2 },
            if params.is_empty() { 2 } else { 3 }
        );

        let data = if params.is_empty() {
            sqlx::query_as::<_, T>(&data_query)
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(pool)
                .await?
        } else {
            sqlx::query_as::<_, T>(&data_query)
                .bind(&params[0])
                .bind(page_size as i64)
                .bind(offset as i64)
                .fetch_all(pool)
                .await?
        };

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
}
