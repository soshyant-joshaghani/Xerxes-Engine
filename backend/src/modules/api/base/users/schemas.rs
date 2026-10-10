use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use crate::core::error::{check_email, check_len, ApiResult};

/// Database row for table `"user"`.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub is_active: bool,
    pub is_superuser: bool,
    pub full_name: Option<String>,
    pub hashed_password: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub is_active: bool,
    pub is_superuser: bool,
    pub full_name: Option<String>,
}

impl From<&User> for UserPublic {
    fn from(user: &User) -> Self {
        UserPublic {
            id: user.id,
            email: user.email.clone(),
            is_active: user.is_active,
            is_superuser: user.is_superuser,
            full_name: user.full_name.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct UsersPublic {
    pub data: Vec<UserPublic>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct Message {
    pub message: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, schemars::JsonSchema)]
pub struct UserCreate {
    pub email: String,
    pub password: String,
    #[serde(default = "default_true")]
    pub is_active: bool,
    #[serde(default)]
    pub is_superuser: bool,
    #[serde(default)]
    pub full_name: Option<String>,
}

impl UserCreate {
    pub fn validate(&self) -> ApiResult<()> {
        check_email("email", &self.email)?;
        check_len("password", &self.password, 8, 128)?;
        if let Some(name) = &self.full_name {
            check_len("full_name", name, 0, 255)?;
        }
        Ok(())
    }
}

/// `null` and "absent" differ for `full_name`: absent leaves it alone, `null` clears it.
fn double_option<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Default, Deserialize, schemars::JsonSchema)]
pub struct UserUpdate {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default, deserialize_with = "double_option")]
    pub full_name: Option<Option<String>>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub is_superuser: Option<bool>,
}

impl UserUpdate {
    pub fn validate(&self) -> ApiResult<()> {
        if let Some(email) = &self.email {
            check_email("email", email)?;
        }
        if let Some(password) = &self.password {
            check_len("password", password, 8, 128)?;
        }
        if let Some(Some(name)) = &self.full_name {
            check_len("full_name", name, 0, 255)?;
        }
        Ok(())
    }
}
