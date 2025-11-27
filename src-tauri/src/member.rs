use serde::{Deserialize, Serialize};

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
pub struct PoprakoMemberBrief {
    pub member_id: String,
    pub username: String,
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
    pub items: Vec<PoprakoMemberBrief>,
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

    let reply: PoprakoEnvelope<Vec<PoprakoMemberBrief>> =
        poprako_post_opt("members/search", Some(&payload))
            .await
            .map_err(|err| format!("Failed to fetch members: {}", err))?;

    if reply.code != 200 {
        return Err(reply.message.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let items = reply.data.unwrap_or_default();

    defer.success();

    Ok(MembersReply { items })
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
