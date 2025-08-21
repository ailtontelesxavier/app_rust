use crate::permissao::{
    model::module::{Perfil, Permission, User, UserRoles},
    repository::{self, PaginatedResponse, Repository},
    schema::{
        PerfilCreateSchema, PerfilUpdateSchema, PermissionCreateSchema, PermissionModuloSchema,
        PermissionUpdateSchema, UserCreateSchema, UserPasswordUpdateSchema, UserRolesCreateSchema,
        UserRolesUpdateSchema, UserRolesViewSchema, UserUpdateSchema,
    },
};
use anyhow::Result;
use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use base32;
use chrono::{DateTime, TimeZone, Utc};
use chrono_tz::Tz;
use otpauth::TOTP;
use password_hash::rand_core::OsRng;
use rand::Rng;
use sqlx::PgPool;
use validator::Validate;

use axum::{extract::State, response::Html};
use shared::SharedState;

use crate::{
    permissao::model::module::Module,
    permissao::schema::{CreateModuleSchema, UpdateModuleSchema},
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
        self.repo.get_by_id(pool, id).await
    }

    pub async fn create(&self, pool: &PgPool, input: CreateModuleSchema) -> Result<Module> {
        self.repo.create(pool, input).await
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: UpdateModuleSchema,
    ) -> Result<Module> {
        self.repo.update(pool, id, input).await
    }

    pub async fn delete(&self, pool: &PgPool, id: i32) -> Result<()> {
        self.repo.delete(pool, id).await
    }

    pub async fn get_paginated(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<PaginatedResponse<Module>> {
        self.repo.get_paginated(pool, find, page, page_size).await
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

    pub async fn get_paginated_with_module(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<repository::PaginatedResponse<PermissionModuloSchema>> {
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
            SELECT COUNT(*) FROM permission p
            INNER JOIN module m ON p.module_id = m.id
            WHERE p.name ILIKE $1 OR m.title ILIKE $1
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
        let items = sqlx::query_as!(
            PermissionModuloSchema,
            r#"
            SELECT 
                p.id, 
                p.name, 
                p.module_id, 
                m.title as "module_title",
                p.created_at,
                p.updated_at
            FROM permission p
            INNER JOIN module m ON p.module_id = m.id
            WHERE p.name ILIKE $1 OR m.title ILIKE $1
            ORDER BY p.id DESC
            LIMIT $2 OFFSET $3
            "#,
            like_term,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        Ok(repository::PaginatedResponse {
            data: items,
            total_records,
            page,
            page_size,
            total_pages,
        })
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

pub struct UserService {
    repo: repository::UserRepository,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            repo: repository::UserRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i64) -> Result<User> {
        Ok(self.repo.get_by_id(pool, id).await?)
    }

    pub async fn create(&self, pool: &PgPool, input: UserCreateSchema) -> Result<User> {
        let mut input = input;

        input.password = Some(
            Self::get_password_hash(&Self::random_base32().to_string())
                .unwrap_or("NovaSenhaTeste!!####".to_string()),
        );
        input.otp_base32 = Some(Self::random_base32());

        Ok(self.repo.create(pool, input).await?)
    }

    pub async fn update(&self, pool: &PgPool, id: i64, input: UserUpdateSchema) -> Result<User> {
        Ok(self.repo.update(pool, id, input).await?)
    }

    pub async fn delete(&self, pool: &PgPool, id: i64) -> Result<()> {
        Ok(self.repo.delete(pool, id).await?)
    }

    pub async fn get_paginated(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> Result<repository::PaginatedResponse<User>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }

    pub fn get_password_hash(password: &str) -> Result<String, password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);

        // Configura Argon2id com parâmetros recomendados (OWASP 2025)
        let argon2 = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15_000, 2, 1, None).unwrap(), // memória KB, iterações, paralelismo
        );

        Ok(argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string())
    }

    pub fn verify_password(password: &str, hash: &str) -> Result<bool, password_hash::Error> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    /// Gera um segredo aleatório em Base32 para OTP
    fn random_base32() -> String {
        let mut rng = rand::rng();
        let bytes: Vec<u8> = (0..20).map(|_| rng.random()).collect();
        base32::encode(base32::Alphabet::Rfc4648 { padding: false }, &bytes)
    }

    /// Valida o OTP com suporte a fuso horário de São Paulo
    pub fn is_valid_otp(otp: &str, otp_base32: &str, timestamp: Option<DateTime<Utc>>) -> bool {
        let totp = TOTP::new(otp_base32.to_string());

        let time = timestamp.unwrap_or_else(Utc::now);

        // Converte para o fuso horário de São Paulo
        let tz: Tz = "America/Sao_Paulo".parse().unwrap();
        let sao_paulo_time = tz.from_utc_datetime(&time.naive_utc());

        println!("Hora atual em São Paulo: {}", sao_paulo_time);

        // Converte string para número
        let code: u32 = match otp.parse() {
            Ok(c) => c,
            Err(_) => return false, // se não for número, OTP inválido
        };

        // 30 é o período padrão de 30 segundos
        totp.verify(code, 30, sao_paulo_time.timestamp() as u64)
    }

    pub fn gerar_otp(otp_base32: &str) -> String {
        let totp = TOTP::new(otp_base32.to_string());
        totp.generate(30, Utc::now().timestamp() as u64).to_string()
    }

    pub fn get_otp_url(username: &str, otp_base32: &str) -> String {
        let totp = TOTP::new(otp_base32.to_string());
        totp.to_uri("YourApp", username)
    }

    pub async fn update_password(
        pool: &PgPool,
        id: i64,
        input: UserPasswordUpdateSchema,
    ) -> Result<User> {
        let password = Self::get_password_hash(&input.password).unwrap();

        Ok(sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET 
                password = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING 
                id, username, password, email, full_name, otp_base32,
                is_active, is_staff, is_superuser, ip_last_login,
                last_login, created_at, updated_at
            "#,
            password,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    pub async fn get_by_username(pool: &PgPool, username: &str) -> Result<User> {
        let query = format!("SELECT * FROM users WHERE username = $1 LIMIT 1");

        Ok(sqlx::query_as(&query)
            .bind(username)
            .fetch_one(pool)
            .await?)
    }
}

pub struct UserRolesService {
    repo: repository::UserRolesRepository,
}

impl UserRolesService {
    pub fn new() -> Self {
        Self {
            repo: repository::UserRolesRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i32) -> Result<UserRoles> {
        Ok(self.repo.get_by_id(pool, id).await?)
    }

    pub async fn create(&self, pool: &PgPool, input: UserRolesCreateSchema) -> Result<UserRoles> {
        Ok(self.repo.create(pool, input).await?)
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i32,
        input: UserRolesUpdateSchema,
    ) -> Result<UserRoles> {
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
    ) -> Result<repository::PaginatedResponse<UserRoles>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }

    pub async fn get_paginated_with_roles(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<repository::PaginatedResponse<UserRolesViewSchema>> {
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
            SELECT COUNT(p) FROM user_roles p
            INNER JOIN roles r ON r.id = p.role_id
            WHERE r.name ILIKE $1
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
        let items: Vec<UserRolesViewSchema> = sqlx::query_as!(
            UserRolesViewSchema,
            r#"
            SELECT 
                p.id, 
                p.user_id, 
                p.role_id,
                r.name
            FROM user_roles p
            INNER JOIN roles r ON r.id = p.role_id
            WHERE r.name ILIKE $1
            ORDER BY p.id DESC
            LIMIT $2 OFFSET $3
            "#,
            like_term,
            page_size as i64,
            offset as i64
        )
        .fetch_all(pool)
        .await?;

        Ok(repository::PaginatedResponse {
            data: items,
            total_records,
            page,
            page_size,
            total_pages,
        })
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
