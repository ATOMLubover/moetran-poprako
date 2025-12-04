use std::fs;
use std::path::Path;
use std::sync::OnceLock;

pub mod cache_metadata;
pub mod token;

pub struct LocalStorage {
    pool: sqlx::SqlitePool,
}

impl LocalStorage {
    /// 初始化： 创建 db 文件, 建立连接池, 执行迁移
    pub async fn init(database_path: &str) -> Result<(), String> {
        let path = Path::new(database_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("Failed to create db directory: {}", err))?;
        }

        if !path.exists() {
            fs::File::create(path).map_err(|err| format!("Failed to create db file: {}", err))?;
        }

        let database_url = format!("sqlite://{}", path.to_string_lossy());

        let pool = sqlx::SqlitePool::connect(&database_url)
            .await
            .map_err(|err| format!("Failed to connect to database: {}", err))?;

        token::migrate_token_table(&pool).await?;
        cache_metadata::migrate_cache_metadata_table(&pool).await?;

        LOCAL_STORAGE
            .set(Self { pool })
            .map_err(|_| "LOCAL_STORAGE is already set".to_string())
    }

    pub fn pool(&self) -> &sqlx::SqlitePool {
        &self.pool
    }
}

pub static LOCAL_STORAGE: OnceLock<LocalStorage> = OnceLock::new();
