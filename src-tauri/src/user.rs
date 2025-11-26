use crate::{
    defer::WarnDefer,
    http::{moetran_get, poprako_post_opt},
};
use serde::{Deserialize, Serialize};

// PopRaKo 同步用户请求 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct ReqSync {
    pub user_id: String,
    pub username: String,
    pub email: String,
}

// PopRaKo 通用返回包裹
#[derive(Debug, Serialize, Deserialize)]
pub struct PoprakoEnvelope<T> {
    pub code: u16,
    pub data: Option<T>,
    pub message: Option<String>,
}

// PopRaKo 同步用户响应 DTO（仅关心 token）
#[derive(Debug, Serialize, Deserialize)]
pub struct ResSync {
    pub token: String,
}

// 执行 PopRaKo 用户同步（包含登录）
#[tauri::command]
pub async fn sync_user(payload: ReqSync) -> Result<ResSync, String> {
    tracing::info!(username = %payload.username, "poprako.sync.request.start");

    let mut defer = WarnDefer::new("poprako.sync.request");

    let reply: PoprakoEnvelope<ResSync> = poprako_post_opt("user/sync", Some(payload))
        .await
        .map_err(|err| format!("Failed to sync user to Poprako: {}", err))?;

    if reply.code != 200 && reply.code != 201 {
        let msg = reply
            .message
            .unwrap_or_else(|| "Poprako sync failed".to_string());

        return Err(msg);
    }

    let data = reply
        .data
        .ok_or_else(|| "Poprako sync response missing data".to_string())?;

    tracing::info!("poprako.sync.request.ok");

    defer.success();

    Ok(data)
}

// 用户信息 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct ResUser {
    pub id: String,
    pub name: String,
    pub has_avatar: bool,
    pub avatar: String,
}

// 获取当前用户信息
#[tauri::command]
pub async fn get_user_info() -> Result<ResUser, String> {
    tracing::info!("user.info.request.start");

    let mut defer = WarnDefer::new("user.info.request");

    let body: ResUser = moetran_get("user/info", None)
        .await
        .map_err(|err| format!("Failed to get user info: {}", err))?;

    tracing::info!("user.info.request.ok");

    defer.success();

    Ok(body)
}
