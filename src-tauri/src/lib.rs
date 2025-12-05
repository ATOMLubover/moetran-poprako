pub mod auth;
mod defer;
mod http;
mod image_cache; // 图片缓存管理
mod member; // 成员搜索等相关
mod project; // 项目与项目集相关
mod result_ex;
mod storage; // 本地存储与数据目录管理
mod team; // 汉化组相关
mod token; // Token 缓存与存取
mod user; // 用户与登录相关

use std::{path::PathBuf, str::FromStr, sync::LazyLock};

use tracing::info;
use tracing_subscriber::EnvFilter;

// 直接导入模块便于 generate_handler 使用路径调用，不强制要求 pub 暴露全部

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
    // 初始化 tracing（一次性），添加 EnvFilter 方便用户通过环境变量调整日志等级：
    // 示例：RUST_LOG=debug,reqwest=warn
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .with_env_filter(filter)
        .try_init()
        .expect("Error when initializing tracing log");

    tauri::Builder::default()
        .setup(|_app| {
            // 异步初始化本地存储，避免使用 block_on 阻塞主事件循环导致 winit 顺序警告
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
            crate::user::sync_user,
            // user info
            crate::user::get_user_info,
            // user teams
            crate::team::get_user_teams,
            // projects (enriched only)
            crate::project::get_user_projects_enriched,
            crate::project::get_project_targets,
            crate::project::get_project_files,
            crate::project::get_page_sources,
            crate::project::create_source,
            crate::project::update_source,
            crate::project::delete_source,
            crate::project::submit_translation,
            crate::project::update_translation,
            crate::project::proxy_image,
            crate::project::create_projset,
            crate::project::create_proj,
            crate::project::get_team_poprako_projsets,
            crate::project::assign_member_to_proj,
            crate::project::search_user_projects_enriched,
            crate::project::search_team_projects_enriched,
            crate::project::get_team_projects_enriched,
            crate::project::update_proj_status,
            crate::project::publish_proj,
            crate::project::upload_project_file,
            crate::project::create_poprako_projset,
            crate::project::get_assignments,
            // member search
            crate::member::get_members,
            crate::member::get_member_info,
            // image cache
            crate::image_cache::check_file_cache,
            crate::image_cache::download_project_files,
            crate::image_cache::delete_file_cache,
            crate::image_cache::load_cached_file,
            crate::image_cache::get_all_cached_projects_list,
            crate::image_cache::get_cached_project_info,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
