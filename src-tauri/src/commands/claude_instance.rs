use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::models::InstanceLaunchMode;
use crate::modules::{instance::InstanceDefaults, platform_adapter, platform_package};

const CLAUDE_MANAGER_PLATFORM_ID: &str = "claude_manager";

fn ensure_claude_manager_package_installed() -> Result<(), String> {
    platform_package::ensure_platform_package_installed(CLAUDE_MANAGER_PLATFORM_ID)
}

fn claude_call<T: DeserializeOwned>(method: &str, payload: Value) -> Result<T, String> {
    ensure_claude_manager_package_installed()?;
    platform_adapter::call_claude_manager(method, payload)
}

async fn claude_call_async<T>(method: &'static str, payload: Value) -> Result<T, String>
where
    T: DeserializeOwned + Send + 'static,
{
    ensure_claude_manager_package_installed()?;
    tauri::async_runtime::spawn_blocking(move || {
        platform_adapter::call_claude_manager(method, payload)
    })
    .await
    .map_err(|error| format!("Claude adapter 任务失败: {}", error))?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeInstanceProfileView {
    pub id: String,
    pub name: String,
    pub user_data_dir: String,
    pub working_dir: Option<String>,
    pub extra_args: String,
    pub bind_account_id: Option<String>,
    pub launch_mode: InstanceLaunchMode,
    pub created_at: i64,
    pub last_launched_at: Option<i64>,
    pub last_pid: Option<u32>,
    pub running: bool,
    pub initialized: bool,
    pub is_default: bool,
    pub follow_local_account: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeInstanceLaunchInfo {
    pub instance_id: String,
    pub user_data_dir: String,
    pub launch_command: String,
}

#[tauri::command]
pub async fn claude_get_instance_defaults() -> Result<InstanceDefaults, String> {
    claude_call_async("instance.getDefaults", json!({})).await
}

#[tauri::command]
pub async fn claude_list_instances() -> Result<Vec<ClaudeInstanceProfileView>, String> {
    claude_call_async("instance.list", json!({})).await
}

#[tauri::command]
pub async fn claude_create_instance(
    name: String,
    user_data_dir: String,
    working_dir: Option<String>,
    extra_args: Option<String>,
    bind_account_id: Option<String>,
    copy_source_instance_id: Option<String>,
    init_mode: Option<String>,
    launch_mode: Option<InstanceLaunchMode>,
) -> Result<ClaudeInstanceProfileView, String> {
    claude_call_async(
        "instance.create",
        json!({
            "name": name,
            "userDataDir": user_data_dir,
            "workingDir": working_dir,
            "extraArgs": extra_args,
            "bindAccountId": bind_account_id,
            "copySourceInstanceId": copy_source_instance_id,
            "initMode": init_mode,
            "launchMode": launch_mode,
        }),
    )
    .await
}

#[tauri::command]
pub async fn claude_update_instance(
    instance_id: String,
    name: Option<String>,
    working_dir: Option<String>,
    extra_args: Option<String>,
    bind_account_id: Option<Option<String>>,
    follow_local_account: Option<bool>,
    launch_mode: Option<InstanceLaunchMode>,
) -> Result<ClaudeInstanceProfileView, String> {
    claude_call_async(
        "instance.update",
        json!({
            "instanceId": instance_id,
            "name": name,
            "workingDir": working_dir,
            "extraArgs": extra_args,
            "bindAccountId": bind_account_id,
            "followLocalAccount": follow_local_account,
            "launchMode": launch_mode,
        }),
    )
    .await
}

#[tauri::command]
pub async fn claude_delete_instance(instance_id: String) -> Result<(), String> {
    claude_call_async("instance.delete", json!({ "instanceId": instance_id })).await
}

#[tauri::command]
pub async fn claude_start_instance(
    instance_id: String,
) -> Result<ClaudeInstanceProfileView, String> {
    claude_call_async("instance.start", json!({ "instanceId": instance_id })).await
}

#[tauri::command]
pub async fn claude_stop_instance(
    instance_id: String,
) -> Result<ClaudeInstanceProfileView, String> {
    claude_call_async("instance.stop", json!({ "instanceId": instance_id })).await
}

#[tauri::command]
pub async fn claude_open_instance_window(instance_id: String) -> Result<(), String> {
    claude_call("instance.openWindow", json!({ "instanceId": instance_id }))
}

#[tauri::command]
pub async fn claude_close_all_instances() -> Result<(), String> {
    claude_call_async("instance.closeAll", json!({})).await
}

#[tauri::command]
pub async fn claude_get_instance_launch_command(
    instance_id: String,
) -> Result<ClaudeInstanceLaunchInfo, String> {
    claude_call_async(
        "instance.getLaunchCommand",
        json!({ "instanceId": instance_id }),
    )
    .await
}

#[tauri::command]
pub async fn claude_execute_instance_launch_command(
    instance_id: String,
    terminal: Option<String>,
) -> Result<String, String> {
    claude_call_async(
        "instance.executeLaunchCommand",
        json!({
            "instanceId": instance_id,
            "terminal": terminal,
        }),
    )
    .await
}
