// 图片缓存管理模块
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

use crate::http::moetran_get_raw;
use crate::storage::cache_metadata::{
    delete_cached_project_metadata, get_all_cached_projects, get_cached_project_metadata,
    upsert_cached_project, CachedProjectMetadata,
};
use crate::storage::LOCAL_STORAGE;
use crate::DATA_DIR;

const MAX_RETRIES: usize = 2;
const CONCURRENT_DOWNLOADS: usize = 5;

/// 检查项目的图片缓存是否存在
#[tauri::command]
#[tracing::instrument]
pub async fn check_file_cache(project_id: String) -> Result<bool, String> {
    tracing::info!("image_cache.check_file_cache.start");

    let cache_dir = get_cache_dir(&project_id);

    let exists = cache_dir.exists();

    tracing::info!(exists = exists, "image_cache.check_file_cache.ok");

    Ok(exists)
}

/// 下载整个项目的所有图片到本地缓存
#[tauri::command]
#[tracing::instrument]
pub async fn download_project_files(
    project_id: String,
    project_name: String,
    files: Vec<FileDownloadInfo>,
) -> Result<(), String> {
    tracing::info!(
        file_count = files.len(),
        "image_cache.download_project_files.start"
    );

    let cache_dir = get_cache_dir(&project_id);

    // 创建缓存目录
    fs::create_dir_all(&cache_dir)
        .await
        .map_err(|e| format!("创建缓存目录失败: {}", e))?;

    // 检查已存在的文件，跳过下载
    let mut files_to_download = Vec::new();
    for (index, file) in files.iter().enumerate() {
        let file_path = cache_dir.join(format!("{}.{}", index, get_extension(&file.url)));
        if !file_path.exists() {
            files_to_download.push((index, file));
        } else {
            tracing::debug!(index = index, "file already cached, skip");
        }
    }

    tracing::info!(
        total = files.len(),
        to_download = files_to_download.len(),
        "image_cache.download_project_files.files_checked"
    );

    let mut download_failed = false;

    if !files_to_download.is_empty() {
        // 使用 semaphore 控制并发度
        let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(CONCURRENT_DOWNLOADS));
        let mut tasks = Vec::new();

        for (index, file) in files_to_download {
            let sem = semaphore.clone();
            let url = file.url.clone();
            let cache_dir = cache_dir.clone();

            let task = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();

                download_file_with_retry(&url, &cache_dir, index).await
            });

            tasks.push(task);
        }

        // 等待所有下载任务完成
        for task in tasks {
            match task.await {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    tracing::error!(error = %e, "download task failed");
                    download_failed = true;
                }
                Err(e) => {
                    tracing::error!(error = %e, "task join failed");
                    download_failed = true;
                }
            }
        }
    }

    // 计算缓存文件大小
    let mut total_size_bytes = 0i64;
    let mut file_count = 0i64;
    for i in 0..files.len() {
        let file_path = cache_dir.join(format!("{}.{}", i, get_extension(&files[i].url)));
        if file_path.exists() {
            if let Ok(metadata) = fs::metadata(&file_path).await {
                total_size_bytes += metadata.len() as i64;
                file_count += 1;
            }
        }
    }

    // 写入元数据到 SQLite
    let status = if download_failed {
        "failed"
    } else {
        "completed"
    };
    let cached_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let metadata = CachedProjectMetadata {
        project_id: project_id.clone(),
        project_name,
        status: status.to_string(),
        file_count,
        total_size_bytes,
        cached_at,
    };

    if let Some(storage) = LOCAL_STORAGE.get() {
        upsert_cached_project(storage.pool(), &metadata).await?;
    } else {
        tracing::warn!("LOCAL_STORAGE not initialized, skip metadata save");
    }

    tracing::info!(
        status = status,
        file_count = file_count,
        total_size_bytes = total_size_bytes,
        "image_cache.download_project_files.ok"
    );

    if download_failed {
        return Err("部分文件下载失败".to_string());
    }

    Ok(())
}

/// 删除项目的图片缓存
#[tauri::command]
#[tracing::instrument]
pub async fn delete_file_cache(project_id: String) -> Result<(), String> {
    tracing::info!("image_cache.delete_file_cache.start");

    let cache_dir = get_cache_dir(&project_id);

    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)
            .await
            .map_err(|e| format!("删除缓存目录失败: {}", e))?;
    }

    // 删除元数据
    if let Some(storage) = LOCAL_STORAGE.get() {
        delete_cached_project_metadata(storage.pool(), &project_id).await?;
    } else {
        tracing::warn!("LOCAL_STORAGE not initialized, skip metadata delete");
    }

    tracing::info!("image_cache.delete_file_cache.ok");

    Ok(())
}

/// 获取所有缓存项目列表
#[tauri::command]
#[tracing::instrument]
pub async fn get_all_cached_projects_list() -> Result<Vec<CachedProjectMetadata>, String> {
    tracing::info!("image_cache.get_all_cached_projects_list.start");

    if let Some(storage) = LOCAL_STORAGE.get() {
        let projects = get_all_cached_projects(storage.pool()).await?;

        tracing::info!(
            count = projects.len(),
            "image_cache.get_all_cached_projects_list.ok"
        );

        Ok(projects)
    } else {
        Err("LOCAL_STORAGE not initialized".to_string())
    }
}

/// 获取单个项目的缓存元数据
#[tauri::command]
#[tracing::instrument]
pub async fn get_cached_project_info(
    project_id: String,
) -> Result<Option<CachedProjectMetadata>, String> {
    tracing::debug!("image_cache.get_cached_project_info.start");

    if let Some(storage) = LOCAL_STORAGE.get() {
        let metadata = get_cached_project_metadata(storage.pool(), &project_id).await?;

        tracing::debug!(
            found = metadata.is_some(),
            "image_cache.get_cached_project_info.ok"
        );

        Ok(metadata)
    } else {
        Err("LOCAL_STORAGE not initialized".to_string())
    }
}

/// 从本地缓存读取图片（base64 编码）
#[tauri::command]
#[tracing::instrument]
pub async fn load_cached_file(
    project_id: String,
    file_index: usize,
) -> Result<CachedFileData, String> {
    tracing::debug!("image_cache.load_cached_file.start");

    let cache_dir = get_cache_dir(&project_id);

    // 检查缓存目录是否存在
    if !cache_dir.exists() {
        return Err(format!("缓存目录不存在: {}", cache_dir.display()));
    }

    // 查找对应索引的文件（不确定扩展名）
    let entries = fs::read_dir(&cache_dir)
        .await
        .map_err(|e| format!("读取缓存目录失败: {}", e))?;

    let mut entries = entries;
    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(|e| format!("遍历缓存目录失败: {}", e))?
    {
        let file_name = entry.file_name();
        let file_name_str = file_name.to_string_lossy();

        // 检查文件名是否匹配索引（格式：{index}.{ext}）
        if let Some(dot_pos) = file_name_str.rfind('.') {
            let name_part = &file_name_str[..dot_pos];
            if name_part == file_index.to_string() {
                let file_path = entry.path();
                let ext = &file_name_str[dot_pos + 1..];
                let content_type = get_content_type(ext);

                let data = fs::read(&file_path)
                    .await
                    .map_err(|e| format!("读取缓存文件失败: {}", e))?;

                let b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &data);

                tracing::debug!("image_cache.load_cached_file.ok");

                return Ok(CachedFileData { b64, content_type });
            }
        }
    }

    Err(format!("缓存文件不存在: index {}", file_index))
}

// ========== 内部辅助函数 ==========

#[derive(Debug, serde::Deserialize)]
pub struct FileDownloadInfo {
    pub url: String,
}

#[derive(serde::Serialize)]
pub struct CachedFileData {
    pub b64: String,
    pub content_type: String,
}

fn get_cache_dir(project_id: &str) -> PathBuf {
    let mut path = DATA_DIR.clone();
    path.push("images");
    path.push(project_id);
    path
}

fn get_extension(url: &str) -> &str {
    if url.ends_with(".png") || url.contains(".png?") {
        "png"
    } else if url.ends_with(".jpg") || url.contains(".jpg?") {
        "jpg"
    } else if url.ends_with(".jpeg") || url.contains(".jpeg?") {
        "jpeg"
    } else if url.ends_with(".webp") || url.contains(".webp?") {
        "webp"
    } else {
        "jpg" // 默认
    }
}

fn get_content_type(ext: &str) -> String {
    match ext {
        "png" => "image/png".to_string(),
        "jpg" | "jpeg" => "image/jpeg".to_string(),
        "webp" => "image/webp".to_string(),
        _ => "image/jpeg".to_string(),
    }
}

async fn download_file_with_retry(url: &str, cache_dir: &Path, index: usize) -> Result<(), String> {
    let ext = get_extension(url);
    let file_path = cache_dir.join(format!("{}.{}", index, ext));

    for attempt in 0..=MAX_RETRIES {
        match download_file(url, &file_path).await {
            Ok(_) => {
                tracing::debug!(index = index, "file downloaded successfully");
                return Ok(());
            }
            Err(e) => {
                if attempt < MAX_RETRIES {
                    tracing::warn!(
                        index = index,
                        attempt = attempt + 1,
                        error = %e,
                        "download failed, retrying"
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                } else {
                    tracing::error!(
                        index = index,
                        error = %e,
                        "download failed after all retries"
                    );
                    return Err(format!("下载文件 {} 失败（索引 {}）: {}", url, index, e));
                }
            }
        }
    }

    unreachable!()
}

async fn download_file(url: &str, file_path: &Path) -> Result<(), String> {
    // 使用 moetran_get_raw 下载图片二进制数据
    let data = moetran_get_raw(url)
        .await
        .map_err(|e| format!("HTTP 请求失败: {}", e))?;

    // 写入文件
    let mut file = fs::File::create(file_path)
        .await
        .map_err(|e| format!("创建文件失败: {}", e))?;

    file.write_all(&data)
        .await
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(())
}
