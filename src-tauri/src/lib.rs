pub mod auth;
mod http;
mod project;
mod storage;
mod team; // 汉化组相关
mod token;
mod user; // 用户与登录相关 // 项目与项目集相关

use std::{path::PathBuf, str::FromStr, sync::LazyLock};

use tracing::info;
use tracing_subscriber::fmt;

// Direct imports not strictly required; commands referenced in generate_handler by path.

const DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    dotenvy::dotenv().expect("Failed to load .end file");

    let app_dir = std::env::var("APP_DIR").unwrap_or_else(|_| "./".to_string());

    let mut path = PathBuf::from_str(&app_dir).expect("Invalid APP_DIR path");

    path.push("data");

    if !path.exists() {
        std::fs::create_dir_all(&path).expect("Failed to create data directory");
    }

    path
});

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize tracing (idempotent if called once). Use RUST_LOG for level control, default to info.
    let _ = fmt().with_target(false).compact().try_init();

    tauri::Builder::default()
        .setup(|_| {
            tauri::async_runtime::block_on(async {
                storage::LocalStorage::init(&DATA_DIR.join("local.db").to_string_lossy())
                    .await
                    .expect("Failed to initialize local storage");

                info!(
                    "Local storage initialized at {:?}",
                    DATA_DIR.join("local.db")
                );
            });

            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            // auth
            crate::auth::get_captcha,
            crate::auth::aquire_token,
            // token cache operations
            crate::token::get_moetran_token,
            crate::token::save_moetran_token,
            crate::token::remove_moetran_token,
            crate::token::get_poprako_token,
            crate::token::save_poprako_token,
            crate::token::remove_poprako_token,
            // poprako login
            crate::user::login_poprako,
            // user info
            crate::user::get_user_info,
            // user teams
            crate::team::get_user_teams,
            // project sets & projects
            crate::project::get_team_project_sets,
            crate::project::get_team_projects,
            crate::project::get_user_projects,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
