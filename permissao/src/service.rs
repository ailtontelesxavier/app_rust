use crate::{
    model::module::{Perfil, Permission, PermissionWithModule, User},
    repository::{self, Repository},
    schema::{
        PerfilCreateSchema, PerfilUpdateSchema, PermissionCreateSchema, PermissionUpdateSchema,
        UserCreateSchema, UserUpdateSchema,
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
use shared::{SharedState};

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
            Self::get_password_hash(&Self::random_base32().to_string()).unwrap_or(
                "NovaSenhaTeste!!####".to_string()
            )
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
