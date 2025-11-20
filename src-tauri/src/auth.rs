use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

use crate::MOETRAN_API_BASE;

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaResponse {
    pub image: String,
    pub info: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoeUserLoginReq {
    pub email: String,
    pub password: String,
    pub captcha: String,
    #[serde(rename = "captcha_info")]
    pub captcha_info: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoeUserLoginRes {
    pub token: String,
}

/// To avoid CORS issues, we fetch the captcha token from the backend.
#[tauri::command]
pub async fn get_captcha() -> Result<CaptchaResponse, String> {
    tracing::info!("captcha.request.start");
    let url = MOETRAN_API_BASE
        .with(|base| base.clone())
        .join("captchas")
        .map_err(|err| format!("Failed to build captcha URL: {}", err))?;
    tracing::debug!(url = %url, "captcha.request.url");

    let client = Client::new();
    let response = client
        .post(url.clone())
        .header(header::ACCEPT, "application/json, text/plain, */*")
        .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .header("Origin", "https://moetran.com")
        .header(header::REFERER, "https://moetran.com/")
        .header("Accept-Language", "zh-CN")
        .body("") // ensure Content-Length: 0
        .send()
        .await
        .map_err(|err| format!("Failed to request captcha: {}", err))?;

    if !response.status().is_success() {
        let status = response.status();

        let text = response.text().await.unwrap_or_default();

        tracing::warn!(?status, body = %text, "captcha.request.failed");

        return Err(format!("Captcha request failed: {} | {}", status, text));
    }

    let body = response
        .json::<CaptchaResponse>()
        .await
        .map_err(|err| format!("Failed to parse captcha JSON: {}", err))?;

    tracing::info!(info = %body.info, "captcha.request.ok");

    Ok(body)
}

#[tauri::command]
pub async fn request_token(payload: MoeUserLoginReq) -> Result<MoeUserLoginRes, String> {
    tracing::info!(email = %payload.email, "login.request.start");
    let url = MOETRAN_API_BASE
        .with(|base| base.clone())
        .join("user/token")
        .map_err(|err| format!("Failed to build token URL: {}", err))?;
    tracing::debug!(url = %url, email = %payload.email, "login.request.url");

    let client = Client::new();
    let response = client
        .post(url.clone())
        .header(header::ACCEPT, "application/json, text/plain, */*")
        .header(header::USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36")
        .header("Origin", "https://moetran.com")
        .header(header::REFERER, "https://moetran.com/")
        .header("Accept-Language", "zh-CN")
        .json(&payload)
        .send()
        .await
        .map_err(|err| format!("Failed to request token: {}", err))?;

    if !response.status().is_success() {
        let status = response.status();

        let text = response.text().await.unwrap_or_default();

        tracing::warn!(?status, body = %text, "login.request.failed");

        return Err(format!("Token request failed: {} | {}", status, text));
    }

    let body = response
        .json::<MoeUserLoginRes>()
        .await
        .map_err(|err| format!("Failed to parse token JSON: {}", err))?;

    tracing::info!(token_len = body.token.len(), "login.request.ok");

    Ok(body)
}
