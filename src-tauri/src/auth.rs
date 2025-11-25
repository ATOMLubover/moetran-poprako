use serde::{Deserialize, Serialize};

use crate::{defer::WarnDefer, http::moetran_post_opt};

// ================== Captcha 与登录 Token DTO 定义 ==================

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
// ================== 获取验证码图与验证码信息 ==================
// 说明：通过后端代理拉取验证码，避免跨域问题；返回图像与 info 标识。
#[tauri::command]
pub async fn get_captcha() -> Result<ResCaptcha, String> {
    tracing::info!("captcha.request.start");

    let mut defer = WarnDefer::new("captcha.request");

    let body = moetran_post_opt::<serde_json::Value, ResCaptcha>("captchas", None)
        .await
        .map_err(|err| format!("Captcha request failed: {}", err))?;

    tracing::info!(info = %body.info, "captcha.request.ok");

    defer.success();

    Ok(body)
}

// ================== 申请登录访问 Token ==================
// 输入：邮箱、密码、验证码及其 info；输出：用户访问 token。
#[tauri::command]
pub async fn aquire_token(payload: ReqToken) -> Result<ResToken, String> {
    tracing::info!(email = %payload.email, "token.request.start");

    let mut defer = WarnDefer::new("token.request");

    let body = moetran_post_opt::<ReqToken, ResToken>("user/token", Some(payload))
        .await
        .map_err(|err| format!("Token request failed: {}", err))?;

    tracing::info!(token_len = body.token.len(), "token.request.ok");

    defer.success();

    Ok(body)
}
