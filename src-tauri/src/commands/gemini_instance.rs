use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::models::InstanceProfileView;
use crate::modules::{instance::InstanceDefaults, platform_adapter, platform_package};

fn ensure_gemini_package_installed() -> Result<(), String> {
    platform_package::ensure_platform_package_installed("gemini")
}

fn gemini_call<T: DeserializeOwned>(method: &str, payload: Value) -> Result<T, String> {
    ensure_gemini_package_installed()?;
    platform_adapter::call_gemini(method, payload)
}

async fn gemini_call_async<T>(method: &'static str, payload: Value) -> Result<T, String>
where
    T: DeserializeOwned + Send + 'static,
{
    ensure_gemini_package_installed()?;
    tauri::async_runtime::spawn_blocking(move || platform_adapter::call_gemini(method, payload))
        .await
        .map_err(|error| format!("Gemini adapter 任务失败: {}", error))?
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiInstanceLaunchInfo {
    pub instance_id: String,
    pub user_data_dir: String,
    pub launch_command: String,
}

#[tauri::command]
pub async fn gemini_get_instance_defaults() -> Result<InstanceDefaults, String> {
    gemini_call_async("instance.getDefaults", json!({})).await
}

#[tauri::command]
pub async fn gemini_list_instances() -> Result<Vec<InstanceProfileView>, String> {
    gemini_call_async("instance.list", json!({})).await
}

#[tauri::command]
pub async fn gemini_create_instance(
    name: String,
    user_data_dir: String,
    working_dir: Option<String>,
    extra_args: Option<String>,
    bind_account_id: Option<String>,
    copy_source_instance_id: Option<String>,
    init_mode: Option<String>,
) -> Result<InstanceProfileView, String> {
    gemini_call_async(
        "instance.create",
        json!({
            "name": name,
            "userDataDir": user_data_dir,
            "workingDir": working_dir,
            "extraArgs": extra_args,
            "bindAccountId": bind_account_id,
            "copySourceInstanceId": copy_source_instance_id,
            "initMode": init_mode,
        }),
    )
    .await
}

#[tauri::command]
pub async fn gemini_update_instance(
    instance_id: String,
    name: Option<String>,
    working_dir: Option<String>,
    extra_args: Option<String>,
    bind_account_id: Option<Option<String>>,
    follow_local_account: Option<bool>,
) -> Result<InstanceProfileView, String> {
    gemini_call_async(
        "instance.update",
        json!({
            "instanceId": instance_id,
            "name": name,
            "workingDir": working_dir,
            "extraArgs": extra_args,
            "bindAccountId": bind_account_id,
            "followLocalAccount": follow_local_account,
        }),
    )
    .await
}

#[tauri::command]
pub async fn gemini_delete_instance(instance_id: String) -> Result<(), String> {
    gemini_call_async("instance.delete", json!({ "instanceId": instance_id })).await
}

#[tauri::command]
pub async fn gemini_start_instance(instance_id: String) -> Result<InstanceProfileView, String> {
    gemini_call_async("instance.start", json!({ "instanceId": instance_id })).await
}

#[tauri::command]
pub async fn gemini_stop_instance(instance_id: String) -> Result<InstanceProfileView, String> {
    gemini_call_async("instance.stop", json!({ "instanceId": instance_id })).await
}

#[tauri::command]
pub async fn gemini_open_instance_window(instance_id: String) -> Result<(), String> {
    gemini_call("instance.openWindow", json!({ "instanceId": instance_id }))
}

#[tauri::command]
pub async fn gemini_close_all_instances() -> Result<(), String> {
    gemini_call_async("instance.closeAll", json!({})).await
}

#[tauri::command]
pub async fn gemini_get_instance_launch_command(
    instance_id: String,
) -> Result<GeminiInstanceLaunchInfo, String> {
    gemini_call_async(
        "instance.getLaunchCommand",
        json!({ "instanceId": instance_id }),
    )
    .await
}

#[tauri::command]
pub async fn gemini_execute_instance_launch_command(
    instance_id: String,
    terminal: Option<String>,
) -> Result<String, String> {
    gemini_call_async(
        "instance.executeLaunchCommand",
        json!({ "instanceId": instance_id, "terminal": terminal }),
    )
    .await
}
