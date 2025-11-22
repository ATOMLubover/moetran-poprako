use crate::http::moetran_get;
use serde::{Deserialize, Serialize};

// 项目集 DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResProjectSet {
    pub id: String,
    pub name: String,
}

// 获取指定汉化组的项目集列表
#[tauri::command]
pub async fn get_team_project_sets(
    team_id: String,
    page: u32,
    limit: u32,
) -> Result<Vec<ResProjectSet>, String> {
    tracing::info!(team_id = %team_id, page = page, limit = limit, "team.project_sets.request.start");

    let path = format!("teams/{team_id}/project-sets?page={page}&limit={limit}");

    let list = moetran_get::<Vec<ResProjectSet>>(&path)
        .await
        .map_err(|err| format!("获取项目集列表失败: {}", err))?;

    tracing::info!(count = list.len(), "team.project_sets.request.ok");
    Ok(list)
}

// 项目 DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResProject {
    pub id: String,
    pub name: String,
    pub source_count: u64,
    pub translated_source_count: u64,
    pub checked_source_count: u64,
    pub team: crate::team::ResTeam,
    pub project_set: ResProjectSet,
}

// 获取指定汉化组下某项目集的项目列表
#[tauri::command]
pub async fn get_team_projects(
    team_id: String,
    project_set: String,
    page: u32,
    limit: u32,
) -> Result<Vec<ResProject>, String> {
    tracing::info!(team_id = %team_id, project_set = %project_set, page = page, limit = limit, "team.projects.request.start");

    let path =
        format!("teams/{team_id}/projects?project_set={project_set}&page={page}&limit={limit}");

    let list = moetran_get::<Vec<ResProject>>(&path)
        .await
        .map_err(|err| format!("获取项目列表失败: {}", err))?;

    tracing::info!(count = list.len(), "team.projects.request.ok");
    Ok(list)
}

// 获取当前用户的项目列表
#[tauri::command]
pub async fn get_user_projects(page: u32, limit: u32) -> Result<Vec<ResProject>, String> {
    tracing::info!(page = page, limit = limit, "user.projects.request.start");

    let path = format!("user/projects?page={page}&limit={limit}");

    let list = moetran_get::<Vec<ResProject>>(&path)
        .await
        .map_err(|err| format!("获取用户项目列表失败: {}", err))?;

    tracing::info!(count = list.len(), "user.projects.request.ok");
    Ok(list)
}
