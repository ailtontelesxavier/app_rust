use anyhow::Ok;
use anyhow::Result;
use async_trait::async_trait;
use shared::PaginatedResponse;
use shared::Repository;
use sqlx::PgPool;

use crate::permissao::model::module::RolePermission;
use crate::permissao::model::module::UserRoles;
use crate::permissao::schema::RolePermissionCreateSchema;
use crate::permissao::schema::RolePermissionUpdateSchema;
use crate::permissao::schema::UserRolesCreateSchema;
use crate::permissao::schema::UserRolesUpdateSchema;
use crate::{
    permissao::model::module::{Module, Perfil, Permission, User},
    permissao::schema::{
        CreateModuleSchema, PerfilCreateSchema, PerfilUpdateSchema, PermissionCreateSchema,
        PermissionUpdateSchema, UpdateModuleSchema, UserCreateSchema, UserUpdateSchema,
    },
};

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

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<Module> {
        Ok(sqlx::query_as!(
            Module,
            "INSERT INTO module (title) VALUES ($1) RETURNING id, title, created_at, updated_at",
            input.title.to_string()
        )
        .fetch_one(pool)
        .await?)
    }

    async fn update(&self, pool: &PgPool, id: i32, input: Self::UpdateInput) -> Result<Module> {
        Ok(sqlx::query_as!(
            Module,
            r#"UPDATE module SET title = $1 WHERE id = $2 RETURNING id, title, created_at, updated_at"#,
            input.title,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<()> {
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

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<Permission> {
        Ok(sqlx::query_as!(
            Permission,
            r#"INSERT INTO permission (name, description, module_id) 
               VALUES ($1, $2, $3) 
               RETURNING id, name, description, module_id, created_at, updated_at"#,
            input.name,
            input.description,
            input.module_id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn update(&self, pool: &PgPool, id: i32, input: Self::UpdateInput) -> Result<Permission> {
        Ok(sqlx::query_as!(
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
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<()> {
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
        &["p.name"]
    }

    fn select_clause(&self) -> &str {
        "p.id, p.name"
    }

    fn from_clause(&self) -> &str {
        "roles p"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<Perfil> {
        Ok(sqlx::query_as!(
            Perfil,
            r#"INSERT INTO roles (name) 
               VALUES ($1) 
               RETURNING id, name"#,
            input.name,
        )
        .fetch_one(pool)
        .await?)
    }

    async fn update(&self, pool: &PgPool, id: i32, input: Self::UpdateInput) -> Result<Perfil> {
        Ok(sqlx::query_as!(
            Perfil,
            r#"UPDATE roles 
               SET name = $1
               WHERE id = $2
               RETURNING id, name"#,
            input.name,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM roles WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct UserRepository;

#[async_trait]
impl Repository<User, i64> for UserRepository {
    type CreateInput = UserCreateSchema;
    type UpdateInput = UserUpdateSchema;

    fn table_name(&self) -> &str {
        "users"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["u.username", "u.email", "u.full_name"]
    }

    fn select_clause(&self) -> &str {
        "u.id, u.username, u.password, u.email, u.full_name, u.otp_base32, \
        u.is_active, u.is_staff, u.is_superuser, u.ip_last_login, \
        u.last_login, u.created_at, u.updated_at"
    }

    fn from_clause(&self) -> &str {
        "users u"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> anyhow::Result<User> {
        // 1. Checar se já existe usuário com email ou username
        if let Some(db_user) = sqlx::query_as!(
            User,
            r#"
            SELECT * FROM users
            WHERE email = $1 OR username = $2
            "#,
            input.email,
            input.username
        )
        .fetch_optional(pool)
        .await?
        {
            if db_user.email == input.email {
                anyhow::bail!("Email already registered");
            } else {
                anyhow::bail!("Username already registered");
            }
        }

        // 2. Inserir novo usuário
        let new_user = sqlx::query_as!(
            User,
            r#"INSERT INTO users (username, password, email, full_name, otp_base32, is_active, is_staff, is_superuser) 
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
               RETURNING * "#,
            input.username,
            input.password,
            input.email,
            input.full_name,
            input.otp_base32,
            input.is_active,
            input.is_staff,
            input.is_superuser
        )
        .fetch_one(pool)
        .await?;

        Ok(new_user)
    }

    async fn update(&self, pool: &PgPool, id: i64, input: Self::UpdateInput) -> Result<User> {
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
            input.username,
            input.email,
            input.full_name,
            input.otp_base32,
            input.is_active,
            input.is_staff,
            input.is_superuser,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query!(r#"DELETE FROM users WHERE id = $1"#, id as i64)
            .execute(pool)
            .await?;
        Ok(())
    }
}

pub struct UserRolesRepository;

#[async_trait]
impl Repository<UserRoles, i32> for UserRolesRepository {
    type CreateInput = UserRolesCreateSchema;
    type UpdateInput = UserRolesUpdateSchema;

    fn table_name(&self) -> &str {
        "user_roles"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["p.user_id", "p.role_id"]
    }

    fn select_clause(&self) -> &str {
        "p.id, p.user_id, p.role_id"
    }

    fn from_clause(&self) -> &str {
        "user_roles p"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<UserRoles> {
        Ok(sqlx::query_as!(
            UserRoles,
            r#"INSERT INTO user_roles (user_id, role_id) 
               VALUES ($1, $2) 
               RETURNING id, user_id, role_id"#,
            input.user_id,
            input.role_id,
        )
        .fetch_one(pool)
        .await?)
    }

    async fn update(&self, pool: &PgPool, id: i32, input: Self::UpdateInput) -> Result<UserRoles> {
        Ok(sqlx::query_as!(
            UserRoles,
            r#"UPDATE user_roles 
               SET role_id = $1
               WHERE id = $2
               RETURNING id, user_id, role_id"#,
            input.role_id,
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i32) -> Result<()> {
        sqlx::query!("DELETE FROM user_roles WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }
}


pub struct RolePermissionRepository;

#[async_trait]
impl Repository<RolePermission, i64> for RolePermissionRepository {
    type CreateInput = RolePermissionCreateSchema;
    type UpdateInput = RolePermissionUpdateSchema;

    fn table_name(&self) -> &str {
        "role_permissions"
    }

    fn searchable_fields(&self) -> &[&str] {
        &["p.role_id", "p.permission_id"]
    }

    fn select_clause(&self) -> &str {
        "p.id, p.role_id, p.permission_id"
    }

    fn from_clause(&self) -> &str {
        "role_permissions p"
    }

    async fn create(&self, pool: &PgPool, input: Self::CreateInput) -> Result<RolePermission> {
        Ok(sqlx::query_as!(
            RolePermission,
            r#"INSERT INTO role_permissions (role_id, permission_id) 
               VALUES ($1, $2) 
               RETURNING id, role_id, permission_id"#,
            input.role_id,
            input.permission_id,
        )
        .fetch_one(pool)
        .await?)
    }

    async fn update(&self, pool: &PgPool, id: i64, input: Self::UpdateInput) -> Result<RolePermission> {
        Ok(sqlx::query_as!(
            RolePermission,
            r#"UPDATE role_permissions 
               SET role_id = $1, permission_id = $2
               WHERE id = $3
               RETURNING id, role_id, permission_id"#,
            input.role_id,
            input.permission_id,
            id as i32
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM role_permissions WHERE id = $1", id as i32)
            .execute(pool)
            .await?;
        Ok(())
    }

}
