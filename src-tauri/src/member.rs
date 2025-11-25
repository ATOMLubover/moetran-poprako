use serde::{Deserialize, Serialize};

use tracing::info;

use crate::{http::poprako_get, defer::WarnDefer};
use std::collections::HashMap;

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
        "Calling PopRaKo /api/v1/members via IPC",
    );

    let mut defer = WarnDefer::new("poprako.members.request");

    // 构造查询参数 Map，由 http::poprako_get 负责编码到 URL
    let mut params: HashMap<&str, String> = HashMap::new();

    params.insert("team_id", payload.team_id.clone());

    if let Some(pos) = payload.position.clone() {
        params.insert("position", pos);
    }
    if let Some(name) = payload.fuzzy_name.clone() {
        params.insert("fuzzy_name", name);
    }
    if let Some(page) = payload.page {
        params.insert("page", page.to_string());
    }
    if let Some(limit) = payload.limit {
        params.insert("limit", limit.to_string());
    }

    let reply: PoprakoEnvelope<Vec<PoprakoMemberBrief>> =
        poprako_get("/api/v1/members", Some(&params))
            .await
            .map_err(|err| format!("Failed to fetch members: {}", err))?;

    if reply.code != 200 {
        return Err(reply.message.unwrap_or_else(|| "Unknown error".to_string()));
    }

    let items = reply.data.unwrap_or_default();

    defer.success();

    Ok(MembersReply { items })
}
