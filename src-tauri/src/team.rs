use crate::{defer::WarnDefer, http::moetran_get};
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
#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserTeamsReq {
    pub page: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn get_user_teams(payload: GetUserTeamsReq) -> Result<Vec<ResTeam>, String> {
    tracing::info!(page = payload.page, limit = payload.limit, "user.teams.request.start");

    let mut defer = WarnDefer::new("user.teams.request");

    let path = format!("user/teams?page={}&limit={}", payload.page, payload.limit);

    let list: Vec<ResTeam> = moetran_get(&path, None)
        .await
        .map_err(|err| format!("获取用户汉化组失败: {}", err))?;

    tracing::info!(count = list.len(), "user.teams.request.ok");

    defer.success();

    Ok(list)
}
