use crate::permissao::{
    self, model::module::{Perfil, Permission, RolePermission, User, UserRoles}, repository::{PerfilRepository, PermissionRepository, RolePermissionRepository, UserRepository, UserRolesRepository}, schema::{
        PerfilCreateSchema, PerfilUpdateSchema, PermissionCreateSchema, PermissionModuloSchema, PermissionUpdateSchema, RolePermissionCreateSchema, RolePermissionUpdateSchema, RolePermissionViewSchema, UserCreateSchema, UserPasswordUpdateSchema, UserRolesCreateSchema, UserRolesUpdateSchema, UserRolesViewSchema, UserUpdateSchema
    }, ModuleRepository};
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
use shared::{PaginatedResponse, Repository, SharedState};

use crate::{
    permissao::model::module::Module,
    permissao::schema::{CreateModuleSchema, UpdateModuleSchema},
};

pub struct ModuleService {
    repo: ModuleRepository,
}

impl ModuleService {
    pub fn new() -> Self {
        Self {
            repo: super::ModuleRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i32) -> Result<Module> {
        Repository::<Module, i32>::get_by_id(&self.repo, pool, id).await
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
        Repository::<Module, i32>::get_paginated(&self.repo, pool, find, page, page_size).await
    }
}

pub struct PermissionService {
    repo: PermissionRepository,
}

impl PermissionService {
    pub fn new() -> Self {
        Self {
            repo: PermissionRepository,
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
    ) -> Result<PaginatedResponse<Permission>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }

    pub async fn get_paginated_with_module(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<PaginatedResponse<PermissionModuloSchema>> {
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

        Ok(PaginatedResponse {
            data: items,
            total_records,
            page,
            page_size,
            total_pages,
        })
    }
}

pub struct PerfilService {
    repo: PerfilRepository,
}

impl PerfilService {
    pub fn new() -> Self {
        Self {
            repo: PerfilRepository,
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
    ) -> Result<PaginatedResponse<Perfil>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }
}

pub struct UserService {
    repo: UserRepository,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            repo: UserRepository,
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
        // 1. Buscar usuário atual
        let current = self.repo.get_by_id(pool, id).await?;

        // 2. Mesclar dados novos com atuais
        let updated_user = Self::apply_to(&current, input);

        Ok(sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET 
                username = $1,
                email = $2,
                full_name = $3,
                otp_base32 = $4,
                is_active = $5,
                is_staff = $6,
                is_superuser = $7,
                updated_at = NOW()
            WHERE id = $8
            RETURNING 
                id, username, password, email, full_name, otp_base32,
                is_active, is_staff, is_superuser, ip_last_login,
                last_login, created_at, updated_at
            "#,
            updated_user.username,
            updated_user.email,
            updated_user.full_name,
            updated_user.otp_base32,
            updated_user.is_active,
            updated_user.is_staff,
            updated_user.is_superuser,
            id
        )
        .fetch_one(pool)
        .await?)
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
    ) -> Result<PaginatedResponse<User>> {
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
    pub fn random_base32() -> String {
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

    /* 
        Utilizado somente por admins super user
     */
    pub async fn update_otp(
        pool: &PgPool,
        id: i64,
    ) -> Result<User> {

        let base = &Self::random_base32().to_string();

        let hash = Self::gerar_otp(base);

        Ok(sqlx::query_as!(
            User,
            r#"
            UPDATE users
            SET 
                otp_base32 = $1,
                updated_at = NOW()
            WHERE id = $2
            RETURNING 
                id, username, password, email, full_name, otp_base32,
                is_active, is_staff, is_superuser, ip_last_login,
                last_login, created_at, updated_at
            "#,
            hash,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    /// Preenche os `None` com os valores atuais do usuário do banco
    fn apply_to(current: &User, input: UserUpdateSchema) -> User {
        User {
            id: current.id,
            username: input.username.unwrap_or_else(|| current.username.clone()),
            email: input.email.unwrap_or_else(|| current.email.clone()),
            full_name: input.full_name.clone().unwrap_or_else(|| current.full_name.clone()),
            otp_base32: input.otp_base32.clone().or_else(|| current.otp_base32.clone()),
            is_active: input.is_active,
            is_staff: input.is_staff,
            is_superuser: input.is_superuser,
            ip_last_login: input.ip_last_login.clone().or_else(|| current.ip_last_login.clone()),
            created_at: current.created_at,   // mantém campos imutáveis
            updated_at: chrono::Utc::now(),
            last_login: current.last_login,
            password: current.password.clone(),
        }
    }
}

pub struct UserRolesService {
    repo: UserRolesRepository,
}

impl UserRolesService {
    pub fn new() -> Self {
        Self {
            repo: UserRolesRepository,
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
    ) -> Result<PaginatedResponse<UserRoles>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }

    pub async fn get_paginated_with_roles(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<PaginatedResponse<UserRolesViewSchema>> {
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

        Ok(PaginatedResponse {
            data: items,
            total_records,
            page,
            page_size,
            total_pages,
        })
    }
}

pub struct RolePermissionService {
    repo: RolePermissionRepository,
}

impl RolePermissionService {
    pub fn new() -> Self {
        Self {
            repo: RolePermissionRepository,
        }
    }

    pub async fn get_by_id(&self, pool: &PgPool, id: i64) -> Result<RolePermission> {
        Ok(self.repo.get_by_id(pool, id).await?)
    }

    pub async fn create(&self, pool: &PgPool, input: RolePermissionCreateSchema) -> Result<RolePermission> {
        Ok(self.repo.create(pool, input).await?)
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        id: i64,
        input: RolePermissionUpdateSchema,
    ) -> Result<RolePermission> {
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
    ) -> Result<PaginatedResponse<RolePermission>> {
        Ok(self.repo.get_paginated(pool, find, page, page_size).await?)
    }

    pub async fn get_paginated_with_permission(
        &self,
        pool: &PgPool,
        find: Option<&str>,
        page: i64,
        page_size: i64,
    ) -> Result<PaginatedResponse<RolePermissionViewSchema>> {
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
            SELECT COUNT(p) FROM role_permissions p
            INNER JOIN permission r ON r.id = p.permission_id
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
        let items: Vec<RolePermissionViewSchema> = sqlx::query_as!(
            RolePermissionViewSchema,
            r#"
            SELECT 
                p.id, 
                p.role_id,
                p.permission_id, 
                r.name
            FROM role_permissions p
            INNER JOIN permission r ON r.id = p.permission_id
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

        Ok(PaginatedResponse {
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
