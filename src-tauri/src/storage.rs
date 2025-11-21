use std::cell::OnceCell;
use std::fs;
use std::path::Path;
use std::sync::{LazyLock, OnceLock};

mod token;

pub struct LocalStorage {
    pool: sqlx::SqlitePool,
}

impl LocalStorage {
    /// Initialize LocalStorage: ensure directory & file exist, open pool, run migrations.
    pub async fn init(database_path: &str) -> Result<(), String> {
        let path = Path::new(database_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Failed to create db directory: {}", err))?;
        }

        if !path.exists() {
            // create empty file so that relative path mistakes are surfaced early.
            fs::File::create(path).map_err(|err| format!("Failed to create db file: {}", err))?;
        }

        let database_url = format!("sqlite://{}", path.to_string_lossy());

        let pool = sqlx::SqlitePool::connect(&database_url)
            .await
            .map_err(|err| format!("Failed to connect to database: {}", err))?;

        token::migrate_token_table(&pool).await?;

        LOCAL_STORAGE
            .set(Self { pool })
            .map_err(|_| "LOCAL_STORAGE is already set".to_string())
    }

    pub fn pool(&self) -> &sqlx::SqlitePool {
        &self.pool
    }
}

pub static LOCAL_STORAGE: OnceLock<LocalStorage> = OnceLock::new();
