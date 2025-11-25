use crate::http::{moetran_get, poprako_post_opt};
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

    let list: Vec<ResProjectSet> = moetran_get(&path, None)
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

    let list: Vec<ResProject> = moetran_get(&path, None)
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

    let list: Vec<ResProject> = moetran_get(&path, None)
        .await
        .map_err(|err| format!("获取用户项目列表失败: {}", err))?;

    tracing::info!(count = list.len(), "user.projects.request.ok");

    Ok(list)
}

// PopRaKo 项目详情 DTO（精简版，仅保留仪表盘需要的字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoProjInfo {
    pub proj_id: String,
    pub proj_name: String,
    pub projset_index: u32,
    pub translating_status: i32,
    pub proofreading_status: i32,
    pub typesetting_status: i32,
    pub reviewing_status: i32,
    pub is_published: bool,
}

// PopRaKo 批量项目详情请求 DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoProjBatchReq {
    pub ids: Vec<String>,
}

// PopRaKo 通用返回包裹（项目模块）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoEnvelope<T> {
    pub code: u16,
    pub data: Option<T>,
    pub message: Option<String>,
}

// enriched 项目 DTO（Moetran + PopRaKo）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResProjectEnriched {
    pub id: String,
    pub name: String,
    pub source_count: u64,
    pub translated_source_count: u64,
    pub checked_source_count: u64,
    pub team: crate::team::ResTeam,
    pub project_set: ResProjectSet,

    pub has_poprako: bool,
    pub projset_index: Option<u32>,
    pub translating_status: Option<i32>,
    pub proofreading_status: Option<i32>,
    pub typesetting_status: Option<i32>,
    pub reviewing_status: Option<i32>,
    pub is_published: Option<bool>,
}

// 获取当前用户的 enriched 项目列表（Moetran 列表 + PopRaKo 批量补充）
#[tauri::command]
pub async fn get_user_projects_enriched(
    page: u32,
    limit: u32,
) -> Result<Vec<ResProjectEnriched>, String> {
    tracing::info!(
        page = page,
        limit = limit,
        "user.projects_enriched.request.start"
    );

    let path = format!("user/projects?page={page}&limit={limit}");

    let base_list: Vec<ResProject> = moetran_get(&path, None)
        .await
        .map_err(|err| format!("获取用户项目列表失败: {}", err))?;

    if base_list.is_empty() {
        tracing::info!("user.projects_enriched.empty");

        return Ok(vec![]);
    }

    let ids: Vec<String> = base_list.iter().map(|p| p.id.clone()).collect();

    let batch_body = PoprakoProjBatchReq { ids };

    let reply = poprako_post_opt::<PoprakoProjBatchReq, PoprakoEnvelope<Vec<PoprakoProjInfo>>>(
        "projs/batch",
        Some(batch_body),
    )
    .await
    .map_err(|err| format!("获取 PopRaKo 项目详情失败: {}", err))?;

    let mut map = std::collections::HashMap::new();

    if reply.code == 200 {
        if let Some(items) = reply.data {
            for item in items {
                map.insert(item.proj_id.clone(), item);
            }
        }
    } else {
        let msg = reply
            .message
            .unwrap_or_else(|| "PopRaKo 批量项目详情失败".to_string());

        tracing::info!(message = %msg, code = reply.code, "poprako.projs.batch.failed");
    }

    let mut enriched_list = Vec::with_capacity(base_list.len());

    for item in base_list {
        if let Some(extra) = map.get(&item.id) {
            enriched_list.push(ResProjectEnriched {
                id: item.id,
                name: item.name,
                source_count: item.source_count,
                translated_source_count: item.translated_source_count,
                checked_source_count: item.checked_source_count,
                team: item.team.clone(),
                project_set: item.project_set.clone(),
                has_poprako: true,
                projset_index: Some(extra.projset_index),
                translating_status: Some(extra.translating_status),
                proofreading_status: Some(extra.proofreading_status),
                typesetting_status: Some(extra.typesetting_status),
                reviewing_status: Some(extra.reviewing_status),
                is_published: Some(extra.is_published),
            });
        } else {
            enriched_list.push(ResProjectEnriched {
                id: item.id,
                name: item.name,
                source_count: item.source_count,
                translated_source_count: item.translated_source_count,
                checked_source_count: item.checked_source_count,
                team: item.team.clone(),
                project_set: item.project_set.clone(),
                has_poprako: false,
                projset_index: None,
                translating_status: None,
                proofreading_status: None,
                typesetting_status: None,
                reviewing_status: None,
                is_published: None,
            });
        }
    }

    tracing::info!(
        count = enriched_list.len(),
        "user.projects_enriched.request.ok"
    );

    Ok(enriched_list)
}
