use std::sync::Arc;

use uuid::Uuid;

use crate::core::error::{ApiError, ApiResult};
use crate::core::security;
use crate::core::state::AppState;
use crate::modules::api::base::users::repository::UserRepository;
use crate::modules::api::base::users::schemas::{
    User, UserCreate, UserPublic, UserUpdate, UsersPublic,
};

const DUPLICATE_EMAIL: &str = "The user with this email already exists in the system.";

#[derive(Clone)]
pub struct UserService {
    users: Arc<dyn UserRepository>,
    bcrypt_cost: u32,
}

impl UserService {
    pub fn new(state: &AppState) -> Self {
        UserService {
            users: state.users.clone(),
            bcrypt_cost: state.config.bcrypt_cost,
        }
    }

    async fn hash(&self, password: &str) -> ApiResult<String> {
        let password = password.to_string();
        let cost = self.bcrypt_cost;
        let joined =
            tokio::task::spawn_blocking(move || security::hash_password(&password, cost)).await;
        match joined {
            Ok(Ok(hashed)) => Ok(hashed),
            Ok(Err(err)) => Err(ApiError::internal(err)),
            Err(err) => Err(ApiError::internal(err)),
        }
    }

    async fn verify(&self, password: &str, hashed: &str) -> ApiResult<bool> {
        let password = password.to_string();
        let hashed = hashed.to_string();
        let joined =
            tokio::task::spawn_blocking(move || security::verify_password(&password, &hashed))
                .await;
        match joined {
            Ok(ok) => Ok(ok),
            Err(err) => Err(ApiError::internal(err)),
        }
    }

    /// Login check. Wrong email/password and inactive users are `400`.
    pub async fn authenticate(&self, email: &str, password: &str) -> ApiResult<User> {
        let user = match self.users.get_by_email(email).await? {
            Some(user) => user,
            None => return Err(ApiError::bad_request("Incorrect email or password")),
        };
        if !self.verify(password, &user.hashed_password).await? {
            return Err(ApiError::bad_request("Incorrect email or password"));
        }
        if !user.is_active {
            return Err(ApiError::bad_request("Inactive user"));
        }
        Ok(user)
    }

    pub async fn list(&self, skip: i64, limit: i64) -> ApiResult<UsersPublic> {
        let count = self.users.count().await?;
        let rows = self.users.list(skip, limit).await?;
        Ok(UsersPublic {
            data: rows.iter().map(UserPublic::from).collect(),
            count,
        })
    }

    pub async fn create(&self, input: UserCreate) -> ApiResult<User> {
        input.validate()?;
        if self.users.get_by_email(&input.email).await?.is_some() {
            return Err(ApiError::bad_request(DUPLICATE_EMAIL));
        }
        let hashed = self.hash(&input.password).await?;
        let user = User {
            id: Uuid::new_v4(),
            email: input.email,
            is_active: input.is_active,
            is_superuser: input.is_superuser,
            full_name: input.full_name,
            hashed_password: hashed,
        };
        self.users.create(user).await
    }

    /// Self can read itself; anyone else requires a superuser.
    pub async fn get_for(&self, current: &User, id: Uuid) -> ApiResult<User> {
        let found = self.users.get_by_id(id).await?;
        if let Some(user) = &found {
            if user.id == current.id {
                return Ok(user.clone());
            }
        }
        if !current.is_superuser {
            return Err(ApiError::forbidden(
                "The user doesn't have enough privileges",
            ));
        }
        match found {
            Some(user) => Ok(user),
            None => Err(ApiError::not_found("User not found")),
        }
    }

    pub async fn update(&self, id: Uuid, input: UserUpdate) -> ApiResult<User> {
        input.validate()?;
        let mut user = match self.users.get_by_id(id).await? {
            Some(user) => user,
            None => {
                return Err(ApiError::not_found(
                    "The user with this id does not exist in the system",
                ))
            }
        };
        if let Some(email) = &input.email {
            if let Some(existing) = self.users.get_by_email(email).await? {
                if existing.id != id {
                    return Err(ApiError::conflict("User with this email already exists"));
                }
            }
        }
        if let Some(email) = input.email {
            user.email = email;
        }
        if let Some(password) = &input.password {
            user.hashed_password = self.hash(password).await?;
        }
        if let Some(full_name) = input.full_name {
            user.full_name = full_name;
        }
        if let Some(is_active) = input.is_active {
            user.is_active = is_active;
        }
        if let Some(is_superuser) = input.is_superuser {
            user.is_superuser = is_superuser;
        }
        self.users.update(user).await
    }

    /// Deletes the user. Superusers cannot delete themselves.
    pub async fn delete(&self, current: &User, id: Uuid) -> ApiResult<()> {
        let user = match self.users.get_by_id(id).await? {
            Some(user) => user,
            None => return Err(ApiError::not_found("User not found")),
        };
        if user.id == current.id {
            return Err(ApiError::forbidden(
                "Super users are not allowed to delete themselves",
            ));
        }
        self.users.delete(id).await
    }

    /// Create the configured first superuser when it does not exist yet.
    pub async fn ensure_first_superuser(&self, email: &str, password: &str) -> ApiResult<()> {
        if self.users.get_by_email(email).await?.is_some() {
            return Ok(());
        }
        let hashed = self.hash(password).await?;
        let user = User {
            id: Uuid::new_v4(),
            email: email.to_string(),
            is_active: true,
            is_superuser: true,
            full_name: None,
            hashed_password: hashed,
        };
        self.users.create(user).await?;
        tracing::info!("created first superuser {}", email);
        Ok(())
    }
}
