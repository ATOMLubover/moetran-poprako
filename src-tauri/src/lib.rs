pub mod auth;
mod http;
mod project;
mod storage;
mod team; // 汉化组相关
mod token;
mod user; // 用户与登录相关 // 项目与项目集相关

use std::{path::PathBuf, str::FromStr, sync::LazyLock};

use tracing::info;
use tracing_subscriber::EnvFilter;

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
    // 初始化 tracing（一次性），添加 EnvFilter 方便用户手动调节日志等级：
    // 例如设置 RUST_LOG=debug,reqwest=warn
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .with_env_filter(filter)
        .try_init()
        .expect("Error when initializing tracing log");

    tauri::Builder::default()
        .setup(|_app| {
            // 不再使用 block_on 阻塞主事件循环，避免 winit 事件顺序异常警告
            // 在后台异步初始化本地存储
            tauri::async_runtime::spawn(async {
                match storage::LocalStorage::init(&DATA_DIR.join("local.db").to_string_lossy())
                    .await
                {
                    Ok(_) => info!(
                        "Local storage initialized at {:?}",
                        DATA_DIR.join("local.db")
                    ),
                    Err(err) => tracing::error!(%err, "Local storage init failed"),
                }
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
