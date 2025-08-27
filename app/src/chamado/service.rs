use std::{collections::HashSet, fs};

use anyhow::Result;
use anyhow::anyhow;
use serde_json::{Value, from_str};
use shared::{PaginatedResponse, Repository};
use sqlx::PgPool;

use crate::chamado::model::GerenciamentoChamado;
use crate::chamado::model::ImagemChamado;
use crate::chamado::repository::GerenciamentoChamadoRepository;
use crate::chamado::schema::CreateGerenciamentoChamado;
use crate::chamado::schema::UpdateGerenciamentoChamado;
use crate::chamado::StatusChamado;
use crate::{
    chamado::{
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
    },
    permissao::{User, UserRolesService},
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
        match self.get_by_id(pool, id).await {
            Ok(old_chamado) => {
                Self::cleanup_images(old_chamado, input.descricao.clone());
            }
            Err(_) => {}
        }

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

    /*
       retorna lista de chamados do usuario
    */
    pub async fn get_paginated_ownership(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
        user_id: i64,
    ) -> Result<PaginatedResponse<Chamado>> {
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
            SELECT COUNT(p) FROM chamado_chamados p
            WHERE p.titulo ILIKE $1 and p.user_solic_id = $2
            "#,
        )
        .bind(&like_term)
        .bind(user_id)
        .fetch_one(pool)
        .await?;

        let total_records = total.0;
        let total_pages = if total_records == 0 {
            1
        } else {
            ((total_records as f64) / (page_size as f64)).ceil() as i32
        };

        // Busca os registros paginados
        let items: Vec<Chamado> = sqlx::query_as!(
            Chamado,
            r#"
            SELECT 
                id,
                titulo,
                descricao,
                status,
                created_at,
                updated_at,
                user_solic_id,
                servico_id,
                tipo_id
            FROM chamado_chamados p
            WHERE p.titulo ILIKE $1 and p.user_solic_id = $2
            ORDER BY p.id DESC
            LIMIT $3 OFFSET $4
            "#,
            like_term,
            user_id,
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

    pub async fn update_status(
        pool: &PgPool,
        id: i64,
        status: i32,
    ) -> Result<Chamado> {
        // valida status
        StatusChamado::try_from(status); 

        Ok(sqlx::query_as!(
            Chamado,
            r#"
            UPDATE chamado_chamados
            SET 
                status = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING *"#,
            status,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    // Função auxiliar para verificar se o usuário é dono do chamado
    pub async fn verify_chamado_ownership(db: &PgPool, user_id: i64, chamado_id: i64) -> bool {
        if chamado_id == 0 {
            return false;
        }

        match sqlx::query!(
            "SELECT user_solic_id FROM chamado_chamados WHERE id = $1",
            chamado_id
        )
        .fetch_optional(db)
        .await
        {
            Ok(Some(record)) => record.user_solic_id == user_id,
            Ok(None) => false, // Chamado não existe
            Err(_) => false,   // Erro na consulta
        }
    }

    /*
       verifica se usuario tem permissao de ver chamado
    */
    pub async fn can_access(user: &User, object_id: i64, db: &PgPool) -> bool {
        // Verifica se o usuário é um superusuário
        if user.is_superuser {
            return true;
        }

        //pegar todas as permissões pelos perfies do usuario.
        let list_permissao = match UserRolesService::get_user_permissions(&db, user.id).await {
            Ok(permissions) => permissions,
            Err(_) => return false, // Em caso de erro, nega acesso
        };

        for permissao in list_permissao {
            // Se tiver permissão de admin, liberar tudo
            match permissao.as_str() {
                "chamado_admin" => return true,
                _ => continue, // Continua verificando outras permissões
            }
        }
        // Se nenhuma permissão foi encontrada, verificar se é dono do chamado
        ChamadoService::verify_chamado_ownership(&db, user.id, object_id).await
    }

    /* pub async fn get_by_titulo(&self, pool: &PgPool, titulo: String) -> Result<Chamado> {
        let query = format!(
            "SELECT {} FROM {} WHERE m.titulo = '$1' LIMIT 1",
            self.repo.select_clause(),
            self.repo.from_clause()
        );

        Ok(sqlx::query_as(&query).bind(titulo).fetch_one(pool).await?)
    } */
    fn cleanup_images(old_chamado: Chamado, new_value: Value) {
        // Extrai URLs das imagens do chamado antigo e do novo
        let old_images: HashSet<String> = Self::extrair_urls_imagens(old_chamado)
            .unwrap_or_default()
            .into_iter()
            .collect();

        let new_images: HashSet<String> = Self::extrair_imagens_do_value(&new_value)
            .unwrap_or_default()
            .into_iter()
            .collect();

        // Imagens que estavam antes mas não existem mais → deletar
        for url in old_images.difference(&new_images) {
            if url.starts_with('/') {
                // evita deletar URLs externas
                let path = &url[1..]; // remove "/" inicial
                if let Err(e) = fs::remove_file(path) {
                    eprintln!("Erro ao deletar {}: {}", url, e);
                } else {
                    println!("Arquivo deletado: {}", url);
                }
            }
        }
    }

    /// Versão simplificada que retorna apenas URLs das imagens
    pub fn extrair_urls_imagens(chamando: Chamado) -> Result<Vec<String>> {
        let imagens = chamando.extrair_imagens()?;
        let urls = imagens
            .into_iter()
            .map(|img| img.url)
            .filter(|url| !url.is_empty())
            .collect();

        Ok(urls)
    }

    fn extrair_imagens_do_value(descricao: &Value) -> Result<Vec<String>> {
        let mut imagens = Vec::new();

        // Converte Value para string se necessário, e depois parseia para JSON
        let descricao_str = match descricao {
            Value::String(s) => s.as_str(),
            _ => return Ok(Vec::new()), // Se não for string, retorna vazio
        };

        // Parseia a string JSON para Value
        let parsed: Value = serde_json::from_str(descricao_str)
            .map_err(|e| anyhow!("Erro ao parsear JSON: {}", e))?;

        // Obtém o array de blocks
        let blocks = parsed
            .get("blocks")
            .ok_or_else(|| anyhow!("Campo 'blocks' não encontrado"))?
            .as_array()
            .ok_or_else(|| anyhow!("'blocks' não é um array"))?;

        // Itera por todos os blocks
        for block in blocks {
            if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                if block_type == "image" {
                    if let Some(data) = block.get("data") {
                        let imagem = ImagemChamado {
                            url: data
                                .get("file")
                                .and_then(|f| f.get("url"))
                                .and_then(|u| u.as_str())
                                .unwrap_or("")
                                .to_string(),
                            caption: data
                                .get("caption")
                                .and_then(|c| c.as_str())
                                .unwrap_or("")
                                .to_string(),
                            with_border: data
                                .get("withBorder")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(false),
                            stretched: data
                                .get("stretched")
                                .and_then(|s| s.as_bool())
                                .unwrap_or(false),
                            with_background: data
                                .get("withBackground")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(false),
                        };

                        // Só adiciona se tiver URL
                        if !imagem.url.is_empty() {
                            imagens.push(imagem);
                        }
                    }
                }
            }
        }

        let urls = imagens
            .into_iter()
            .map(|img| img.url)
            .filter(|url| !url.is_empty())
            .collect();

        Ok(urls)
    }
}


pub struct GerenciamentoChamadoService {
    repo: GerenciamentoChamadoRepository,
}

impl GerenciamentoChamadoService {
    pub fn new() -> Self {
        Self {
            repo: GerenciamentoChamadoRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i64) -> Result<GerenciamentoChamado> {
        Repository::<GerenciamentoChamado, i64>::get_by_id(&self.repo, pool, id).await
    }

    pub async fn get_by_chamado_id(&self, pool: &PgPool, chamado_id: i64) -> Result<GerenciamentoChamado> {
        let query = format!(
            "SELECT {} FROM {} WHERE chamado_id = $1 LIMIT 1",
            self.repo.select_clause(),
            self.repo.from_clause(),
        );

        Ok(sqlx::query_as(&query).bind(chamado_id).fetch_one(pool).await?)
    }

    pub async fn create(
        &self,
        pool: &PgPool,
        input: CreateGerenciamentoChamado,
    ) -> Result<GerenciamentoChamado> {
        self.repo.create(pool, input).await
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        input: UpdateGerenciamentoChamado,
    ) -> Result<GerenciamentoChamado> {
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
    ) -> Result<PaginatedResponse<GerenciamentoChamado>> {
        Repository::<GerenciamentoChamado, i64>::get_paginated(&self.repo, pool, find, page, page_size)
            .await
    }

}