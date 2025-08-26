use std::{collections::HashSet, fs};

use anyhow::Result;
use shared::{PaginatedResponse, Repository};
use sqlx::PgPool;

use crate::chamado::{
    model::{CategoriaChamado, Chamado, ServicoChamado, TipoChamado},
    repository::{
        CategoriaChamadoRepository, ChamadoRepository, ServicoChamadoRepository,
        TipoChamadoRepository,
    },
    schema::{
        CreateCategoriaChamadoSchema, CreateChamado, CreateServicoChamadoSchema,
        CreateTipoChamadoSchema, ServicoTipoViewSchema, UpdateCategoriaChamadoSchema,
        UpdateChamado, UpdateServicoChamadoSchema, UpdateTipoChamadoSchema,
    },
};

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

    pub async fn create(
        &self,
        pool: &PgPool,
        input: CreateTipoChamadoSchema,
    ) -> Result<TipoChamado> {
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

    pub async fn get_by_name(&self, pool: &PgPool, nome: String) -> Result<TipoChamado> {
        let query: String = format!(
            "SELECT {} FROM {} WHERE m.nome = '$1' LIMIT 1",
            self.repo.select_clause(),
            self.repo.from_clause()
        );

        Ok(sqlx::query_as(&query).bind(nome).fetch_one(pool).await?)
    }
}

pub struct CategoriaService {
    repo: CategoriaChamadoRepository,
}

impl CategoriaService {
    pub fn new() -> Self {
        Self {
            repo: CategoriaChamadoRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i64) -> Result<CategoriaChamado> {
        Repository::<CategoriaChamado, i64>::get_by_id(&self.repo, pool, id).await
    }

    pub async fn create(
        &self,
        pool: &PgPool,
        input: CreateCategoriaChamadoSchema,
    ) -> Result<CategoriaChamado> {
        self.repo.create(pool, input).await
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        input: UpdateCategoriaChamadoSchema,
    ) -> Result<CategoriaChamado> {
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
    ) -> Result<PaginatedResponse<CategoriaChamado>> {
        Repository::<CategoriaChamado, i64>::get_paginated(&self.repo, pool, find, page, page_size)
            .await
    }

    pub async fn get_by_name(&self, pool: &PgPool, nome: String) -> Result<CategoriaChamado> {
        let query = format!(
            "SELECT {} FROM {} WHERE m.nome = '$1' LIMIT 1",
            self.repo.select_clause(),
            self.repo.from_clause()
        );

        Ok(sqlx::query_as(&query).bind(nome).fetch_one(pool).await?)
    }
}

pub struct ServicoService {
    repo: ServicoChamadoRepository,
}

impl ServicoService {
    pub fn new() -> Self {
        Self {
            repo: ServicoChamadoRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i64) -> Result<ServicoChamado> {
        Repository::<ServicoChamado, i64>::get_by_id(&self.repo, pool, id).await
    }

    pub async fn create(
        &self,
        pool: &PgPool,
        input: CreateServicoChamadoSchema,
    ) -> Result<ServicoChamado> {
        self.repo.create(pool, input).await
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        input: UpdateServicoChamadoSchema,
    ) -> Result<ServicoChamado> {
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
    ) -> Result<PaginatedResponse<ServicoChamado>> {
        Repository::<ServicoChamado, i64>::get_paginated(&self.repo, pool, find, page, page_size)
            .await
    }

    pub async fn get_by_name(&self, pool: &PgPool, nome: String) -> Result<ServicoChamado> {
        let query = format!(
            "SELECT {} FROM {} WHERE m.nome = '$1' LIMIT 1",
            self.repo.select_clause(),
            self.repo.from_clause()
        );

        Ok(sqlx::query_as(&query).bind(nome).fetch_one(pool).await?)
    }

    pub async fn get_paginated_with_tipo(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<ServicoTipoViewSchema>> {
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
            SELECT COUNT(p) FROM chamado_servico_chamado p
            INNER JOIN chamado_tipos_chamado r ON r.id = p.tipo_id
            WHERE r.nome ILIKE $1 or p.nome ILIKE $1
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
        let items: Vec<ServicoTipoViewSchema> = sqlx::query_as!(
            ServicoTipoViewSchema,
            r#"
            SELECT 
                p.id, 
                p.nome,
                p.tipo_id, 
                r.nome as nome_tipo
            FROM chamado_servico_chamado p
            INNER JOIN chamado_tipos_chamado r ON r.id = p.tipo_id
            WHERE r.nome ILIKE $1
            ORDER BY p.id DESC
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

pub struct ChamadoService {
    repo: ChamadoRepository,
}

impl ChamadoService {
    pub fn new() -> Self {
        Self {
            repo: ChamadoRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i64) -> Result<Chamado> {
        Repository::<Chamado, i64>::get_by_id(&self.repo, pool, id).await
    }

    pub async fn create(&self, pool: &PgPool, input: CreateChamado) -> Result<Chamado> {
        self.repo.create(pool, input).await
    }

    pub async fn update(&self, pool: &PgPool, id: i64, input: UpdateChamado) -> Result<Chamado> {
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
    ) -> Result<PaginatedResponse<Chamado>> {
        Repository::<Chamado, i64>::get_paginated(&self.repo, pool, find, page, page_size).await
    }

    /* pub async fn get_by_titulo(&self, pool: &PgPool, titulo: String) -> Result<Chamado> {
        let query = format!(
            "SELECT {} FROM {} WHERE m.titulo = '$1' LIMIT 1",
            self.repo.select_clause(),
            self.repo.from_clause()
        );

        Ok(sqlx::query_as(&query).bind(titulo).fetch_one(pool).await?)
    } */
   fn cleanup_images(old_blocks: &serde_json::Value, new_blocks: &serde_json::Value) {
        let mut old_images = HashSet::new();
        let mut new_images = HashSet::new();

        // coleta URLs antigas
        for block in old_blocks.as_array().unwrap_or(&vec![]) {
            if block["type"] == "image" {
                if let Some(url) = block["data"]["file"]["url"].as_str() {
                    old_images.insert(url.to_string());
                }
            }
        }

        // coleta URLs novas
        for block in new_blocks.as_array().unwrap_or(&vec![]) {
            if block["type"] == "image" {
                if let Some(url) = block["data"]["file"]["url"].as_str() {
                    new_images.insert(url.to_string());
                }
            }
        }

        // imagens que estavam antes mas não existem mais → deletar
        for url in old_images.difference(&new_images) {
            if url.starts_with("/") { // evita deletar URLs externas
                if let Err(e) = fs::remove_file(&url[1..]) { // remove "/" inicial
                    eprintln!("Erro ao deletar {}: {}", url, e);
                } else {
                    println!("Arquivo deletado: {}", url);
                }
            }
        }
    }
}
