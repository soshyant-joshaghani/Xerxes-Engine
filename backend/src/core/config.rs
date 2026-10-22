use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("invalid value for {0}: {1}")]
    Invalid(String, String),
    #[error("{0}")]
    Insecure(String),
}

/// Which series of modules a process serves (`SERVICES`): both together today, one each when
/// the multiplayer series moves to its own container (see `modules/mod.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Services {
    All,
    Api,
    Multiplayer,
}

impl Services {
    pub fn api(self) -> bool {
        matches!(self, Services::All | Services::Api)
    }

    pub fn multiplayer(self) -> bool {
        matches!(self, Services::All | Services::Multiplayer)
    }
}

impl FromStr for Services {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, ()> {
        match value.trim().to_ascii_lowercase().as_str() {
            "all" => Ok(Services::All),
            "api" => Ok(Services::Api),
            "multiplayer" => Ok(Services::Multiplayer),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub api_v1_str: String,
    pub secret_key: String,
    pub access_token_expire_minutes: i64,
    pub frontend_host: String,
    pub environment: String,
    pub services: Services,
    pub backend_cors_origins: Vec<String>,
    pub project_name: String,
    pub postgres_server: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_db: String,
    pub redis_host: String,
    pub redis_port: u16,
    pub redis_db: u32,
    pub redis_password: String,
    pub first_superuser: String,
    pub first_superuser_password: String,
    pub app_host: String,
    pub app_port: u16,
    pub bcrypt_cost: u32,
    pub migrations_dir: Option<String>,
    /// The Xerxes repo root (holds `games/` and `dist/`). The API runs from `backend/`, so `..`.
    pub xerxes_root: String,
}

fn num<T: FromStr>(value: Option<String>, key: &str, default: T) -> Result<T, ConfigError> {
    match value {
        None => Ok(default),
        Some(v) => match v.trim().parse::<T>() {
            Ok(n) => Ok(n),
            Err(_) => Err(ConfigError::Invalid(key.to_string(), v)),
        },
    }
}

fn parse_cors(raw: &str) -> Vec<String> {
    let raw = raw.trim();
    let items: Vec<String> = if raw.starts_with('[') {
        serde_json::from_str::<Vec<String>>(raw).unwrap_or_default()
    } else {
        raw.split(',').map(|s| s.to_string()).collect()
    };
    items
        .into_iter()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn percent_encode(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        let keep = b.is_ascii_alphanumeric() || b == b'-' || b == b'.' || b == b'_' || b == b'~';
        if keep {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

impl Settings {
    /// Build settings from a flat key/value map. Empty values count as unset.
    pub fn from_map(map: &HashMap<String, String>) -> Result<Settings, ConfigError> {
        let get = |key: &str| -> Option<String> { map.get(key).cloned().filter(|v| !v.is_empty()) };
        let text = |key: &str, default: &str| -> String {
            get(key).unwrap_or_else(|| default.to_string())
        };

        let environment = text("ENVIRONMENT", "local");
        if environment != "local" && environment != "staging" && environment != "production" {
            return Err(ConfigError::Invalid("ENVIRONMENT".to_string(), environment));
        }

        Ok(Settings {
            api_v1_str: text("API_V1_STR", "/api/v1"),
            secret_key: text("SECRET_KEY", "changethis"),
            access_token_expire_minutes: num(
                get("ACCESS_TOKEN_EXPIRE_MINUTES"),
                "ACCESS_TOKEN_EXPIRE_MINUTES",
                60 * 24 * 8,
            )?,
            frontend_host: text("FRONTEND_HOST", "http://dashboard.localhost"),
            environment,
            services: num(get("SERVICES"), "SERVICES", Services::All)?,
            backend_cors_origins: parse_cors(&text("BACKEND_CORS_ORIGINS", "")),
            project_name: text("PROJECT_NAME", "Xerxes"),
            postgres_server: text("POSTGRES_SERVER", "localhost"),
            postgres_port: num(get("POSTGRES_PORT"), "POSTGRES_PORT", 5432)?,
            postgres_user: text("POSTGRES_USER", "postgres"),
            postgres_password: text("POSTGRES_PASSWORD", ""),
            postgres_db: text("POSTGRES_DB", ""),
            redis_host: text("REDIS_HOST", "localhost"),
            redis_port: num(get("REDIS_PORT"), "REDIS_PORT", 6379)?,
            redis_db: num(get("REDIS_DB"), "REDIS_DB", 0)?,
            redis_password: text("REDIS_PASSWORD", ""),
            first_superuser: text("FIRST_SUPERUSER", "admin@example.com"),
            first_superuser_password: text("FIRST_SUPERUSER_PASSWORD", "changethis"),
            app_host: text("APP_HOST", "0.0.0.0"),
            app_port: num(get("APP_PORT"), "APP_PORT", 8000)?,
            bcrypt_cost: num(get("BCRYPT_COST"), "BCRYPT_COST", 12)?,
            migrations_dir: get("MIGRATIONS_DIR"),
            xerxes_root: text("XERXES_ROOT", ".."),
        })
    }

    /// Environment variables, then `.env` (looked up in `.`, `..`, `../..`).
    pub fn from_env() -> Result<Settings, ConfigError> {
        Settings::from_map(&load_env())
    }

    /// Settings for tests: local environment, cheap bcrypt.
    pub fn for_tests() -> Settings {
        let mut map = HashMap::new();
        map.insert("ENVIRONMENT".to_string(), "local".to_string());
        map.insert("SECRET_KEY".to_string(), "test-secret-key".to_string());
        map.insert(
            "FIRST_SUPERUSER".to_string(),
            "admin@example.com".to_string(),
        );
        map.insert(
            "FIRST_SUPERUSER_PASSWORD".to_string(),
            "adminpass123".to_string(),
        );
        map.insert("BCRYPT_COST".to_string(), "4".to_string());
        Settings::from_map(&map).expect("test settings are valid")
    }

    pub fn is_local(&self) -> bool {
        self.environment == "local"
    }

    /// Outside `local`, refuse the placeholder secrets.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if !self.is_local() {
            if self.secret_key == "changethis" {
                return Err(ConfigError::Insecure(
                    "SECRET_KEY must be set in production".to_string(),
                ));
            }
            if self.first_superuser_password == "changethis" {
                return Err(ConfigError::Insecure(
                    "FIRST_SUPERUSER_PASSWORD must be set in production".to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn all_cors_origins(&self) -> Vec<String> {
        let mut out = self.backend_cors_origins.clone();
        let host = self.frontend_host.trim().trim_end_matches('/').to_string();
        if !host.is_empty() && !out.contains(&host) {
            out.push(host);
        }
        out
    }

    pub fn redis_url(&self) -> String {
        let auth = if self.redis_password.is_empty() {
            String::new()
        } else {
            format!(":{}@", percent_encode(&self.redis_password))
        };
        format!(
            "redis://{}{}:{}/{}",
            auth, self.redis_host, self.redis_port, self.redis_db
        )
    }
}

/// Parse `KEY=VALUE` lines (comments, `export`, simple quotes).
pub fn parse_env_file(content: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let line = line.strip_prefix("export ").unwrap_or(line);
        let (key, raw) = match line.split_once('=') {
            Some(pair) => pair,
            None => continue,
        };
        let key = key.trim().to_string();
        if key.is_empty() {
            continue;
        }
        let raw = raw.trim();
        let quoted = raw.len() >= 2
            && ((raw.starts_with('"') && raw.ends_with('"'))
                || (raw.starts_with('\'') && raw.ends_with('\'')));
        let value = if quoted {
            raw[1..raw.len() - 1].to_string()
        } else {
            match raw.find(" #") {
                Some(i) => raw[..i].trim().to_string(),
                None => raw.to_string(),
            }
        };
        out.push((key, value));
    }
    out
}

/// `.env` file values overridden by real (non-empty) environment variables.
pub fn load_env() -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    for dir in [".", "..", "../.."] {
        let path = Path::new(dir).join(".env");
        if let Ok(content) = std::fs::read_to_string(&path) {
            for (k, v) in parse_env_file(&content) {
                map.insert(k, v);
            }
            break;
        }
    }
    for (k, v) in std::env::vars() {
        if !v.is_empty() {
            map.insert(k, v);
        }
    }
    map
}
