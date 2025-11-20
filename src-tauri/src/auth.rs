use reqwest;
use serde::Deserialize;

use crate::MOETRAN_API_BASE;

pub struct MoeUserLoginReq {
    pub email: String,
    pub password: String,
    pub captcha: String,
    pub captcha_info: String,
}

#[derive(Deserialize)]
pub struct MoeUserLoginRes {
    pub token: String,
}

/// To avoid CORS issues, we fetch the captcha token from the backend.
#[tauri::command]
pub async fn get_captcha() -> Result<String, String> {
    let url = MOETRAN_API_BASE
        .with(|base| base.clone())
        .join("/token")
        .map_err(|err| format!("Failed to parse get captcha URL: {}", err))?;

    let response = reqwest::get(url)
        .await
        .map_err(|err| format!("Failed to get captcha: {}", err))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to get captcha, status: {}",
            response.status()
        ));
    }

    let body = response
        .text()
        .await
        .map_err(|err| format!("Failed to read captcha response: {}", err))?;

    Ok(body)
}
