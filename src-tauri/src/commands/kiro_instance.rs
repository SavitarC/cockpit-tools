use serde::de::DeserializeOwned;
use serde_json::{json, Value};

use crate::models::InstanceProfileView;
use crate::modules::{instance::InstanceDefaults, platform_adapter, platform_package};

fn ensure_kiro_package_installed() -> Result<(), String> {
    platform_package::ensure_platform_package_installed("kiro")
}

fn kiro_call<T: DeserializeOwned>(method: &str, payload: Value) -> Result<T, String> {
    ensure_kiro_package_installed()?;
    platform_adapter::call_kiro(method, payload)
}

async fn kiro_call_async<T>(method: &'static str, payload: Value) -> Result<T, String>
where
    T: DeserializeOwned + Send + 'static,
{
    ensure_kiro_package_installed()?;
    tauri::async_runtime::spawn_blocking(move || platform_adapter::call_kiro(method, payload))
        .await
        .map_err(|error| format!("Kiro adapter 任务失败: {}", error))?
}

#[tauri::command]
pub async fn kiro_get_instance_defaults() -> Result<InstanceDefaults, String> {
    kiro_call_async("instance.getDefaults", json!({})).await
}

#[tauri::command]
pub async fn kiro_list_instances() -> Result<Vec<InstanceProfileView>, String> {
    kiro_call_async("instance.list", json!({})).await
}

#[tauri::command]
pub async fn kiro_create_instance(
    name: String,
    user_data_dir: String,
    extra_args: Option<String>,
    bind_account_id: Option<String>,
    copy_source_instance_id: Option<String>,
    init_mode: Option<String>,
) -> Result<InstanceProfileView, String> {
    kiro_call_async(
        "instance.create",
        json!({
            "name": name,
            "userDataDir": user_data_dir,
            "extraArgs": extra_args,
            "bindAccountId": bind_account_id,
            "copySourceInstanceId": copy_source_instance_id,
            "initMode": init_mode,
        }),
    )
    .await
}

#[tauri::command]
pub async fn kiro_update_instance(
    instance_id: String,
    name: Option<String>,
    extra_args: Option<String>,
    bind_account_id: Option<Option<String>>,
    follow_local_account: Option<bool>,
) -> Result<InstanceProfileView, String> {
    kiro_call_async(
        "instance.update",
        json!({
            "instanceId": instance_id,
            "name": name,
            "extraArgs": extra_args,
            "bindAccountId": bind_account_id,
            "followLocalAccount": follow_local_account,
        }),
    )
    .await
}

#[tauri::command]
pub async fn kiro_delete_instance(instance_id: String) -> Result<(), String> {
    kiro_call_async("instance.delete", json!({ "instanceId": instance_id })).await
}

#[tauri::command]
pub async fn kiro_start_instance(instance_id: String) -> Result<InstanceProfileView, String> {
    kiro_call_async("instance.start", json!({ "instanceId": instance_id })).await
}

#[tauri::command]
pub async fn kiro_stop_instance(instance_id: String) -> Result<InstanceProfileView, String> {
    kiro_call_async("instance.stop", json!({ "instanceId": instance_id })).await
}

#[tauri::command]
pub async fn kiro_open_instance_window(instance_id: String) -> Result<(), String> {
    kiro_call("instance.openWindow", json!({ "instanceId": instance_id }))
}

#[tauri::command]
pub async fn kiro_close_all_instances() -> Result<(), String> {
    kiro_call_async("instance.closeAll", json!({})).await
}
