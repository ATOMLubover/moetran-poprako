use crate::http::{moetran_get, poprako_post_opt};
use serde::{Deserialize, Serialize};

// Poprako 登录请求 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct ReqLogin {
    pub username: String,
    pub password: String,
}

// Poprako 登录响应 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct ResLogin {
    pub user_id: u64,
    pub token: String,
}

// 执行 Poprako 登录
#[tauri::command]
pub async fn login_poprako(payload: ReqLogin) -> Result<ResLogin, String> {
    tracing::info!(username = %payload.username, "poprako.login.request.start");

    let body = poprako_post_opt::<ReqLogin, ResLogin>("auth/login", Some(payload))
        .await
        .map_err(|err| format!("Poprako 登录请求失败: {}", err))?;

    tracing::info!(user_id = body.user_id, "poprako.login.request.ok");
    Ok(body)
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

    let body = moetran_get::<ResUser>("user/info")
        .await
        .map_err(|err| format!("获取用户信息失败: {}", err))?;

    tracing::info!("user.info.request.ok");
    Ok(body)
}
