use crate::{
    defer::WarnDefer,
    http::{moetran_get, poprako_get, poprako_post_opt},
    token::get_moetran_token,
};
use serde::{Deserialize, Serialize};

// 项目集 DTO
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResProjectSet {
    pub id: String,
    pub name: String,
}

// 获取指定汉化组的项目集列表
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTeamProjectSetsReq {
    pub team_id: String,
    pub page: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn get_team_project_sets(
    payload: GetTeamProjectSetsReq,
) -> Result<Vec<ResProjectSet>, String> {
    tracing::info!(team_id = %payload.team_id, page = payload.page, limit = payload.limit, "team.project_sets.request.start");

    let mut defer = WarnDefer::new("team.project_sets.request");

    let path = format!(
        "teams/{}/project-sets?page={}&limit={}",
        payload.team_id, payload.page, payload.limit
    );

    let list: Vec<ResProjectSet> = moetran_get(&path, None)
        .await
        .map_err(|err| format!("获取项目集列表失败: {}", err))?;

    tracing::info!(count = list.len(), "team.project_sets.request.ok");

    defer.success();

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
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetTeamProjectsReq {
    pub team_id: String,
    pub project_set: String,
    pub page: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn get_team_projects(payload: GetTeamProjectsReq) -> Result<Vec<ResProject>, String> {
    tracing::info!(team_id = %payload.team_id, project_set = %payload.project_set, page = payload.page, limit = payload.limit, "team.projects.request.start");

    let mut defer = WarnDefer::new("team.projects.request");

    let path = format!(
        "teams/{}/projects?project_set={}&page={}&limit={}",
        payload.team_id, payload.project_set, payload.page, payload.limit
    );

    let list: Vec<ResProject> = moetran_get(&path, None)
        .await
        .map_err(|err| format!("获取项目列表失败: {}", err))?;

    tracing::info!(count = list.len(), "team.projects.request.ok");

    defer.success();

    Ok(list)
}

// 获取当前用户的项目列表
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GetUserProjectsReq {
    pub page: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn get_user_projects(payload: GetUserProjectsReq) -> Result<Vec<ResProject>, String> {
    tracing::info!(
        page = payload.page,
        limit = payload.limit,
        "user.projects.request.start"
    );

    let mut defer = WarnDefer::new("user.projects.request");

    let path = format!(
        "user/projects?page={}&limit={}",
        payload.page, payload.limit
    );

    let list: Vec<ResProject> = moetran_get(&path, None)
        .await
        .map_err(|err| format!("获取用户项目列表失败: {}", err))?;

    tracing::info!(count = list.len(), "user.projects.request.ok");

    defer.success();

    Ok(list)
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
    pub time_start: Option<i64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
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

    let path = format!(
        "user/projects?page={}&limit={}",
        payload.page, payload.limit
    );

    let base_list: Vec<ResProject> = moetran_get(&path, None)
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

    let path = format!(
        "teams/{}/projects?page={}&limit={}",
        payload.team_id, payload.page, payload.limit
    );

    let base_list: Vec<ResProject> = moetran_get(&path, None)
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
    team_id: String,
    filter: PoprakoProjFilterReq,
) -> Result<Vec<ResProjectEnriched>, String> {
    tracing::info!(team_id = %team_id, "team.projects_enriched.search.start");

    let mut defer = WarnDefer::new("team.projects_enriched.search");

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
            tracing::info!(team_id = %team_id, "team.projects_enriched.search.empty");
            defer.success();
            return Ok(vec![]);
        }
    };

    let mut enriched_list = Vec::new();

    for extra in items {
        let mut query = std::collections::HashMap::new();
        query.insert("word", extra.proj_name.clone());

        let path = format!("teams/{}/projects", team_id);

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
            });
        }
    }

    tracing::info!(
        team_id = %team_id,
        count = enriched_list.len(),
        "team.projects_enriched.search.ok"
    );

    defer.success();

    Ok(enriched_list)
}
