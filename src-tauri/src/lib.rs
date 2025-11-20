pub mod auth;

use auth::get_captcha;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![get_captcha])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

thread_local! {
    pub static MOETRAN_API_BASE: reqwest::Url = "https://api.moetran.com/v1".parse().expect("invalid MOETRAN_API_BASE URL");
}
