use anyhow::Ok;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};
use sqlx::{Encode, FromRow, PgPool, Postgres, Type, postgres::PgRow};
use std::fmt::Display;
use tracing::{debug, info};

use crate::permissao::model::module::PermissionWithModule;
use crate::permissao::model::module::RolePermission;
use crate::permissao::model::module::UserRoles;
use crate::permissao::schema::RolePermissionCreateSchema;
use crate::permissao::schema::RolePermissionUpdateSchema;
use crate::permissao::schema::UserRolesCreateSchema;
use crate::permissao::schema::UserRolesUpdateSchema;
use crate::permissao::schema::UserRolesViewSchema;
use crate::{
    permissao::model::module::{Module, Perfil, Permission, User},
    permissao::schema::{
        CreateModuleSchema, PerfilCreateSchema, PerfilUpdateSchema, PermissionCreateSchema,
        PermissionUpdateSchema, UpdateModuleSchema, UserCreateSchema, UserUpdateSchema,
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

    async fn get_by_id(&self, pool: &PgPool, id: ID) -> Result<T> {
        let query = format!(
            "SELECT * FROM {} WHERE {} = $1 LIMIT 1",
            self.table_name(),
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
            id
        )
        .fetch_one(pool)
        .await?)
    }

    async fn delete(&self, pool: &PgPool, id: i64) -> Result<()> {
        sqlx::query!("DELETE FROM role_permissions WHERE id = $1", id)
            .execute(pool)
            .await?;
        Ok(())
    }

}
