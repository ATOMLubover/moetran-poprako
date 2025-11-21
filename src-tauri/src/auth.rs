use serde::{Deserialize, Serialize};

use crate::http::moetran_post_opt;

#[derive(Debug, Serialize, Deserialize)]
pub struct ResCaptcha {
    pub image: String,
    pub info: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReqToken {
    pub email: String,
    pub password: String,
    pub captcha: String,
    #[serde(rename = "captcha_info")]
    pub captcha_info: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResToken {
    pub token: String,
}

/// To avoid CORS issues, we fetch the captcha token from the backend.
#[tauri::command]
pub async fn get_captcha() -> Result<ResCaptcha, String> {
    tracing::info!("captcha.request.start");

    let body = moetran_post_opt::<serde_json::Value, ResCaptcha>("captchas", None)
        .await
        .map_err(|err| format!("Captcha request failed: {}", err))?;

    tracing::info!(info = %body.info, "captcha.request.ok");

    Ok(body)
}

#[tauri::command]
pub async fn aquire_token(payload: ReqToken) -> Result<ResToken, String> {
    tracing::info!(email = %payload.email, "token.request.start");

    let body = moetran_post_opt::<ReqToken, ResToken>("user/token", Some(payload))
        .await
        .map_err(|err| format!("Token request failed: {}", err))?;

    tracing::info!(token_len = body.token.len(), "token.request.ok");

    Ok(body)
}
