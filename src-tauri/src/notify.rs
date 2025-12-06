use serde::Deserialize;
use tracing::warn;

use crate::http::poprako_get;

#[derive(Deserialize)]
struct UpdateResponse {
    data: UpdateData,
}

#[derive(Deserialize)]
struct UpdateData {
    has_update: bool,
}

#[tauri::command]
pub async fn update() -> bool {
    let result: Result<UpdateResponse, String> = poprako_get("notify/update", None).await;

    match result {
        Ok(resp) => resp.data.has_update,
        Err(e) => {
            warn!("Failed to check for app update: {}", e);
            false
        }
    }
}
