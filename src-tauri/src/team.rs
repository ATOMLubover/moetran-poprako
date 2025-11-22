use crate::http::moetran_get;
use serde::{Deserialize, Serialize};

// 汉化组 DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResTeam {
    pub id: String,
    pub avatar: String,
    pub has_avatar: bool,
    pub name: String,
}

// 获取当前用户的汉化组列表
#[tauri::command]
pub async fn get_user_teams(page: u32, limit: u32) -> Result<Vec<ResTeam>, String> {
    tracing::info!(page = page, limit = limit, "user.teams.request.start");

    let path = format!("user/teams?page={page}&limit={limit}");

    let list = moetran_get::<Vec<ResTeam>>(&path)
        .await
        .map_err(|err| format!("获取用户汉化组失败: {}", err))?;

    tracing::info!(count = list.len(), "user.teams.request.ok");
    Ok(list)
}
