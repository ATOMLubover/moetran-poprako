// 图片缓存元数据存储（SQLite）
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedProjectMetadata {
    pub project_id: String,
    pub project_name: String,
    pub status: String, // "completed" | "failed"
    pub file_count: i64,
    pub total_size_bytes: i64,
    pub cached_at: i64, // Unix timestamp
}

// 创建缓存元数据表
pub async fn migrate_cache_metadata_table(pool: &SqlitePool) -> Result<(), String> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS cached_projects (
            project_id TEXT PRIMARY KEY,
            project_name TEXT NOT NULL,
            status TEXT NOT NULL,
            file_count INTEGER NOT NULL DEFAULT 0,
            total_size_bytes INTEGER NOT NULL DEFAULT 0,
            cached_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await
    .map_err(|err| format!("Failed to create cached_projects table: {}", err))?;

    Ok(())
}

// 插入或更新缓存元数据
pub async fn upsert_cached_project(
    pool: &SqlitePool,
    metadata: &CachedProjectMetadata,
) -> Result<(), String> {
    sqlx::query(
        r#"
        INSERT INTO cached_projects (project_id, project_name, status, file_count, total_size_bytes, cached_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(project_id) DO UPDATE SET
            project_name = excluded.project_name,
            status = excluded.status,
            file_count = excluded.file_count,
            total_size_bytes = excluded.total_size_bytes,
            cached_at = excluded.cached_at
        "#,
    )
    .bind(&metadata.project_id)
    .bind(&metadata.project_name)
    .bind(&metadata.status)
    .bind(metadata.file_count)
    .bind(metadata.total_size_bytes)
    .bind(metadata.cached_at)
    .execute(pool)
    .await
    .map_err(|err| format!("Failed to upsert cached project: {}", err))?;

    Ok(())
}

// 获取所有缓存项目列表
pub async fn get_all_cached_projects(
    pool: &SqlitePool,
) -> Result<Vec<CachedProjectMetadata>, String> {
    let rows = sqlx::query_as::<_, (String, String, String, i64, i64, i64)>(
        r#"
        SELECT project_id, project_name, status, file_count, total_size_bytes, cached_at
        FROM cached_projects
        ORDER BY cached_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(|err| format!("Failed to fetch cached projects: {}", err))?;

    Ok(rows
        .into_iter()
        .map(
            |(project_id, project_name, status, file_count, total_size_bytes, cached_at)| {
                CachedProjectMetadata {
                    project_id,
                    project_name,
                    status,
                    file_count,
                    total_size_bytes,
                    cached_at,
                }
            },
        )
        .collect())
}

// 删除缓存元数据
pub async fn delete_cached_project_metadata(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<(), String> {
    sqlx::query("DELETE FROM cached_projects WHERE project_id = ?")
        .bind(project_id)
        .execute(pool)
        .await
        .map_err(|err| format!("Failed to delete cached project metadata: {}", err))?;

    Ok(())
}

// 获取单个缓存项目元数据
pub async fn get_cached_project_metadata(
    pool: &SqlitePool,
    project_id: &str,
) -> Result<Option<CachedProjectMetadata>, String> {
    let row = sqlx::query_as::<_, (String, String, String, i64, i64, i64)>(
        r#"
        SELECT project_id, project_name, status, file_count, total_size_bytes, cached_at
        FROM cached_projects
        WHERE project_id = ?
        "#,
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|err| format!("Failed to fetch cached project metadata: {}", err))?;

    Ok(row.map(
        |(project_id, project_name, status, file_count, total_size_bytes, cached_at)| {
            CachedProjectMetadata {
                project_id,
                project_name,
                status,
                file_count,
                total_size_bytes,
                cached_at,
            }
        },
    ))
}
