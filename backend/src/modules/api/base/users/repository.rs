use std::sync::Mutex;

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::core::db::db_err;
use crate::core::error::ApiResult;
use crate::modules::api::base::users::schemas::User;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn get_by_id(&self, id: Uuid) -> ApiResult<Option<User>>;
    async fn get_by_email(&self, email: &str) -> ApiResult<Option<User>>;
    /// Ordered by email.
    async fn list(&self, skip: i64, limit: i64) -> ApiResult<Vec<User>>;
    async fn count(&self) -> ApiResult<i64>;
    async fn create(&self, user: User) -> ApiResult<User>;
    /// Writes every mutable column of `user` (matched by id).
    async fn update(&self, user: User) -> ApiResult<User>;
    async fn delete(&self, id: Uuid) -> ApiResult<()>;
}

const USER_COLUMNS: &str = "id, email, is_active, is_superuser, full_name, hashed_password";

pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        PgUserRepository { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn get_by_id(&self, id: Uuid) -> ApiResult<Option<User>> {
        let sql = format!("SELECT {} FROM \"user\" WHERE id = $1", USER_COLUMNS);
        sqlx::query_as::<_, User>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)
    }

    async fn get_by_email(&self, email: &str) -> ApiResult<Option<User>> {
        let sql = format!("SELECT {} FROM \"user\" WHERE email = $1", USER_COLUMNS);
        sqlx::query_as::<_, User>(&sql)
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_err)
    }

    async fn list(&self, skip: i64, limit: i64) -> ApiResult<Vec<User>> {
        let sql = format!(
            "SELECT {} FROM \"user\" ORDER BY email OFFSET $1 LIMIT $2",
            USER_COLUMNS
        );
        sqlx::query_as::<_, User>(&sql)
            .bind(skip)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
            .map_err(db_err)
    }

    async fn count(&self) -> ApiResult<i64> {
        let row: (i64,) = sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM \"user\"")
            .fetch_one(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(row.0)
    }

    async fn create(&self, user: User) -> ApiResult<User> {
        let sql = format!(
            "INSERT INTO \"user\" ({}) VALUES ($1, $2, $3, $4, $5, $6) RETURNING {}",
            USER_COLUMNS, USER_COLUMNS
        );
        sqlx::query_as::<_, User>(&sql)
            .bind(user.id)
            .bind(user.email)
            .bind(user.is_active)
            .bind(user.is_superuser)
            .bind(user.full_name)
            .bind(user.hashed_password)
            .fetch_one(&self.pool)
            .await
            .map_err(db_err)
    }

    async fn update(&self, user: User) -> ApiResult<User> {
        let sql = format!(
            "UPDATE \"user\" SET email = $2, is_active = $3, is_superuser = $4, \
             full_name = $5, hashed_password = $6 WHERE id = $1 RETURNING {}",
            USER_COLUMNS
        );
        sqlx::query_as::<_, User>(&sql)
            .bind(user.id)
            .bind(user.email)
            .bind(user.is_active)
            .bind(user.is_superuser)
            .bind(user.full_name)
            .bind(user.hashed_password)
            .fetch_one(&self.pool)
            .await
            .map_err(db_err)
    }

    async fn delete(&self, id: Uuid) -> ApiResult<()> {
        sqlx::query("DELETE FROM \"user\" WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(db_err)?;
        Ok(())
    }
}

/// In-memory repository used by the integration tests.
#[derive(Default)]
pub struct InMemoryUserRepository {
    users: Mutex<Vec<User>>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        InMemoryUserRepository::default()
    }
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn get_by_id(&self, id: Uuid) -> ApiResult<Option<User>> {
        let guard = self.users.lock().unwrap();
        Ok(guard.iter().find(|u| u.id == id).cloned())
    }

    async fn get_by_email(&self, email: &str) -> ApiResult<Option<User>> {
        let guard = self.users.lock().unwrap();
        Ok(guard.iter().find(|u| u.email == email).cloned())
    }

    async fn list(&self, skip: i64, limit: i64) -> ApiResult<Vec<User>> {
        let guard = self.users.lock().unwrap();
        let mut all: Vec<User> = guard.clone();
        all.sort_by(|a, b| a.email.cmp(&b.email));
        Ok(all
            .into_iter()
            .skip(skip.max(0) as usize)
            .take(limit.max(0) as usize)
            .collect())
    }

    async fn count(&self) -> ApiResult<i64> {
        let guard = self.users.lock().unwrap();
        Ok(guard.len() as i64)
    }

    async fn create(&self, user: User) -> ApiResult<User> {
        let mut guard = self.users.lock().unwrap();
        guard.push(user.clone());
        Ok(user)
    }

    async fn update(&self, user: User) -> ApiResult<User> {
        let mut guard = self.users.lock().unwrap();
        if let Some(slot) = guard.iter_mut().find(|u| u.id == user.id) {
            *slot = user.clone();
        }
        Ok(user)
    }

    async fn delete(&self, id: Uuid) -> ApiResult<()> {
        let mut guard = self.users.lock().unwrap();
        guard.retain(|u| u.id != id);
        Ok(())
    }
}
