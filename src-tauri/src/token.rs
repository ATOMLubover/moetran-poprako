use std::sync::RwLock;

use crate::storage::{token as storage_token, LOCAL_STORAGE};

static MOETRAN_TOKEN: RwLock<Option<String>> = RwLock::new(None);

static POPRAKO_TOKEN: RwLock<Option<String>> = RwLock::new(None);

// 获取 Moetran token（从内存或数据库）
#[tauri::command]
pub async fn get_moetran_token() -> Result<Option<String>, String> {
    // 先检查内存缓存
    {
        let guard = MOETRAN_TOKEN
            .read()
            .map_err(|err| format!("Failed to read MOETRAN_TOKEN: {}", err))?;
        if guard.is_some() {
            return Ok(guard.clone());
        }
    }

    // 内存中没有，尝试从数据库加载
    let storage = LOCAL_STORAGE
        .get()
        .ok_or("LOCAL_STORAGE not initialized".to_string())?;

    match storage_token::get_moetran_token(storage.pool()).await {
        Ok(token) => {
            // 加载成功后更新内存缓存
            let mut guard = MOETRAN_TOKEN
                .write()
                .map_err(|err| format!("Failed to write MOETRAN_TOKEN: {}", err))?;
            *guard = Some(token.clone());
            Ok(Some(token))
        }
        Err(_) => Ok(None), // 数据库中也没有
    }
}

// 保存 Moetran token（到内存和数据库）
#[tauri::command]
pub async fn save_moetran_token(token: String) -> Result<(), String> {
    let storage = LOCAL_STORAGE
        .get()
        .ok_or("LOCAL_STORAGE not initialized".to_string())?;

    // 保存到数据库
    storage_token::save_moetran_token(storage.pool(), &token).await?;

    // 更新内存缓存
    let mut guard = MOETRAN_TOKEN
        .write()
        .map_err(|err| format!("Failed to write MOETRAN_TOKEN: {}", err))?;
    *guard = Some(token);

    Ok(())
}

// 删除 Moetran token（从内存和数据库）
#[tauri::command]
pub async fn remove_moetran_token() -> Result<(), String> {
    let storage = LOCAL_STORAGE
        .get()
        .ok_or("LOCAL_STORAGE not initialized".to_string())?;

    // 从数据库删除
    storage_token::remove_moetran_token(storage.pool()).await?;

    // 清空内存缓存
    let mut guard = MOETRAN_TOKEN
        .write()
        .map_err(|err| format!("Failed to write MOETRAN_TOKEN: {}", err))?;
    *guard = None;

    Ok(())
}

// 获取 Poprako token（从内存或数据库）
#[tauri::command]
pub async fn get_poprako_token() -> Result<Option<String>, String> {
    // 先检查内存缓存
    {
        let guard = POPRAKO_TOKEN
            .read()
            .map_err(|err| format!("Failed to read POPRAKO_TOKEN: {}", err))?;
        if guard.is_some() {
            return Ok(guard.clone());
        }
    }

    // 内存中没有，尝试从数据库加载
    let storage = LOCAL_STORAGE
        .get()
        .ok_or("LOCAL_STORAGE not initialized".to_string())?;

    match storage_token::get_poprako_token(storage.pool()).await {
        Ok(token) => {
            // 加载成功后更新内存缓存
            let mut guard = POPRAKO_TOKEN
                .write()
                .map_err(|err| format!("Failed to write POPRAKO_TOKEN: {}", err))?;
            *guard = Some(token.clone());
            Ok(Some(token))
        }
        Err(_) => Ok(None), // 数据库中也没有
    }
}

// 保存 Poprako token（到内存和数据库）
#[tauri::command]
pub async fn save_poprako_token(token: String) -> Result<(), String> {
    let storage = LOCAL_STORAGE
        .get()
        .ok_or("LOCAL_STORAGE not initialized".to_string())?;

    // 保存到数据库
    storage_token::save_poprako_token(storage.pool(), &token).await?;

    // 更新内存缓存
    let mut guard = POPRAKO_TOKEN
        .write()
        .map_err(|err| format!("Failed to write POPRAKO_TOKEN: {}", err))?;
    *guard = Some(token);

    Ok(())
}

// 删除 Poprako token（从内存和数据库）
#[tauri::command]
pub async fn remove_poprako_token() -> Result<(), String> {
    let storage = LOCAL_STORAGE
        .get()
        .ok_or("LOCAL_STORAGE not initialized".to_string())?;

    // 从数据库删除
    storage_token::remove_poprako_token(storage.pool()).await?;

    // 清空内存缓存
    let mut guard = POPRAKO_TOKEN
        .write()
        .map_err(|err| format!("Failed to write POPRAKO_TOKEN: {}", err))?;
    *guard = None;

    Ok(())
}

pub(crate) fn cached_moetran_token() -> Option<String> {
    MOETRAN_TOKEN.read().ok().and_then(|guard| guard.clone())
}

pub(crate) fn cached_poprako_token() -> Option<String> {
    POPRAKO_TOKEN.read().ok().and_then(|guard| guard.clone())
}
