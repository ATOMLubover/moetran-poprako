use serde::{Deserialize, Serialize};

use time::OffsetDateTime;
use tracing::info;

use crate::{
    defer::WarnDefer,
    http::{poprako_get, poprako_post_opt},
};

#[derive(Debug, Deserialize)]
pub struct PoprakoEnvelope<T> {
    pub code: u16,
    pub data: Option<T>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoprakoMemberSearchRaw {
    pub member_id: String,
    pub user_id: String,
    pub username: String,
    pub is_admin: Option<bool>,
    pub is_translator: Option<bool>,
    pub is_proofreader: Option<bool>,
    pub is_typesetter: Option<bool>,
    pub is_redrawer: Option<bool>,
    pub is_principal: Option<bool>,
    pub last_active: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize)]
pub struct PoprakoMemberSearchItem {
    pub member_id: String,
    pub user_id: String,
    pub username: String,
    pub is_admin: Option<bool>,
    pub is_translator: Option<bool>,
    pub is_proofreader: Option<bool>,
    pub is_typesetter: Option<bool>,
    pub is_redrawer: Option<bool>,
    pub is_principal: Option<bool>,
    pub last_active: Option<i64>,
}

// 当前登录用户在指定 team 中的成员信息（用于判断是否为管理员等）
#[derive(Debug, Serialize, Deserialize)]
pub struct PoprakoMemberInfo {
    pub member_id: String,
    pub is_admin: bool,
    pub is_translator: bool,
    pub is_proofreader: bool,
    pub is_typesetter: bool,
    pub is_principal: bool,
}

// 与 PopRaKo 文档中的 PickMemberPayload 对应
#[derive(Debug, Serialize, Deserialize)]
pub struct ReqMembers {
    pub team_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fuzzy_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

// IPC 返回结构：包一层，避免直接使用 Vec 作为 IpcResponse
#[derive(Debug, Serialize)]
pub struct MembersReply {
    pub items: Vec<PoprakoMemberSearchItem>,
}

#[tauri::command]
pub async fn get_members(payload: ReqMembers) -> Result<MembersReply, String> {
    info!(
        team_id = %payload.team_id,
        position = ?payload.position,
        fuzzy_name = ?payload.fuzzy_name,
        page = ?payload.page,
        limit = ?payload.limit,
        "poprako.members.request",
    );

    let mut defer = WarnDefer::new("poprako.members.request");

    let reply: PoprakoEnvelope<Vec<PoprakoMemberSearchRaw>> =
        poprako_post_opt("members/search", Some(&payload))
            .await
            .map_err(|err| format!("Failed to fetch members: {}", err))?;

    if reply.code != 200 {
        return Err(reply.message.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let items = reply.data.unwrap_or_default();

    let converted: Vec<PoprakoMemberSearchItem> = items
        .into_iter()
        .map(|m| PoprakoMemberSearchItem {
            member_id: m.member_id,
            user_id: m.user_id,
            username: m.username,
            is_admin: m.is_admin,
            is_translator: m.is_translator,
            is_proofreader: m.is_proofreader,
            is_typesetter: m.is_typesetter,
            is_redrawer: m.is_redrawer,
            is_principal: m.is_principal,
            last_active: m.last_active.map(|dt| dt.unix_timestamp()),
        })
        .collect();

    defer.success();

    Ok(MembersReply { items: converted })
}

// 获取当前登录用户在指定 team 中的成员信息（含 is_admin 标记）
#[derive(Debug, Serialize, Deserialize)]
pub struct GetMemberInfoReq {
    pub team_id: String,
}

#[tauri::command]
pub async fn get_member_info(payload: GetMemberInfoReq) -> Result<PoprakoMemberInfo, String> {
    info!(team_id = %payload.team_id, "Calling PopRaKo /api/v1/member/info via IPC");

    let mut defer = WarnDefer::new("poprako.member.info.request");

    #[derive(Debug, Deserialize)]
    struct Envelope<T> {
        code: u16,
        data: Option<T>,
        message: Option<String>,
    }

    use std::collections::HashMap;

    let mut q = HashMap::new();
    q.insert("team_id", payload.team_id.clone());

    let reply: Envelope<PoprakoMemberInfo> = poprako_get("members/info", Some(&q))
        .await
        .map_err(|err| format!("Failed to fetch member info: {}", err))?;

    if reply.code != 200 {
        let msg = reply.message.unwrap_or_else(|| "Unknown error".to_string());
        return Err(msg);
    }

    let info = reply
        .data
        .ok_or_else(|| "PopRaKo member info response missing data".to_string())?;

    defer.success();

    Ok(info)
}

// 获取团队活跃成员列表（包含 last_active）
// We deserialize PopRaKo's `last_active` into `time::OffsetDateTime` and
// convert it to a unix timestamp (seconds) before returning to the frontend.
#[derive(Debug, Serialize, Deserialize)]
pub struct PoprakoActiveMemberRaw {
    pub member_id: String,
    pub user_id: String,
    pub username: String,
    pub is_admin: Option<bool>,
    pub is_translator: Option<bool>,
    pub is_proofreader: Option<bool>,
    pub is_typesetter: Option<bool>,
    pub is_redrawer: Option<bool>,
    pub is_principal: Option<bool>,
    // Expect OffsetDateTime via serde (time crate with serde feature)
    pub last_active: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoprakoActiveMember {
    pub member_id: String,
    pub user_id: String,
    pub username: String,
    pub is_admin: Option<bool>,
    pub is_translator: Option<bool>,
    pub is_proofreader: Option<bool>,
    pub is_typesetter: Option<bool>,
    pub is_redrawer: Option<bool>,
    pub is_principal: Option<bool>,
    // unix timestamp (seconds) or null
    pub last_active: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetActiveMembersReq {
    pub team_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
}

#[tauri::command]
pub async fn get_active_members(
    payload: GetActiveMembersReq,
) -> Result<Vec<PoprakoActiveMember>, String> {
    info!(team_id=%payload.team_id, page=?payload.page, limit=?payload.limit, "poprako.members.active.request");

    let mut defer = WarnDefer::new("poprako.members.active.request");

    use std::collections::HashMap;

    let mut q = HashMap::new();
    q.insert("team_id", payload.team_id.clone());
    if let Some(p) = payload.page {
        q.insert("page", p.to_string());
    }
    if let Some(l) = payload.limit {
        q.insert("limit", l.to_string());
    }

    // PopRaKo returns an envelope with code/data/message for this endpoint
    let reply: PoprakoEnvelope<Vec<PoprakoActiveMemberRaw>> =
        poprako_get("members/active", Some(&q))
            .await
            .map_err(|err| format!("Failed to fetch active members: {}", err))?;

    if reply.code != 200 {
        return Err(reply.message.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let items = reply.data.unwrap_or_default();

    // Convert OffsetDateTime -> unix timestamp (seconds)
    let converted: Vec<PoprakoActiveMember> = items
        .into_iter()
        .map(|m| PoprakoActiveMember {
            member_id: m.member_id,
            user_id: m.user_id,
            username: m.username,
            is_admin: m.is_admin,
            is_translator: m.is_translator,
            is_proofreader: m.is_proofreader,
            is_typesetter: m.is_typesetter,
            is_redrawer: m.is_redrawer,
            is_principal: m.is_principal,
            last_active: m.last_active.map(|dt| dt.unix_timestamp()),
        })
        .collect();

    defer.success();

    Ok(converted)
}
