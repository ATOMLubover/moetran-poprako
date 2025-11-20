pub mod auth;

use auth::{get_captcha, request_token};
use tracing_subscriber::fmt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing (idempotent if called once). Use RUST_LOG for level control, default to info.
    let _ = fmt().with_target(false).compact().try_init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_captcha, request_token])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

thread_local! {
    // Trailing slash ensures Url::join treats path as a directory (v1/) rather than a file (v1)
    pub static MOETRAN_API_BASE: reqwest::Url = "https://api.moetran.com/v1/".parse().expect("invalid MOETRAN_API_BASE URL");
}
