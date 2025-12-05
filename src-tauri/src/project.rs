use crate::{
    defer::WarnDefer,
    http::{
        moetran_delete, moetran_get, moetran_post_opt, moetran_put_opt, poprako_get,
        poprako_post_opt, poprako_put_opt,
    },
    token::get_moetran_token,
};
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE, REFERER, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::time::Duration;
use url::Url;

// Moetran 项目集 DTO（仅用于 enriched flows）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResProjectSet {
    pub id: String,
    pub name: String,
}

// Moetran 项目 DTO（仅用于 enriched flows）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResProject {
    pub id: String,
    pub name: String,
    pub source_count: u64,
    pub translated_source_count: u64,
    pub checked_source_count: u64,
    pub team: crate::team::ResTeam,
    pub project_set: ResProjectSet,
    // Moetran may include a `role` field for native projects indicating the
    // current user's role in the project; we only passthrough it (may be null).
    #[serde(default)]
    pub role: Option<Value>,
}

// PopRaKo 项目搜索返回的精简 DTO（参考 ProjInfoReply）
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
    #[serde(default)]
    pub members: Option<Vec<PoprakoMember>>,
}

// PopRaKo 项目内的成员信息（search 接口会返回）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoMember {
    // PopRaKo 返回的用户 id 字段
    // Accept common upstream variants for robustness
    #[serde(alias = "userId", alias = "userid")]
    pub user_id: String,
    pub member_id: String,
    pub username: String,
    pub is_admin: bool,
    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
    pub is_principal: bool,
}

// PopRaKo 通用返回包裹（项目模块）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoEnvelope<T> {
    pub code: u16,
    pub data: Option<T>,
    pub message: Option<String>,
}

// PopRaKo 创建项目集请求 DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoProjSetCreateReq {
    pub projset_name: String,
    pub projset_description: String,
    pub team_id: String,
    pub mtr_token: String,
}

// PopRaKo 创建项目集响应 data DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoProjSetCreateData {
    pub projset_serial: u32,
}

// PopRaKo 项目集列表 DTO（对应 GET /projsets 返回的单项）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoProjSetInfo {
    pub projset_id: String,
    pub projset_name: String,
    pub projset_description: Option<String>,
    pub projset_serial: u32,
    pub team_id: String,
}

// PopRaKo 项目集列表外层 data 包裹
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoProjSetListData {
    pub projsets: Vec<PoprakoProjSetInfo>,
}

// PopRaKo 创建项目请求 DTO（与 ProjCreatePayload 对齐）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoProjCreateReq {
    pub proj_name: String,
    pub proj_description: String,
    pub team_id: String,
    pub projset_id: String,
    pub mtr_auth: String,
    pub workset_index: i32,
    pub source_language: String,
    pub target_languages: Vec<String>,
    pub allow_apply_type: i32,
    pub application_check_type: i32,
    pub default_role: String,
}

// PopRaKo 创建项目响应 data DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoProjCreateData {
    pub proj_id: String,
    pub proj_serial: u32,
    pub projset_index: u32,
}

// PopRaKo 指派成员到项目的请求 DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoAssignReq {
    pub proj_id: String,
    pub member_id: String,
    pub mtr_auth: String,
    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
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
    // PopRaKo 返回的成员列表（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<PoprakoMember>>,
    // 从 members 中提取的负责人 user id 列表（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principals: Option<Vec<String>>,
    // Passthrough of Moetran `role` for native projects; may be null.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<Value>,
}

// ========== Moetran 项目 target / files DTO（供 ProjectDetail 使用） ==========

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoetranProjectTarget {
    pub id: String,
    pub translated_source_count: u64,
    pub checked_source_count: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoetranProjectFile {
    pub id: String,
    pub name: String,
    pub source_count: u64,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetProjectTargetsReq {
    pub project_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetProjectFilesReq {
    pub project_id: String,
    pub target_id: Option<String>,
}

// PopRaKo 项目搜索请求 DTO（与 PickProjPayload 对齐的子集）
// 包含 proj_ids 批量查询时也需要的分页字段，避免服务端 422
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoProjSearchReq {
    pub proj_ids: Vec<String>,
    pub page: u32,
    pub limit: u32,
}

// PopRaKo 项目复杂筛选请求 DTO（仅保留仪表盘暂时需要的字段，后续可扩展）
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct PoprakoProjFilterReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuzzy_proj_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub translating_status: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub proofreading_status: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub typesetting_status: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewing_status: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_published: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_ids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub projset_ids: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_start: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

// 单一 payload: 包含 team_id 与 filter（用于 Tauri IPC）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchTeamProjectsEnrichedReq {
    pub team_id: String,
    pub filter: PoprakoProjFilterReq,
}

// 在指定团队下创建项目集（调用 PopRaKo /projset/create）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateProjsetReq {
    pub projset_name: String,
    pub projset_description: String,
    pub team_id: String,
    pub mtr_token: String,
}

#[tauri::command]
pub async fn create_projset(payload: CreateProjsetReq) -> Result<PoprakoProjSetCreateData, String> {
    tracing::info!(
        team_id = %payload.team_id,
        projset_name = %payload.projset_name,
        "poprako.projset.create.request.start"
    );

    let mut defer = WarnDefer::new("poprako.projset.create");

    let body = PoprakoProjSetCreateReq {
        projset_name: payload.projset_name,
        projset_description: payload.projset_description,
        team_id: payload.team_id,
        mtr_token: payload.mtr_token,
    };

    let reply = poprako_post_opt::<
        PoprakoProjSetCreateReq,
        PoprakoEnvelope<PoprakoProjSetCreateData>,
    >("projsets", Some(body))
    .await
    .map_err(|err| format!("创建项目集失败: {}", err))?;

    if reply.code != 201 {
        let msg = reply
            .message
            .unwrap_or_else(|| "PopRaKo 创建项目集失败".to_string());

        tracing::info!(message = %msg, code = reply.code, "poprako.projset.create.failed");

        return Err(msg);
    }

    let data = reply
        .data
        .ok_or_else(|| "PopRaKo 创建项目集返回空数据".to_string())?;

    tracing::info!(
        projset_serial = data.projset_serial,
        "poprako.projset.create.ok"
    );

    defer.success();

    Ok(data)
}

// 列出 PopRaKo 中指定团队下的项目集（调用 PopRaKo GET /projsets?team_id=）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTeamPoprakoProjsetsReq {
    pub team_id: String,
}

#[tauri::command]
pub async fn get_team_poprako_projsets(
    payload: GetTeamPoprakoProjsetsReq,
) -> Result<Vec<PoprakoProjSetInfo>, String> {
    tracing::info!(team_id = %payload.team_id, "poprako.projsets.list.request.start");

    let mut defer = WarnDefer::new("poprako.projsets.list");

    let mut query = std::collections::HashMap::new();
    query.insert("team_id", payload.team_id.clone());

    let reply = poprako_get::<PoprakoEnvelope<PoprakoProjSetListData>>("projsets", Some(&query))
        .await
        .map_err(|err| format!("获取 PopRaKo 项目集列表失败: {}", err))?;

    if reply.code != 200 {
        let msg = reply
            .message
            .unwrap_or_else(|| "PopRaKo 获取项目集列表失败".to_string());

        tracing::info!(message = %msg, code = reply.code, "poprako.projsets.list.failed");

        return Err(msg);
    }

    let data = reply
        .data
        .ok_or_else(|| "PopRaKo 获取项目集列表返回空数据".to_string())?;

    let count = data.projsets.len();
    tracing::info!(team_id = %payload.team_id, count = count, "poprako.projsets.list.ok");

    defer.success();

    Ok(data.projsets)
}

// 在已有项目集中创建项目（调用 PopRaKo /proj/create）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateProjReq {
    pub proj_name: String,
    pub proj_description: String,
    pub team_id: String,
    pub projset_id: String,
    pub mtr_auth: String,
    pub workset_index: i32,
    pub source_language: String,
    pub target_languages: Vec<String>,
    pub allow_apply_type: i32,
    pub application_check_type: i32,
    pub default_role: String,
}

#[tauri::command]
pub async fn create_proj(payload: CreateProjReq) -> Result<PoprakoProjCreateData, String> {
    tracing::info!(
        team_id = %payload.team_id,
        proj_name = %payload.proj_name,
        projset_id = %payload.projset_id,
        "poprako.proj.create.request.start"
    );

    let mut defer = WarnDefer::new("poprako.proj.create");

    let body = PoprakoProjCreateReq {
        proj_name: payload.proj_name,
        proj_description: payload.proj_description,
        team_id: payload.team_id,
        projset_id: payload.projset_id,
        mtr_auth: payload.mtr_auth,
        workset_index: payload.workset_index,
        source_language: payload.source_language,
        target_languages: payload.target_languages,
        allow_apply_type: payload.allow_apply_type,
        application_check_type: payload.application_check_type,
        default_role: payload.default_role,
    };

    let reply = poprako_post_opt::<PoprakoProjCreateReq, PoprakoEnvelope<PoprakoProjCreateData>>(
        "projs",
        Some(body),
    )
    .await
    .map_err(|err| format!("创建项目失败: {}", err))?;

    if reply.code != 201 {
        let msg = reply
            .message
            .unwrap_or_else(|| "PopRaKo 创建项目失败".to_string());

        tracing::info!(message = %msg, code = reply.code, "poprako.proj.create.failed");

        return Err(msg);
    }

    let data = reply
        .data
        .ok_or_else(|| "PopRaKo 创建项目返回空数据".to_string())?;

    tracing::info!(
        proj_id = %data.proj_id,
        proj_serial = data.proj_serial,
        projset_index = data.projset_index,
        "poprako.proj.create.ok"
    );

    defer.success();

    Ok(data)
}

// 为项目指派成员角色（调用 PopRaKo POST /projs/{proj_id}/assign）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssignMemberReq {
    pub proj_id: String,
    pub member_id: String,
    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
}

#[tauri::command]
pub async fn assign_member_to_proj(payload: AssignMemberReq) -> Result<(), String> {
    tracing::info!(
        proj_id = %payload.proj_id,
        member_id = %payload.member_id,
        "poprako.proj.assign.request.start"
    );

    let mut defer = WarnDefer::new("poprako.proj.assign");

    let moetran_token = get_moetran_token()
        .await?
        .ok_or_else(|| "无法获取 Moetran Token".to_string())?;

    let body = PoprakoAssignReq {
        proj_id: payload.proj_id.clone(),
        member_id: payload.member_id.clone(),
        mtr_auth: moetran_token,
        is_translator: payload.is_translator,
        is_proofreader: payload.is_proofreader,
        is_typesetter: payload.is_typesetter,
    };

    let path = format!("projs/{}/assign", payload.proj_id);

    poprako_post_opt::<PoprakoAssignReq, ()>(&path, Some(body))
        .await
        .map_err(|err| format!("指派成员到项目失败: {}", err))?;

    tracing::info!("poprako.proj.assign.ok");

    defer.success();

    Ok(())
}

// ========== Moetran 项目 targets / files 命令（供 ProjectDetail 使用） ==========

#[tauri::command]
pub async fn get_project_targets(
    payload: GetProjectTargetsReq,
) -> Result<Vec<MoetranProjectTarget>, String> {
    tracing::info!(project_id = %payload.project_id, "moetran.project.targets.request.start");

    let mut defer = WarnDefer::new("moetran.project.targets");

    let mut query = std::collections::HashMap::new();
    query.insert("page", "1".to_string());
    // 默认每页请求较小的数量，避免一次性拉取过多导致超时或 OOM
    query.insert("limit", "5".to_string());
    query.insert("word", "".to_string());
    // 仅请求尨译项目（status=0）
    query.insert("status", "0".to_string());

    let path = format!("projects/{}/targets", payload.project_id);
    tracing::debug!(%path, ?query, "moetran.get_project_targets request");

    let raw_list: Vec<serde_json::Value> = match moetran_get(&path, Some(&query)).await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(project_id = %payload.project_id, %path, ?query, error = %e, "moetran.get_project_targets failed");
            return Err(format!("获取项目 targets 失败: {}", e));
        }
    };

    let result: Vec<MoetranProjectTarget> = raw_list
        .into_iter()
        .filter_map(|v| {
            let id = v.get("id")?.as_str()?.to_string();
            let translated = v
                .get("translated_source_count")
                .and_then(|x| x.as_u64())
                .unwrap_or(0);
            let checked = v
                .get("checked_source_count")
                .and_then(|x| x.as_u64())
                .unwrap_or(0);

            Some(MoetranProjectTarget {
                id,
                translated_source_count: translated,
                checked_source_count: checked,
            })
        })
        .collect();

    let count = result.len();
    tracing::info!(project_id = %payload.project_id, count = count, "moetran.project.targets.ok");

    defer.success();

    Ok(result)
}

#[tauri::command]
pub async fn get_project_files(
    payload: GetProjectFilesReq,
) -> Result<Vec<MoetranProjectFile>, String> {
    tracing::info!(
        project_id = %payload.project_id,
        target_id = ?payload.target_id,
        "moetran.project.files.request.start"
    );

    let mut defer = WarnDefer::new("moetran.project.files");

    let mut query = std::collections::HashMap::new();
    query.insert("page", "1".to_string());
    query.insert("limit", "100000".to_string());
    query.insert("word", "".to_string());
    if let Some(t) = &payload.target_id {
        query.insert("target", t.clone());
    }
    // 仅请求尨译项目（status=0）
    query.insert("status", "0".to_string());

    let path = format!("projects/{}/files", payload.project_id);
    tracing::debug!(%path, ?query, "moetran.get_project_files request");

    let raw_list: Vec<serde_json::Value> = match moetran_get(&path, Some(&query)).await {
        Ok(list) => list,
        Err(e) => {
            tracing::error!(project_id = %payload.project_id, target_id = ?payload.target_id, %path, ?query, error = %e, "moetran.get_project_files failed");
            return Err(format!("获取项目 files 失败: {}", e));
        }
    };

    let result: Vec<MoetranProjectFile> = raw_list
        .into_iter()
        .filter_map(|v| {
            let id = v.get("id")?.as_str()?.to_string();
            let name = v.get("name")?.as_str()?.to_string();
            let source = v.get("source_count").and_then(|x| x.as_u64()).unwrap_or(0);
            let url = v.get("url")?.as_str()?.to_string();

            Some(MoetranProjectFile {
                id,
                name,
                source_count: source,
                url,
            })
        })
        .collect();

    let count = result.len();
    tracing::info!(
        project_id = %payload.project_id,
        target_id = ?payload.target_id,
        count = count,
        "moetran.project.files.ok"
    );

    defer.success();

    Ok(result)
}

// 获取当前用户的 enriched 项目列表（Moetran 列表 + PopRaKo /projs/search 补充）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetUserProjectsEnrichedReq {
    pub page: u32,
    pub limit: u32,
}

#[tauri::command]
#[tracing::instrument]
pub async fn get_user_projects_enriched(
    payload: GetUserProjectsEnrichedReq,
) -> Result<Vec<ResProjectEnriched>, String> {
    tracing::info!(
        page = payload.page,
        limit = payload.limit,
        "user.projects_enriched.request.start"
    );

    let path = "user/projects".to_string();
    let mut query = std::collections::HashMap::new();
    query.insert("page", payload.page.to_string());
    query.insert("limit", payload.limit.to_string());
    query.insert("status", "0".to_string());

    let base_list: Vec<ResProject> = moetran_get(&path, Some(&query))
        .await
        .map_err(|err| format!("获取用户项目列表失败: {}", err))?;

    if base_list.is_empty() {
        tracing::info!("user.projects_enriched.empty");

        return Ok(vec![]);
    }

    let ids: Vec<String> = base_list.iter().map(|p| p.id.clone()).collect();

    let search_body = PoprakoProjSearchReq {
        proj_ids: ids,
        page: payload.page,
        limit: payload.limit,
    };

    let reply = poprako_post_opt::<PoprakoProjSearchReq, PoprakoEnvelope<Vec<PoprakoProjInfo>>>(
        "projs/search",
        Some(search_body),
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
            .unwrap_or_else(|| "PopRaKo 项目搜索失败".to_string());

        tracing::info!(message = %msg, code = reply.code, "poprako.projs.search.failed");
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
                members: extra.members.clone(),
                principals: extra.members.as_ref().map(|ms| {
                    ms.iter()
                        .filter(|m| m.is_principal)
                        .map(|m| m.user_id.clone())
                        .collect()
                }),
                role: item.role.clone(),
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
                members: None,
                principals: None,
                role: item.role.clone(),
            });
        }
    }

    tracing::info!(
        count = enriched_list.len(),
        "user.projects_enriched.request.ok"
    );

    Ok(enriched_list)
}

// 获取指定汉化组的 enriched 项目列表（Moetran 列表 + PopRaKo /projs/search 补充）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTeamProjectsEnrichedReq {
    pub team_id: String,
    pub page: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn get_team_projects_enriched(
    payload: GetTeamProjectsEnrichedReq,
) -> Result<Vec<ResProjectEnriched>, String> {
    tracing::info!(team_id = %payload.team_id, page = payload.page, limit = payload.limit, "team.projects_enriched.request.start");

    let path = format!("teams/{}/projects", payload.team_id);
    let mut query = std::collections::HashMap::new();
    query.insert("page", payload.page.to_string());
    query.insert("limit", payload.limit.to_string());
    query.insert("status", "0".to_string());

    let base_list: Vec<ResProject> = moetran_get(&path, Some(&query))
        .await
        .map_err(|err| format!("获取团队项目列表失败: {}", err))?;

    if base_list.is_empty() {
        tracing::info!(team_id = %payload.team_id, "team.projects_enriched.empty");
        return Ok(vec![]);
    }

    let ids: Vec<String> = base_list.iter().map(|p| p.id.clone()).collect();

    let search_body = PoprakoProjSearchReq {
        proj_ids: ids,
        page: payload.page,
        limit: payload.limit,
    };

    let reply = poprako_post_opt::<PoprakoProjSearchReq, PoprakoEnvelope<Vec<PoprakoProjInfo>>>(
        "projs/search",
        Some(search_body),
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
            .unwrap_or_else(|| "PopRaKo 项目搜索失败".to_string());

        tracing::info!(message = %msg, code = reply.code, "poprako.projs.search.failed");
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
                members: extra.members.clone(),
                principals: extra.members.as_ref().map(|ms| {
                    ms.iter()
                        .filter(|m| m.is_principal)
                        .map(|m| m.user_id.clone())
                        .collect()
                }),
                role: item.role.clone(),
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
                members: None,
                principals: None,
                role: item.role.clone(),
            });
        }
    }

    tracing::info!(team_id = %payload.team_id, count = enriched_list.len(), "team.projects_enriched.request.ok");

    Ok(enriched_list)
}

// user 维度：基于 PopRaKo /projs/search + Moetran /user/projects?word= 进行组合搜索
#[tauri::command]
pub async fn search_user_projects_enriched(
    filter: PoprakoProjFilterReq,
) -> Result<Vec<ResProjectEnriched>, String> {
    tracing::info!("user.projects_enriched.search.start");

    let mut defer = WarnDefer::new("user.projects_enriched.search");

    let reply = poprako_post_opt::<PoprakoProjFilterReq, PoprakoEnvelope<Vec<PoprakoProjInfo>>>(
        "projs/search",
        Some(filter),
    )
    .await
    .map_err(|err| format!("PopRaKo 项目搜索失败: {}", err))?;

    if reply.code != 200 {
        let msg = reply
            .message
            .unwrap_or_else(|| "PopRaKo 项目搜索失败".to_string());

        tracing::info!(message = %msg, code = reply.code, "poprako.projs.search.failed");

        return Err(msg);
    }

    let items = match reply.data {
        Some(v) => v,
        None => {
            tracing::info!("user.projects_enriched.search.empty");
            defer.success();
            return Ok(vec![]);
        }
    };

    // 逐个 proj_name 调用 Moetran /user/projects?word=，由于后端保证唯一匹配，直接取第一个
    let mut enriched_list = Vec::new();

    for extra in items {
        let mut query = std::collections::HashMap::new();
        query.insert("word", extra.proj_name.clone());
        query.insert("status", "0".to_string());

        let list: Vec<ResProject> = moetran_get("user/projects", Some(&query))
            .await
            .map_err(|err| format!("获取用户项目列表失败: {}", err))?;

        if let Some(base) = list.first() {
            enriched_list.push(ResProjectEnriched {
                id: base.id.clone(),
                name: base.name.clone(),
                source_count: base.source_count,
                translated_source_count: base.translated_source_count,
                checked_source_count: base.checked_source_count,
                team: base.team.clone(),
                project_set: base.project_set.clone(),
                has_poprako: true,
                projset_index: Some(extra.projset_index),
                translating_status: Some(extra.translating_status),
                proofreading_status: Some(extra.proofreading_status),
                typesetting_status: Some(extra.typesetting_status),
                reviewing_status: Some(extra.reviewing_status),
                is_published: Some(extra.is_published),
                members: extra.members.clone(),
                principals: extra.members.as_ref().map(|ms| {
                    ms.iter()
                        .filter(|m| m.is_principal)
                        .map(|m| m.user_id.clone())
                        .collect()
                }),
                role: base.role.clone(),
            });
        }
    }

    tracing::info!(
        count = enriched_list.len(),
        "user.projects_enriched.search.ok"
    );

    defer.success();

    Ok(enriched_list)
}

// team 维度：基于 PopRaKo /projs/search + Moetran /teams/:team_id/projects?word= 进行组合搜索
#[tauri::command]
pub async fn search_team_projects_enriched(
    payload: SearchTeamProjectsEnrichedReq,
) -> Result<Vec<ResProjectEnriched>, String> {
    tracing::info!(team_id = %payload.team_id, "team.projects_enriched.search.start");

    let mut defer = WarnDefer::new("team.projects_enriched.search");

    let reply = poprako_post_opt::<PoprakoProjFilterReq, PoprakoEnvelope<Vec<PoprakoProjInfo>>>(
        "projs/search",
        Some(payload.filter.clone()),
    )
    .await
    .map_err(|err| format!("PopRaKo 项目搜索失败: {}", err))?;

    if reply.code != 200 {
        let msg = reply
            .message
            .unwrap_or_else(|| "PopRaKo 项目搜索失败".to_string());

        tracing::info!(message = %msg, code = reply.code, "poprako.projs.search.failed");

        return Err(msg);
    }

    let items = match reply.data {
        Some(v) => v,
        None => {
            tracing::info!(team_id = %payload.team_id, "team.projects_enriched.search.empty");
            defer.success();
            return Ok(vec![]);
        }
    };

    let mut enriched_list = Vec::new();

    for extra in items {
        let mut query = std::collections::HashMap::new();
        query.insert("word", extra.proj_name.clone());
        query.insert("status", "0".to_string());

        let path = format!("teams/{}/projects", payload.team_id);

        let list: Vec<ResProject> = moetran_get(&path, Some(&query))
            .await
            .map_err(|err| format!("获取团队项目列表失败: {}", err))?;

        if let Some(base) = list.first() {
            enriched_list.push(ResProjectEnriched {
                id: base.id.clone(),
                name: base.name.clone(),
                source_count: base.source_count,
                translated_source_count: base.translated_source_count,
                checked_source_count: base.checked_source_count,
                team: base.team.clone(),
                project_set: base.project_set.clone(),
                has_poprako: true,
                projset_index: Some(extra.projset_index),
                translating_status: Some(extra.translating_status),
                proofreading_status: Some(extra.proofreading_status),
                typesetting_status: Some(extra.typesetting_status),
                reviewing_status: Some(extra.reviewing_status),
                is_published: Some(extra.is_published),
                members: extra.members.clone(),
                principals: extra.members.as_ref().map(|ms| {
                    ms.iter()
                        .filter(|m| m.is_principal)
                        .map(|m| m.user_id.clone())
                        .collect()
                }),
                role: base.role.clone(),
            });
        }
    }

    tracing::info!(
        team_id = %payload.team_id,
        count = enriched_list.len(),
        "team.projects_enriched.search.ok"
    );

    defer.success();

    Ok(enriched_list)
}

// ========== 获取文件的 sources（用于 TranslatorView） ==========

// Moetran 单个 translation DTO（精简）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoetranTranslation {
    pub id: String,
    pub content: String,
    pub proofread_content: Option<String>,
    pub selected: bool,
}

// Moetran source DTO（精简版，仅包含 TranslatorView 所需字段）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoetranSource {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub position_type: i32,
    pub my_translation: Option<MoetranTranslation>,
    #[serde(default)]
    pub translations: Vec<MoetranTranslation>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetPageSourcesReq {
    pub file_id: String,
    pub target_id: String,
}

#[tauri::command]
pub async fn get_page_sources(payload: GetPageSourcesReq) -> Result<Vec<MoetranSource>, String> {
    tracing::info!(
        file_id = %payload.file_id,
        target_id = %payload.target_id,
        "moetran.sources.fetch.request.start"
    );

    let mut defer = WarnDefer::new("moetran.sources.fetch");

    let endpoint = format!("files/{}/sources", payload.file_id);
    let mut query = std::collections::HashMap::new();
    query.insert("target_id", payload.target_id.clone());
    query.insert("paging", "false".to_string());

    let sources = moetran_get::<Vec<MoetranSource>>(&endpoint, Some(&query))
        .await
        .map_err(|err| format!("获取页面源失败: {}", err))?;

    let count = sources.len();
    tracing::info!(
        file_id = %payload.file_id,
        target_id = %payload.target_id,
        count = count,
        "moetran.sources.fetch.ok"
    );

    defer.success();

    Ok(sources)
}

// 在指定文件上创建一个 source（标记）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateSourceReq {
    pub file_id: String,
    pub x: f64,
    pub y: f64,
    #[serde(default)]
    pub position_type: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
}

#[tauri::command]
pub async fn create_source(payload: CreateSourceReq) -> Result<MoetranSource, String> {
    tracing::info!(file_id = %payload.file_id, x = payload.x, y = payload.y, "moetran.source.create.start");

    let mut defer = WarnDefer::new("moetran.source.create");

    let path = format!("files/{}/sources", payload.file_id);

    let mut body = serde_json::Map::new();

    body.insert("x".to_string(), serde_json::Value::from(payload.x));
    body.insert("y".to_string(), serde_json::Value::from(payload.y));
    body.insert(
        "position_type".to_string(),
        serde_json::Value::from(payload.position_type),
    );

    if let Some(w) = payload.width {
        body.insert("width".to_string(), serde_json::Value::from(w));
    }

    if let Some(h) = payload.height {
        body.insert("height".to_string(), serde_json::Value::from(h));
    }

    let reply = moetran_post_opt::<serde_json::Value, MoetranSource>(
        &path,
        Some(serde_json::Value::Object(body)),
    )
    .await
    .map_err(|err| format!("创建 source 失败: {}", err))?;

    tracing::info!(source_id = %reply.id, "moetran.source.create.ok");

    defer.success();

    Ok(reply)
}

// 更新 source（框内/框外切换）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateSourceReq {
    pub source_id: String,
    pub position_type: i32,
}

#[tauri::command]
pub async fn update_source(payload: UpdateSourceReq) -> Result<MoetranSource, String> {
    tracing::info!(
        source_id = %payload.source_id,
        position_type = payload.position_type,
        "moetran.source.update.start"
    );

    let mut defer = WarnDefer::new("moetran.source.update");

    let path = format!("sources/{}", payload.source_id);

    let mut body = serde_json::Map::new();
    body.insert(
        "id".to_string(),
        serde_json::Value::String(payload.source_id.clone()),
    );
    body.insert(
        "position_type".to_string(),
        serde_json::Value::from(payload.position_type),
    );

    let reply = moetran_put_opt::<serde_json::Value, MoetranSource>(
        &path,
        Some(serde_json::Value::Object(body)),
    )
    .await
    .map_err(|err| format!("更新 source 失败: {}", err))?;

    tracing::info!(
        source_id = %reply.id,
        position_type = reply.position_type,
        "moetran.source.update.ok"
    );

    defer.success();

    Ok(reply)
}

// 删除 source
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeleteSourceReq {
    pub source_id: String,
}

#[tauri::command]
pub async fn delete_source(payload: DeleteSourceReq) -> Result<(), String> {
    tracing::info!(source_id = %payload.source_id, "moetran.source.delete.start");

    let mut defer = WarnDefer::new("moetran.source.delete");

    let path = format!("sources/{}", payload.source_id);

    moetran_delete::<serde_json::Value>(&path)
        .await
        .map_err(|err| format!("删除 source 失败: {}", err))?;

    tracing::info!(source_id = %payload.source_id, "moetran.source.delete.ok");

    defer.success();

    Ok(())
}

// 提交翻译稿
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmitTranslationReq {
    pub source_id: String,
    pub target_id: String,
    pub content: String,
}

#[tauri::command]
pub async fn submit_translation(
    payload: SubmitTranslationReq,
) -> Result<MoetranTranslation, String> {
    tracing::info!(
        source_id = %payload.source_id,
        target_id = %payload.target_id,
        content_len = payload.content.len(),
        "moetran.translation.submit.start"
    );

    let mut defer = WarnDefer::new("moetran.translation.submit");

    let path = format!("sources/{}/translations", payload.source_id);

    let body = serde_json::json!({
        "target_id": payload.target_id,
        "content": payload.content,
    });

    let reply = moetran_post_opt::<serde_json::Value, MoetranTranslation>(&path, Some(body))
        .await
        .map_err(|err| format!("提交翻译失败: {}", err))?;

    tracing::info!(
        translation_id = %reply.id,
        source_id = %payload.source_id,
        "moetran.translation.submit.ok"
    );

    defer.success();

    Ok(reply)
}

// 更新翻译稿（包括校对状态与校对内容）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateTranslationReq {
    pub translation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proofread_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[tauri::command]
pub async fn update_translation(
    payload: UpdateTranslationReq,
) -> Result<MoetranTranslation, String> {
    let has_selected = payload.selected.is_some();
    let has_proof = payload.proofread_content.is_some();
    let has_content = payload.content.is_some();

    if !has_selected && !has_proof && !has_content {
        return Err("至少需要一个可更新字段".to_string());
    }

    tracing::info!(
        translation_id = %payload.translation_id,
        has_selected,
        has_proof,
        has_content,
        "moetran.translation.update.start"
    );

    let mut defer = WarnDefer::new("moetran.translation.update");

    let mut body = Map::new();

    if let Some(selected) = payload.selected {
        body.insert("selected".to_string(), Value::Bool(selected));
    }

    if let Some(proof) = payload.proofread_content {
        body.insert("proofread_content".to_string(), Value::String(proof));
    }

    if let Some(content) = payload.content {
        body.insert("content".to_string(), Value::String(content));
    }

    let path = format!("translations/{}", payload.translation_id);

    let reply =
        moetran_put_opt::<serde_json::Value, MoetranTranslation>(&path, Some(Value::Object(body)))
            .await
            .map_err(|err| format!("更新翻译失败: {}", err))?;

    tracing::info!(
        translation_id = %reply.id,
        selected = reply.selected,
        "moetran.translation.update.ok"
    );

    defer.success();

    Ok(reply)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProxyImageReply {
    pub b64: String,
    pub content_type: String,
}

#[tauri::command]
pub async fn proxy_image(url: String) -> Result<ProxyImageReply, String> {
    tracing::info!(%url, "proxy_image.request.start");

    // Basic validation: parse URL and whitelist host
    let parsed = Url::parse(&url).map_err(|e| format!("Invalid URL: {}", e))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| "URL missing host".to_string())?;

    // Only allow m-t.pics subdomains for now. Adjust whitelist as needed.
    if !host.ends_with("m-t.pics") {
        return Err("Host not allowed".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;

    let mut headers = HeaderMap::new();
    headers.insert(
        ACCEPT,
        HeaderValue::from_static(
            "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
        ),
    );
    headers.insert(REFERER, HeaderValue::from_static("https://moetran.com/"));
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0"),
    );
    headers.insert(
        "Sec-CH-UA",
        HeaderValue::from_static(
            "\"Chromium\";v=\"142\", \"Microsoft Edge\";v=\"142\", \"Not_A Brand\";v=\"99\"",
        ),
    );

    let resp = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Fetch failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Remote returned status {}", resp.status()));
    }

    // Enforce size limit (32 MB)
    let content_length = resp.content_length().unwrap_or(0);
    if content_length > 32 * 1024 * 1024 {
        return Err("Remote file too large".to_string());
    }

    let content_type = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Read body failed: {}", e))?;
    if (bytes.len() as u64) > 32 * 1024 * 1024 {
        return Err("Remote file too large".to_string());
    }

    let b64 = general_purpose::STANDARD.encode(&bytes);

    tracing::info!(size = bytes.len(), "proxy_image.request.ok");

    Ok(ProxyImageReply { b64, content_type })
}

// ========== 更新项目状态与发布（PopRaKo API #9, #10） ==========

// 更新项目流程状态（仅项目负责人可调用）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpdateProjStatusReq {
    pub proj_id: String,
    pub status_type: String, // "translating" / "proofreading" / "typesetting" / "reviewing"
    pub new_status: i32,     // 0=pending, 1=wip, 2=completed
}

#[tauri::command]
pub async fn update_proj_status(payload: UpdateProjStatusReq) -> Result<(), String> {
    tracing::info!(
        proj_id = %payload.proj_id,
        status_type = %payload.status_type,
        new_status = payload.new_status,
        "poprako.proj.status.update.request.start"
    );

    let mut defer = WarnDefer::new("poprako.proj.status.update");

    let path = format!("projs/{}/status", payload.proj_id);

    let body = serde_json::json!({
        "proj_id": payload.proj_id,
        "status_type": payload.status_type,
        "new_status": payload.new_status,
    });

    // PopRaKo API returns 204 No Content on success
    // Use unit `()` as the expected response type so empty body / 204 is handled.
    poprako_put_opt::<serde_json::Value, ()>(&path, Some(body))
        .await
        .map_err(|err| format!("更新项目状态失败: {}", err))?;

    tracing::info!(
        proj_id = %payload.proj_id,
        status_type = %payload.status_type,
        new_status = payload.new_status,
        "poprako.proj.status.update.ok"
    );

    defer.success();

    Ok(())
}

// 标记项目为已发布（仅项目负责人可调用）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublishProjReq {
    pub proj_id: String,
}

#[tauri::command]
pub async fn publish_proj(payload: PublishProjReq) -> Result<(), String> {
    tracing::info!(
        proj_id = %payload.proj_id,
        "poprako.proj.publish.request.start"
    );

    let mut defer = WarnDefer::new("poprako.proj.publish");

    let path = format!("projs/{}/publish", payload.proj_id);

    // PopRaKo API returns 204 No Content on success (no body)
    // Use unit `()` as the expected response type so empty body / 204 is handled.
    poprako_put_opt::<(), ()>(&path, None)
        .await
        .map_err(|err| format!("标记项目为已发布失败: {}", err))?;

    tracing::info!(
        proj_id = %payload.proj_id,
        "poprako.proj.publish.ok"
    );

    defer.success();

    Ok(())
}

// 上传漫画页文件到 Moetran 项目
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UploadProjectFileReq {
    pub project_id: String,
    pub file_name: String,
    pub file_bytes: Vec<u8>,
}

#[tauri::command]
pub async fn upload_project_file(payload: UploadProjectFileReq) -> Result<(), String> {
    tracing::info!(
        project_id = %payload.project_id,
        file_name = %payload.file_name,
        file_size = payload.file_bytes.len(),
        "moetran.project.file.upload.start"
    );

    let mut defer = WarnDefer::new("moetran.project.file.upload");

    // 验证文件类型（仅支持 jpg/jpeg/png/bmp）
    let ext = payload
        .file_name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase();
    if !matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "bmp") {
        return Err(format!(
            "Unsupported file type: {}. Only jpg/jpeg/png/bmp are allowed",
            ext
        ));
    }

    // 构建 multipart/form-data 请求
    let token = match get_moetran_token().await {
        Ok(Some(t)) => t,
        Ok(None) => return Err("Missing Moetran token: Authorization required".to_string()),
        Err(e) => return Err(format!("Failed to get Moetran token: {}", e)),
    };

    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(payload.file_bytes)
            .file_name(payload.file_name.clone())
            .mime_str("application/octet-stream")
            .map_err(|err| format!("Failed to set file mime type: {}", err))?,
    );

    let base_url = std::env::var("MOETRAN_URL").unwrap_or("https://api.moetran.com".to_string());
    let url = format!("{}/v1/projects/{}/files", base_url, payload.project_id);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|err| format!("Failed to create HTTP client: {}", err))?;

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .multipart(form)
        .send()
        .await
        .map_err(|err| format!("File upload failed: {}", err))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_else(|_| "<empty>".to_string());
        return Err(format!(
            "File upload failed with status {}: {}",
            status, body
        ));
    }

    tracing::info!(
        project_id = %payload.project_id,
        file_name = %payload.file_name,
        "moetran.project.file.upload.ok"
    );

    defer.success();

    Ok(())
}

// ==================== Assignment 相关 ====================

// PopRaKo Assignment DTO（对应 API 文档中的 ProjAssignInfo）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PoprakoAssignment {
    pub proj_id: String,
    pub proj_name: String,
    pub projset_serial: u32,
    pub projset_index: u32,
    pub member_id: String,
    pub username: String,
    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
    pub updated_at: i64, // Unix timestamp (seconds)
}

// 获取 assignments 请求
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetAssignmentsReq {
    #[serde(default)]
    pub time_start: i64,
}

// 获取 assignments 列表（调用 PopRaKo GET /assigns）
#[tauri::command]
pub async fn get_assignments(payload: GetAssignmentsReq) -> Result<Vec<PoprakoAssignment>, String> {
    tracing::info!(
        time_start = payload.time_start,
        "poprako.assigns.list.request.start"
    );

    let mut defer = WarnDefer::new("poprako.assigns.list");

    let mut query = std::collections::HashMap::new();
    query.insert("time_start", payload.time_start.to_string());

    let reply = poprako_get::<PoprakoEnvelope<Vec<PoprakoAssignment>>>("assigns", Some(&query))
        .await
        .map_err(|err| format!("获取派活列表失败: {}", err))?;

    if reply.code != 200 {
        let msg = reply
            .message
            .unwrap_or_else(|| "PopRaKo 获取派活列表失败".to_string());

        tracing::info!(
            message = %msg,
            code = reply.code,
            "poprako.assigns.list.failed"
        );

        return Err(msg);
    }

    let data = reply
        .data
        .ok_or_else(|| "PopRaKo 获取派活列表返回空数据".to_string())?;

    let count = data.len();
    tracing::info!(
        time_start = payload.time_start,
        count = count,
        "poprako.assigns.list.ok"
    );

    defer.success();

    Ok(data)
}

// 创建 PopRaKo 项目集的别名命令（前端调用 create_poprako_projset）
#[tauri::command]
pub async fn create_poprako_projset(
    payload: CreateProjsetReq,
) -> Result<PoprakoProjSetCreateData, String> {
    create_projset(payload).await
}
