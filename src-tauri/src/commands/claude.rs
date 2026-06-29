use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::time::Duration;
use std::time::Instant;
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;

use crate::models::claude::{
    ClaudeAccount, ClaudeDesktopGatewayModelMapping, ClaudeDesktopGatewayModelsResult,
    ClaudeDesktopLoginStartResponse, ClaudeOAuthStartResponse,
};
use crate::modules::{logger, platform_adapter};

const CLAUDE_MANAGER_PLATFORM_ID: &str = "claude_manager";
const CLAUDE_FAST_LOCAL_MUTATION_TIMEOUT: Duration = Duration::from_secs(20);

fn ensure_claude_manager_runtime() -> Result<(), String> {
    crate::modules::platform_package::ensure_platform_package_installed(CLAUDE_MANAGER_PLATFORM_ID)
}

fn claude_call<T: DeserializeOwned>(method: &str, payload: Value) -> Result<T, String> {
    ensure_claude_manager_runtime()?;
    platform_adapter::call_claude_manager(method, payload)
}

async fn claude_call_async<T>(method: &'static str, payload: Value) -> Result<T, String>
where
    T: DeserializeOwned + Send + 'static,
{
    ensure_claude_manager_runtime()?;
    tauri::async_runtime::spawn_blocking(move || {
        platform_adapter::call_claude_manager(method, payload)
    })
    .await
    .map_err(|error| format!("Claude adapter 任务失败: {}", error))?
}

async fn claude_call_async_with_timeout<T>(
    method: &'static str,
    payload: Value,
    timeout: Duration,
) -> Result<T, String>
where
    T: DeserializeOwned + Send + 'static,
{
    ensure_claude_manager_runtime()?;
    tauri::async_runtime::spawn_blocking(move || {
        platform_adapter::call_claude_manager_with_timeout(method, payload, timeout)
    })
    .await
    .map_err(|error| format!("Claude adapter 任务失败: {}", error))?
}

fn update_tray_menu_in_background(app: AppHandle) {
    tauri::async_runtime::spawn_blocking(move || {
        let _ = crate::modules::tray::update_tray_menu(&app);
    });
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ClaudeSwitchResult {
    message: String,
    current_platform: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCliLaunchInfo {
    pub account_id: String,
    pub account_email: String,
    pub working_dir: String,
    pub launch_command: String,
}

#[tauri::command]
pub async fn list_claude_accounts() -> Result<Vec<ClaudeAccount>, String> {
    claude_call_async_with_timeout(
        "accounts.list",
        json!({}),
        CLAUDE_FAST_LOCAL_MUTATION_TIMEOUT,
    )
    .await
}

#[tauri::command]
pub async fn delete_claude_account(app: AppHandle, account_id: String) -> Result<(), String> {
    claude_call_async_with_timeout::<()>(
        "accounts.delete",
        json!({ "accountId": account_id }),
        CLAUDE_FAST_LOCAL_MUTATION_TIMEOUT,
    )
    .await?;
    update_tray_menu_in_background(app);
    Ok(())
}

#[tauri::command]
pub async fn delete_claude_accounts(
    app: AppHandle,
    account_ids: Vec<String>,
) -> Result<(), String> {
    claude_call_async_with_timeout::<()>(
        "accounts.deleteMany",
        json!({ "accountIds": account_ids }),
        CLAUDE_FAST_LOCAL_MUTATION_TIMEOUT,
    )
    .await?;
    update_tray_menu_in_background(app);
    Ok(())
}

#[tauri::command]
pub async fn import_claude_from_json(
    app: AppHandle,
    json_content: String,
) -> Result<Vec<ClaudeAccount>, String> {
    let accounts = claude_call_async_with_timeout(
        "accounts.importJson",
        json!({ "jsonContent": json_content }),
        CLAUDE_FAST_LOCAL_MUTATION_TIMEOUT,
    )
    .await?;
    update_tray_menu_in_background(app);
    Ok(accounts)
}

#[tauri::command]
pub async fn import_claude_api_key(
    app: AppHandle,
    api_key: String,
    account_name: Option<String>,
    api_base_url: Option<String>,
    api_provider_id: Option<String>,
    api_provider_name: Option<String>,
    api_provider_source_tag: Option<String>,
    api_provider_website: Option<String>,
    api_provider_api_key_url: Option<String>,
    api_key_field: Option<String>,
    api_model_catalog: Option<Vec<String>>,
    api_extra_env: Option<BTreeMap<String, String>>,
) -> Result<ClaudeAccount, String> {
    let account = claude_call_async(
        "accounts.importApiKey",
        json!({
            "apiKey": api_key,
            "accountName": account_name,
            "apiBaseUrl": api_base_url,
            "apiProviderId": api_provider_id,
            "apiProviderName": api_provider_name,
            "apiProviderSourceTag": api_provider_source_tag,
            "apiProviderWebsite": api_provider_website,
            "apiProviderApiKeyUrl": api_provider_api_key_url,
            "apiKeyField": api_key_field,
            "apiModelCatalog": api_model_catalog,
            "apiExtraEnv": api_extra_env,
        }),
    )
    .await?;
    update_tray_menu_in_background(app);
    Ok(account)
}

#[tauri::command]
pub async fn import_claude_desktop_gateway(
    app: AppHandle,
    api_key: String,
    account_name: Option<String>,
    api_base_url: Option<String>,
    api_provider_id: Option<String>,
    api_provider_name: Option<String>,
    api_provider_source_tag: Option<String>,
    api_provider_website: Option<String>,
    api_provider_api_key_url: Option<String>,
    api_key_field: Option<String>,
    api_model_catalog: Option<Vec<String>>,
    api_extra_env: Option<BTreeMap<String, String>>,
    auth_scheme: Option<String>,
    desktop_gateway_models: Option<Vec<String>>,
    desktop_gateway_connection_mode: Option<String>,
    desktop_gateway_upstream_models: Option<Vec<String>>,
    desktop_gateway_model_mappings: Option<Vec<ClaudeDesktopGatewayModelMapping>>,
) -> Result<ClaudeAccount, String> {
    let account = claude_call_async(
        "accounts.importDesktopGateway",
        json!({
            "apiKey": api_key,
            "accountName": account_name,
            "apiBaseUrl": api_base_url,
            "apiProviderId": api_provider_id,
            "apiProviderName": api_provider_name,
            "apiProviderSourceTag": api_provider_source_tag,
            "apiProviderWebsite": api_provider_website,
            "apiProviderApiKeyUrl": api_provider_api_key_url,
            "apiKeyField": api_key_field,
            "apiModelCatalog": api_model_catalog,
            "apiExtraEnv": api_extra_env,
            "authScheme": auth_scheme,
            "desktopGatewayModels": desktop_gateway_models,
            "desktopGatewayConnectionMode": desktop_gateway_connection_mode,
            "desktopGatewayUpstreamModels": desktop_gateway_upstream_models,
            "desktopGatewayModelMappings": desktop_gateway_model_mappings,
        }),
    )
    .await?;
    update_tray_menu_in_background(app);
    Ok(account)
}

#[tauri::command]
pub async fn update_claude_desktop_gateway(
    app: AppHandle,
    account_id: String,
    api_key: String,
    account_name: Option<String>,
    api_base_url: Option<String>,
    api_provider_id: Option<String>,
    api_provider_name: Option<String>,
    api_provider_source_tag: Option<String>,
    api_provider_website: Option<String>,
    api_provider_api_key_url: Option<String>,
    api_key_field: Option<String>,
    api_model_catalog: Option<Vec<String>>,
    api_extra_env: Option<BTreeMap<String, String>>,
    auth_scheme: Option<String>,
    desktop_gateway_models: Option<Vec<String>>,
    desktop_gateway_connection_mode: Option<String>,
    desktop_gateway_upstream_models: Option<Vec<String>>,
    desktop_gateway_model_mappings: Option<Vec<ClaudeDesktopGatewayModelMapping>>,
) -> Result<ClaudeAccount, String> {
    let account = claude_call_async(
        "accounts.updateDesktopGateway",
        json!({
            "accountId": account_id,
            "apiKey": api_key,
            "accountName": account_name,
            "apiBaseUrl": api_base_url,
            "apiProviderId": api_provider_id,
            "apiProviderName": api_provider_name,
            "apiProviderSourceTag": api_provider_source_tag,
            "apiProviderWebsite": api_provider_website,
            "apiProviderApiKeyUrl": api_provider_api_key_url,
            "apiKeyField": api_key_field,
            "apiModelCatalog": api_model_catalog,
            "apiExtraEnv": api_extra_env,
            "authScheme": auth_scheme,
            "desktopGatewayModels": desktop_gateway_models,
            "desktopGatewayConnectionMode": desktop_gateway_connection_mode,
            "desktopGatewayUpstreamModels": desktop_gateway_upstream_models,
            "desktopGatewayModelMappings": desktop_gateway_model_mappings,
        }),
    )
    .await?;
    update_tray_menu_in_background(app);
    Ok(account)
}

#[tauri::command]
pub async fn claude_desktop_gateway_list_models(
    api_key: String,
    api_base_url: String,
    auth_scheme: Option<String>,
) -> Result<ClaudeDesktopGatewayModelsResult, String> {
    claude_call_async(
        "gateway.listModels",
        json!({
            "apiKey": api_key,
            "apiBaseUrl": api_base_url,
            "authScheme": auth_scheme,
        }),
    )
    .await
}

#[tauri::command]
pub fn claude_oauth_login_prepare() -> Result<ClaudeOAuthStartResponse, String> {
    claude_call("oauth.start", json!({}))
}

#[tauri::command]
pub async fn claude_oauth_login_start(app: AppHandle) -> Result<ClaudeOAuthStartResponse, String> {
    let response: ClaudeOAuthStartResponse = claude_call_async("oauth.start", json!({})).await?;
    if let Err(error) = app
        .opener()
        .open_url(&response.verification_uri, None::<String>)
    {
        let _ = claude_call::<()>("oauth.cancel", json!({ "loginId": response.login_id }));
        return Err(format!("打开 Claude OAuth 授权页失败: {}", error));
    }
    Ok(response)
}

#[tauri::command]
pub async fn claude_oauth_login_complete(
    app: AppHandle,
    login_id: String,
    callback_or_code: String,
    email_hint: Option<String>,
) -> Result<ClaudeAccount, String> {
    let account = claude_call_async(
        "oauth.complete",
        json!({
            "loginId": login_id,
            "callbackOrCode": callback_or_code,
            "emailHint": email_hint,
        }),
    )
    .await?;
    update_tray_menu_in_background(app);
    Ok(account)
}

#[tauri::command]
pub fn claude_oauth_login_cancel(login_id: Option<String>) -> Result<(), String> {
    claude_call("oauth.cancel", json!({ "loginId": login_id }))
}

#[tauri::command]
pub async fn import_claude_cli_from_local(app: AppHandle) -> Result<ClaudeAccount, String> {
    let account = claude_call_async("accounts.importCliLocal", json!({})).await?;
    update_tray_menu_in_background(app);
    Ok(account)
}

#[tauri::command]
pub async fn claude_desktop_login_start(
    _app: AppHandle,
    progress_id: Option<String>,
) -> Result<ClaudeDesktopLoginStartResponse, String> {
    claude_call_async(
        "desktopLogin.start",
        json!({
            "progressId": progress_id,
        }),
    )
    .await
}

#[tauri::command]
pub async fn claude_desktop_login_complete(
    app: AppHandle,
    login_id: String,
    account_name: Option<String>,
) -> Result<ClaudeAccount, String> {
    let account = claude_call_async(
        "desktopLogin.complete",
        json!({
            "loginId": login_id,
            "accountName": account_name,
        }),
    )
    .await?;
    update_tray_menu_in_background(app);
    Ok(account)
}

#[tauri::command]
pub fn claude_desktop_login_cancel(login_id: Option<String>) -> Result<(), String> {
    claude_call("desktopLogin.cancel", json!({ "loginId": login_id }))
}

#[tauri::command]
pub async fn claude_open_verification_window(account_id: String) -> Result<(), String> {
    claude_call_async(
        "desktopLogin.openVerificationWindow",
        json!({ "accountId": account_id }),
    )
    .await
}

#[tauri::command]
pub fn export_claude_accounts(account_ids: Vec<String>) -> Result<String, String> {
    claude_call("accounts.export", json!({ "accountIds": account_ids }))
}

#[tauri::command]
pub async fn refresh_claude_quota(
    app: AppHandle,
    account_id: String,
) -> Result<ClaudeAccount, String> {
    let started_at = Instant::now();
    logger::log_info(&format!(
        "[Claude Command] 手动刷新账号开始: account_id={}",
        account_id
    ));

    let account: ClaudeAccount =
        claude_call_async("accounts.refresh", json!({ "accountId": account_id })).await?;
    update_tray_menu_in_background(app);
    logger::log_info(&format!(
        "[Claude Command] 刷新完成: account_id={}, email={}, elapsed={}ms",
        account.id,
        account.email,
        started_at.elapsed().as_millis()
    ));
    Ok(account)
}

#[tauri::command]
pub async fn refresh_all_claude_quotas(app: AppHandle) -> Result<i32, String> {
    let started_at = Instant::now();
    logger::log_info("[Claude Command] 批量刷新开始");
    let success_count: i32 = claude_call_async("accounts.refreshAll", json!({})).await?;
    update_tray_menu_in_background(app);
    logger::log_info(&format!(
        "[Claude Command] 批量刷新完成: success={}, elapsed={}ms",
        success_count,
        started_at.elapsed().as_millis()
    ));
    Ok(success_count)
}

#[tauri::command]
pub fn update_claude_account_tags(
    account_id: String,
    tags: Vec<String>,
) -> Result<ClaudeAccount, String> {
    claude_call(
        "accounts.updateTags",
        json!({ "accountId": account_id, "tags": tags }),
    )
}

#[tauri::command]
pub fn update_claude_account_plan(
    account_id: String,
    plan_type: Option<String>,
) -> Result<ClaudeAccount, String> {
    claude_call(
        "accounts.updatePlan",
        json!({ "accountId": account_id, "planType": plan_type }),
    )
}

#[tauri::command]
pub fn update_claude_account_note(
    account_id: String,
    note: Option<String>,
) -> Result<ClaudeAccount, String> {
    claude_call(
        "accounts.updateNote",
        json!({ "accountId": account_id, "note": note }),
    )
}

#[tauri::command]
pub fn get_claude_accounts_index_path() -> Result<String, String> {
    claude_call("accounts.indexPath", json!({}))
}

#[tauri::command]
pub fn claude_get_cli_launch_command(
    app: AppHandle,
    account_id: String,
    working_dir: String,
) -> Result<ClaudeCliLaunchInfo, String> {
    let started_at = Instant::now();
    logger::log_info(&format!(
        "[Claude CLI] 准备启动命令: account_id={}, working_dir={}",
        account_id, working_dir
    ));

    let info: ClaudeCliLaunchInfo = claude_call(
        "cli.getLaunchCommand",
        json!({
            "accountId": account_id,
            "workingDir": working_dir,
        }),
    )?;
    update_tray_menu_in_background(app);

    logger::log_info(&format!(
        "[Claude CLI] 启动命令已准备: account_id={}, email={}, elapsed={}ms",
        info.account_id,
        info.account_email,
        started_at.elapsed().as_millis()
    ));

    Ok(info)
}

#[tauri::command]
pub fn claude_execute_cli_launch_command(
    app: AppHandle,
    account_id: String,
    working_dir: String,
    terminal: Option<String>,
) -> Result<String, String> {
    let started_at = Instant::now();
    logger::log_info(&format!(
        "[Claude CLI] 开始终端执行: account_id={}, working_dir={}",
        account_id, working_dir
    ));

    let result: String = claude_call(
        "cli.executeLaunchCommand",
        json!({
            "accountId": account_id,
            "workingDir": working_dir,
            "terminal": terminal,
        }),
    )?;
    update_tray_menu_in_background(app);

    logger::log_info(&format!(
        "[Claude CLI] 终端执行完成: elapsed={}ms",
        started_at.elapsed().as_millis()
    ));
    Ok(result)
}

#[tauri::command]
pub fn claude_launch_cli(
    app: AppHandle,
    account_id: String,
    working_dir: String,
    terminal: Option<String>,
) -> Result<String, String> {
    let started_at = Instant::now();
    logger::log_info(&format!(
        "[Claude CLI] 开始启动: account_id={}, working_dir={}",
        account_id, working_dir
    ));

    let result: String = claude_call(
        "cli.executeLaunchCommand",
        json!({
            "accountId": account_id,
            "workingDir": working_dir,
            "terminal": terminal,
        }),
    )?;
    update_tray_menu_in_background(app);

    logger::log_info(&format!(
        "[Claude CLI] 启动完成: elapsed={}ms",
        started_at.elapsed().as_millis()
    ));
    Ok(result)
}

#[tauri::command]
pub fn switch_claude_account(app: AppHandle, account_id: String) -> Result<String, String> {
    let started_at = Instant::now();
    logger::log_info(&format!(
        "[Claude Switch] 开始切换账号: account_id={}",
        account_id
    ));

    let result: ClaudeSwitchResult =
        claude_call("switch.inject", json!({ "accountId": account_id }))?;
    let _ = crate::modules::tray::update_tray_menu(&app);

    logger::log_info(&format!(
        "[Claude Switch] 切号成功: current_platform={}, elapsed={}ms",
        result.current_platform,
        started_at.elapsed().as_millis()
    ));
    Ok(result.message)
}
