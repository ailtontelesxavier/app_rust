use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Deserialize)]
pub struct GenericListParams {
    pub page: Option<usize>,
    pub search: Option<String>,
    pub flash_message: Option<String>,
    pub flash_status: Option<String>,
}

#[derive(Serialize)]
pub struct ListConfig {
    pub entity_name: String,
    pub entity_label: String,
    pub plural_label: String,
    pub base_url: String,
    pub fields: Vec<(String, String)>, // (field_name, display_label)
    pub searchable_fields: Vec<String>,
    pub items_per_page: usize,
}

#[async_trait::async_trait]
pub trait GenericListHandler {
    type Item: serde::Serialize + Send + Sync;

    async fn count_items(
        &self,
        pool: &PgPool,
        params: &GenericListParams,
    ) -> Result<usize, sqlx::Error>;

    async fn fetch_items(
        &self,
        pool: &PgPool,
        params: &GenericListParams,
    ) -> Result<Vec<Self::Item>, sqlx::Error>;
}

#[async_trait::async_trait]
pub trait GenericCrudHandler: GenericListHandler {
    type CreateData: for<'de> serde::Deserialize<'de> + Send;
    type UpdateData: for<'de> serde::Deserialize<'de> + Send;

    async fn create_item(
        &self,
        pool: &PgPool,
        data: Self::CreateData,
    ) -> Result<Self::Item, sqlx::Error>;
    async fn update_item(
        &self,
        pool: &PgPool,
        id: i32,
        data: Self::UpdateData,
    ) -> Result<Self::Item, sqlx::Error>;
    async fn delete_item(&self, pool: &PgPool, id: i32) -> Result<u64, sqlx::Error>;
    async fn get_item_by_id(&self, pool: &PgPool, id: i32) -> Result<Self::Item, sqlx::Error>;
}
