use std::time::Duration;
use tauri::AppHandle;

const FAST_LOCAL_READ_TIMEOUT: Duration = Duration::from_secs(20);

fn resolve_provider_current_account_id(platform: &str) -> Result<Option<String>, String> {
    match platform {
        "windsurf" => crate::modules::platform_adapter::call_windsurf_with_timeout(
            "accounts.current",
            serde_json::json!({}),
            FAST_LOCAL_READ_TIMEOUT,
        ),
        "kiro" => crate::modules::platform_adapter::call_kiro_with_timeout(
            "accounts.current",
            serde_json::json!({}),
            FAST_LOCAL_READ_TIMEOUT,
        ),
        "cursor" => crate::modules::platform_adapter::call_cursor_with_timeout(
            "accounts.current",
            serde_json::json!({}),
            FAST_LOCAL_READ_TIMEOUT,
        ),
        "gemini" => crate::modules::platform_adapter::call_gemini_with_timeout(
            "accounts.current",
            serde_json::json!({}),
            FAST_LOCAL_READ_TIMEOUT,
        ),
        "codebuddy" => {
            if !crate::modules::platform_package::is_platform_package_installed("codebuddy") {
                return Ok(None);
            }
            crate::modules::platform_adapter::call_codebuddy_with_timeout(
                "accounts.current",
                serde_json::json!({}),
                FAST_LOCAL_READ_TIMEOUT,
            )
        }
        "codebuddy_cn" | "codebuddy-cn" => {
            if !crate::modules::platform_package::is_platform_package_installed("codebuddy_cn") {
                return Ok(None);
            }
            crate::modules::platform_adapter::call_codebuddy_cn_with_timeout(
                "accounts.current",
                serde_json::json!({}),
                FAST_LOCAL_READ_TIMEOUT,
            )
        }
        "qoder" => {
            if !crate::modules::platform_package::is_platform_package_installed("qoder") {
                return Ok(None);
            }
            crate::modules::platform_adapter::call_qoder_with_timeout(
                "accounts.current",
                serde_json::json!({}),
                FAST_LOCAL_READ_TIMEOUT,
            )
        }
        "trae" => {
            if !crate::modules::platform_package::is_platform_package_installed("trae") {
                return Ok(None);
            }
            crate::modules::platform_adapter::call_trae_with_timeout(
                "accounts.current",
                serde_json::json!({}),
                FAST_LOCAL_READ_TIMEOUT,
            )
        }
        "workbuddy" => {
            if !crate::modules::platform_package::is_platform_package_runtime_ready("workbuddy") {
                return Ok(None);
            }
            crate::modules::platform_adapter::call_workbuddy_with_timeout(
                "accounts.current",
                serde_json::json!({}),
                FAST_LOCAL_READ_TIMEOUT,
            )
        }
        "github_copilot" | "github-copilot" | "ghcp" => {
            crate::modules::platform_adapter::call_github_copilot_with_timeout(
                "accounts.current",
                serde_json::json!({}),
                FAST_LOCAL_READ_TIMEOUT,
            )
        }
        "zed" => crate::modules::platform_adapter::call_zed_with_timeout(
            "accounts.current",
            serde_json::json!({}),
            FAST_LOCAL_READ_TIMEOUT,
        ),
        other => Err(format!("不支持的平台: {}", other)),
    }
}

#[tauri::command]
pub async fn get_provider_current_account_id(
    app: AppHandle,
    platform: String,
) -> Result<Option<String>, String> {
    let current_account_id = resolve_provider_current_account_id(platform.trim())?;
    tauri::async_runtime::spawn_blocking(move || {
        let _ = crate::modules::tray::update_tray_menu(&app);
    });
    Ok(current_account_id)
}
