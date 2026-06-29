use crate::models::codex_local_access::{CodexLocalAccessTestFailure, CodexLocalAccessTestResult};
use crate::modules::{codex_local_access, codex_wakeup};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, Instant};

const CODEX_MODEL_PROVIDER_TEST_TIMEOUT_SECS: u64 = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModelProviderChatTestTarget {
    pub provider_id: String,
    pub provider_name: String,
    pub base_url: String,
    pub api_key_id: Option<String>,
    pub api_key_name: Option<String>,
    pub api_key: String,
    pub wire_api: Option<String>,
    #[serde(default)]
    pub model_catalog: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModelProviderChatTestRecord {
    pub provider_id: String,
    pub provider_name: String,
    pub api_key_id: Option<String>,
    pub api_key_name: Option<String>,
    pub wire_api: String,
    pub access_mode: String,
    pub model_id: Option<String>,
    pub success: bool,
    pub prompt: String,
    pub reply: Option<String>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModelProviderChatTestBatchResult {
    pub run_id: String,
    pub records: Vec<CodexModelProviderChatTestRecord>,
    pub success_count: usize,
    pub failure_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModelProviderChatTestProgressPayload {
    pub run_id: String,
    pub total: usize,
    pub completed: usize,
    pub success_count: usize,
    pub failure_count: usize,
    pub running: bool,
    pub phase: String,
    pub current_provider_id: Option<String>,
    pub item: Option<CodexModelProviderChatTestRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModelProviderUsageDetail {
    pub key: String,
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModelProviderModel {
    pub id: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModelProviderModelsResult {
    pub models: Vec<CodexModelProviderModel>,
    pub latency_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexModelProviderUsageSummary {
    pub mode: Option<String>,
    pub is_valid: Option<bool>,
    pub status: Option<String>,
    pub plan_name: Option<String>,
    pub remaining: Option<f64>,
    pub balance: Option<f64>,
    pub unit: Option<String>,
    pub quota_unlimited: Option<bool>,
    pub quota_limit: Option<f64>,
    pub quota_used: Option<f64>,
    pub quota_remaining: Option<f64>,
    pub today_requests: Option<i64>,
    pub today_total_tokens: Option<i64>,
    pub today_cost: Option<f64>,
    pub total_requests: Option<i64>,
    pub total_total_tokens: Option<i64>,
    pub total_cost: Option<f64>,
    pub model_stats_count: usize,
    pub latency_ms: u64,
    pub details: Vec<CodexModelProviderUsageDetail>,
}

fn codex_model_provider_models_url(base_url: &str) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("PROVIDER_BASE_URL_INVALID".to_string());
    }
    let mut url =
        reqwest::Url::parse(trimmed).map_err(|_| "PROVIDER_BASE_URL_INVALID".to_string())?;
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("PROVIDER_BASE_URL_INVALID".to_string()),
    }
    let next_path = if url.path().is_empty() || url.path() == "/" {
        "/models".to_string()
    } else {
        format!("{}/models", url.path().trim_end_matches('/'))
    };
    url.set_path(&next_path);
    url.set_query(None);
    Ok(url.to_string())
}

fn normalize_model_provider_wire_api(value: Option<&str>, base_url: &str) -> String {
    match value.map(str::trim) {
        Some("chat_completions") => return "chat_completions".to_string(),
        Some("responses") => return "responses".to_string(),
        _ => {}
    }
    let lower = base_url.trim().to_ascii_lowercase();
    if lower.contains("/chat/completions")
        || lower.contains("api.deepseek.com")
        || lower.contains("api.moonshot.cn")
        || lower.contains("api.siliconflow.cn")
        || lower.contains("api.siliconflow.com")
        || lower.contains("open.bigmodel.cn")
        || lower.contains("api.z.ai")
        || lower.contains("volces.com")
        || lower.contains("bytepluses.com")
        || lower.contains("qianfan.baidubce.com")
        || lower.contains("dashscope.aliyuncs.com")
        || lower.contains("api.stepfun.com")
        || lower.contains("api.stepfun.ai")
        || lower.contains("modelscope.cn")
        || lower.contains("api.longcat.chat")
        || lower.contains("api.minimax.io")
        || lower.contains("api.mini-max.chat")
        || lower.contains("api.minimaxi.com")
        || lower.contains("api.mimo.dev")
        || lower.contains("token-plan-cn.xiaomimimo.com")
        || lower.contains("api.novita.ai")
        || lower.contains("integrate.api.nvidia.com")
        || lower.contains("runapi.co")
        || lower.contains("relaxycode.com")
        || lower.contains("compshare.cn")
        || lower.contains("api.lemondata.cc")
        || lower.contains("e-flowcode.cc")
        || lower.contains("cc-api.pipellm.ai")
        || lower.contains("openrouter.ai")
        || lower.contains("api.therouter.ai")
    {
        "chat_completions".to_string()
    } else {
        "responses".to_string()
    }
}

const RESPONSES_NATIVE_CHAT_TEST_MODEL_PRIORITY: &[&str] =
    &["gpt-5.5", "gpt-5.4", "gpt-5", "gpt-4.1", "gpt-4o"];

fn is_image_generation_model_id(model_id: &str) -> bool {
    let lower = model_id.trim().to_ascii_lowercase();
    lower.starts_with("gpt-image") || lower.starts_with("dall-e") || lower.contains("image-gen")
}

fn first_non_empty_model_id(models: &[String]) -> Option<String> {
    models
        .iter()
        .map(|item| item.trim())
        .find(|item| !item.is_empty())
        .map(ToOwned::to_owned)
}

fn select_model_provider_chat_test_model(
    wire_api: &str,
    explicit_model: Option<&str>,
    model_catalog: &[String],
) -> Option<String> {
    if let Some(model) = explicit_model
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return Some(model.to_string());
    }

    if wire_api.trim() == "responses" {
        for preferred in RESPONSES_NATIVE_CHAT_TEST_MODEL_PRIORITY {
            if let Some(model) = model_catalog
                .iter()
                .map(|item| item.trim())
                .find(|item| item.eq_ignore_ascii_case(preferred))
            {
                return Some(model.to_string());
            }
        }
        if let Some(model) = model_catalog
            .iter()
            .map(|item| item.trim())
            .find(|item| !item.is_empty() && !is_image_generation_model_id(item))
        {
            return Some(model.to_string());
        }
    }

    first_non_empty_model_id(model_catalog)
}

fn model_ids_from_provider_models(body: &Value) -> Vec<String> {
    body.get("data")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(|id| id.as_str()))
                .map(str::trim)
                .filter(|id| !id.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn first_model_from_provider_models(body: &Value, wire_api: &str) -> Option<String> {
    let models = model_ids_from_provider_models(body);
    select_model_provider_chat_test_model(wire_api, None, &models)
}

async fn discover_model_provider_model(
    client: &reqwest::Client,
    base_url: &str,
    api_key: &str,
    wire_api: &str,
) -> Option<String> {
    let url = codex_model_provider_models_url(base_url).ok()?;
    let response = client
        .get(url)
        .bearer_auth(api_key.trim())
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .ok()?;
    if !response.status().is_success() {
        return None;
    }
    let text = response.text().await.ok()?;
    let parsed = serde_json::from_str::<Value>(&text).ok()?;
    first_model_from_provider_models(&parsed, wire_api)
}

async fn run_single_model_provider_chat_test(
    client: &reqwest::Client,
    target: CodexModelProviderChatTestTarget,
    prompt: &str,
    model: Option<&str>,
    run_id: &str,
) -> CodexModelProviderChatTestRecord {
    let wire_api = normalize_model_provider_wire_api(target.wire_api.as_deref(), &target.base_url);
    let access_mode = "gateway".to_string();
    let timestamp = chrono::Utc::now().timestamp_millis();
    let api_key = target.api_key.trim().to_string();
    if api_key.is_empty() {
        return CodexModelProviderChatTestRecord {
            provider_id: target.provider_id,
            provider_name: target.provider_name,
            api_key_id: target.api_key_id,
            api_key_name: target.api_key_name,
            wire_api,
            access_mode,
            model_id: None,
            success: false,
            prompt: prompt.to_string(),
            reply: None,
            error: Some("供应商缺少 API Key".to_string()),
            duration_ms: None,
            timestamp,
        };
    }
    let configured_model_id =
        select_model_provider_chat_test_model(&wire_api, model, &target.model_catalog);
    let model_id = match configured_model_id {
        Some(model_id) => Some(model_id),
        None => discover_model_provider_model(client, &target.base_url, &api_key, &wire_api).await,
    };
    let Some(model_id) = model_id else {
        return CodexModelProviderChatTestRecord {
            provider_id: target.provider_id,
            provider_name: target.provider_name,
            api_key_id: target.api_key_id,
            api_key_name: target.api_key_name,
            wire_api,
            access_mode,
            model_id: None,
            success: false,
            prompt: prompt.to_string(),
            reply: None,
            error: Some("无法确定测试模型，请先配置模型目录或确认 /models 可用".to_string()),
            duration_ms: None,
            timestamp,
        };
    };

    let result = codex_local_access::run_model_provider_gateway_chat_test(
        codex_local_access::CodexModelProviderGatewayChatTestRequest {
            run_id: run_id.to_string(),
            provider_id: target.provider_id.clone(),
            provider_name: target.provider_name.clone(),
            base_url: target.base_url.clone(),
            api_key_id: target.api_key_id.clone(),
            api_key_name: target.api_key_name.clone(),
            api_key,
            wire_api: wire_api.clone(),
            model_catalog: target.model_catalog.clone(),
            model_id: model_id.clone(),
            prompt: prompt.to_string(),
        },
    )
    .await
    .map(|result| (result.duration_ms, result.reply));

    match result {
        Ok((duration_ms, reply)) => CodexModelProviderChatTestRecord {
            provider_id: target.provider_id,
            provider_name: target.provider_name,
            api_key_id: target.api_key_id,
            api_key_name: target.api_key_name,
            wire_api,
            access_mode,
            model_id: Some(model_id),
            success: true,
            prompt: prompt.to_string(),
            reply: Some(reply),
            error: None,
            duration_ms: Some(duration_ms),
            timestamp,
        },
        Err(error) => CodexModelProviderChatTestRecord {
            provider_id: target.provider_id,
            provider_name: target.provider_name,
            api_key_id: target.api_key_id,
            api_key_name: target.api_key_name,
            wire_api,
            access_mode,
            model_id: Some(model_id),
            success: false,
            prompt: prompt.to_string(),
            reply: None,
            error: Some(error),
            duration_ms: None,
            timestamp,
        },
    }
}

fn codex_model_provider_usage_url(base_url: &str) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("PROVIDER_BASE_URL_INVALID".to_string());
    }
    let mut url =
        reqwest::Url::parse(trimmed).map_err(|_| "PROVIDER_BASE_URL_INVALID".to_string())?;
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("PROVIDER_BASE_URL_INVALID".to_string()),
    }
    let next_path = if url.path().is_empty() || url.path() == "/" {
        "/usage".to_string()
    } else {
        format!("{}/usage", url.path().trim_end_matches('/'))
    };
    url.set_path(&next_path);
    url.set_query(None);
    Ok(url.to_string())
}

fn codex_model_provider_new_api_billing_url(
    base_url: &str,
    endpoint: &str,
) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("PROVIDER_BASE_URL_INVALID".to_string());
    }
    let mut url =
        reqwest::Url::parse(trimmed).map_err(|_| "PROVIDER_BASE_URL_INVALID".to_string())?;
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("PROVIDER_BASE_URL_INVALID".to_string()),
    }
    let base_path = url.path().trim_end_matches('/');
    let next_path = if base_path.is_empty() {
        format!("/{}", endpoint.trim_start_matches('/'))
    } else {
        format!("{}/{}", base_path, endpoint.trim_start_matches('/'))
    };
    url.set_path(&next_path);
    url.set_query(None);
    Ok(url.to_string())
}

fn codex_model_provider_new_api_api_url(base_url: &str, endpoint: &str) -> Result<String, String> {
    let trimmed = base_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("PROVIDER_BASE_URL_INVALID".to_string());
    }
    let mut url =
        reqwest::Url::parse(trimmed).map_err(|_| "PROVIDER_BASE_URL_INVALID".to_string())?;
    match url.scheme() {
        "http" | "https" => {}
        _ => return Err("PROVIDER_BASE_URL_INVALID".to_string()),
    }
    let mut base_path = url.path().trim_end_matches('/').to_string();
    if base_path == "/v1" {
        base_path.clear();
    }
    let next_path = if base_path.is_empty() {
        format!("/{}", endpoint.trim_start_matches('/'))
    } else {
        format!("{}/{}", base_path, endpoint.trim_start_matches('/'))
    };
    url.set_path(&next_path);
    url.set_query(None);
    Ok(url.to_string())
}

fn codex_model_provider_failure(
    title: &str,
    stage: &str,
    cause: String,
    suggestion: &str,
    status: Option<u16>,
    detail: Option<String>,
) -> CodexLocalAccessTestResult {
    CodexLocalAccessTestResult {
        model_id: None,
        latency_ms: None,
        output: None,
        failure: Some(CodexLocalAccessTestFailure {
            title: title.to_string(),
            stage: stage.to_string(),
            cause,
            suggestion: suggestion.to_string(),
            status,
            model_id: None,
            detail,
            gateway_output: None,
        }),
    }
}

fn summarize_model_provider_models(body: &Value) -> (Option<String>, Option<String>) {
    let ids: Vec<String> = body
        .get("data")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("id").and_then(|id| id.as_str()))
                .take(8)
                .map(|id| id.to_string())
                .collect()
        })
        .unwrap_or_default();
    let first = ids.first().cloned();
    let output = if ids.is_empty() {
        None
    } else {
        Some(ids.join(", "))
    };
    (first, output)
}

fn list_model_provider_models(body: &Value) -> Vec<CodexModelProviderModel> {
    let mut seen = std::collections::HashSet::new();
    body.get("data")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    let id = item.get("id").and_then(|id| id.as_str())?.trim();
                    if id.is_empty() {
                        return None;
                    }
                    let key = id.to_ascii_lowercase();
                    if !seen.insert(key) {
                        return None;
                    }
                    Some(CodexModelProviderModel {
                        id: id.to_string(),
                        display_name: item
                            .get("display_name")
                            .or_else(|| item.get("displayName"))
                            .and_then(|value| value.as_str())
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(str::to_string),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn json_f64_at(value: &Value, path: &[&str]) -> Option<f64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_f64()
}

fn json_i64_at(value: &Value, path: &[&str]) -> Option<i64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_i64()
}

fn json_string_at(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(|item| item.to_string())
}

fn json_bool_at(value: &Value, path: &[&str]) -> Option<bool> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_bool()
}

fn format_usage_number(value: f64) -> String {
    if value.fract().abs() < f64::EPSILON {
        format!("{:.0}", value)
    } else {
        format!("{:.4}", value)
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

fn push_usage_detail(
    details: &mut Vec<CodexModelProviderUsageDetail>,
    key: &str,
    label: &str,
    value: Option<String>,
) {
    let Some(value) = value else {
        return;
    };
    if value.trim().is_empty() {
        return;
    }
    details.push(CodexModelProviderUsageDetail {
        key: key.to_string(),
        label: label.to_string(),
        value,
    });
}

fn build_model_provider_chat_test_progress_payload(
    run_id: &str,
    total: usize,
    completed: usize,
    success_count: usize,
    failure_count: usize,
    running: bool,
    phase: &str,
    current_provider_id: Option<&str>,
    item: Option<CodexModelProviderChatTestRecord>,
) -> CodexModelProviderChatTestProgressPayload {
    CodexModelProviderChatTestProgressPayload {
        run_id: run_id.to_string(),
        total,
        completed,
        success_count,
        failure_count,
        running,
        phase: phase.to_string(),
        current_provider_id: current_provider_id.map(ToOwned::to_owned),
        item,
    }
}

pub async fn chat_test_batch<F>(
    targets: Vec<CodexModelProviderChatTestTarget>,
    prompt: Option<String>,
    model: Option<String>,
    run_id: Option<String>,
    mut emit_progress: F,
) -> Result<CodexModelProviderChatTestBatchResult, String>
where
    F: FnMut(CodexModelProviderChatTestProgressPayload) -> Result<(), String>,
{
    let cleaned_targets: Vec<CodexModelProviderChatTestTarget> = targets
        .into_iter()
        .filter(|target| {
            !target.provider_id.trim().is_empty() && !target.base_url.trim().is_empty()
        })
        .collect();
    if cleaned_targets.is_empty() {
        return Err("至少选择一个模型供应商".to_string());
    }
    let prompt = prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(codex_wakeup::DEFAULT_PROMPT)
        .to_string();
    let model = model
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned);
    let run_id = run_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let total = cleaned_targets.len();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(CODEX_MODEL_PROVIDER_TEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("CREATE_HTTP_CLIENT_FAILED: {}", e))?;

    emit_progress(build_model_provider_chat_test_progress_payload(
        &run_id,
        total,
        0,
        0,
        0,
        true,
        "batch_started",
        None,
        None,
    ))?;

    let mut records = Vec::with_capacity(total);
    let mut success_count = 0usize;
    let mut failure_count = 0usize;
    for (index, target) in cleaned_targets.into_iter().enumerate() {
        emit_progress(build_model_provider_chat_test_progress_payload(
            &run_id,
            total,
            index,
            success_count,
            failure_count,
            true,
            "provider_started",
            Some(&target.provider_id),
            None,
        ))?;
        let record = run_single_model_provider_chat_test(
            &client,
            target,
            &prompt,
            model.as_deref(),
            &run_id,
        )
        .await;
        if record.success {
            success_count += 1;
        } else {
            failure_count += 1;
        }
        emit_progress(build_model_provider_chat_test_progress_payload(
            &run_id,
            total,
            index + 1,
            success_count,
            failure_count,
            true,
            "provider_completed",
            Some(&record.provider_id),
            Some(record.clone()),
        ))?;
        records.push(record);
    }

    emit_progress(build_model_provider_chat_test_progress_payload(
        &run_id,
        total,
        total,
        success_count,
        failure_count,
        false,
        "batch_completed",
        None,
        None,
    ))?;

    Ok(CodexModelProviderChatTestBatchResult {
        run_id,
        records,
        success_count,
        failure_count,
    })
}

fn summarize_model_provider_usage(body: &Value, latency_ms: u64) -> CodexModelProviderUsageSummary {
    let model_stats_count = body
        .get("model_stats")
        .and_then(|value| value.as_array())
        .map(|items| items.len())
        .unwrap_or(0);
    let mut details = Vec::new();
    push_usage_detail(
        &mut details,
        "mode",
        "Mode",
        json_string_at(body, &["mode"]),
    );
    push_usage_detail(
        &mut details,
        "status",
        "Status",
        json_string_at(body, &["status"]),
    );
    push_usage_detail(
        &mut details,
        "planName",
        "Plan",
        json_string_at(body, &["planName"]),
    );
    push_usage_detail(
        &mut details,
        "remaining",
        "Remaining",
        json_f64_at(body, &["remaining"]).map(format_usage_number),
    );
    push_usage_detail(
        &mut details,
        "balance",
        "Balance",
        json_f64_at(body, &["balance"]).map(format_usage_number),
    );
    push_usage_detail(
        &mut details,
        "todayRequests",
        "Today Requests",
        json_i64_at(body, &["usage", "today", "requests"]).map(|value| value.to_string()),
    );
    push_usage_detail(
        &mut details,
        "todayTokens",
        "Today Tokens",
        json_i64_at(body, &["usage", "today", "total_tokens"]).map(|value| value.to_string()),
    );
    push_usage_detail(
        &mut details,
        "todayCost",
        "Today Cost",
        json_f64_at(body, &["usage", "today", "cost"]).map(format_usage_number),
    );
    push_usage_detail(
        &mut details,
        "totalRequests",
        "Total Requests",
        json_i64_at(body, &["usage", "total", "requests"]).map(|value| value.to_string()),
    );
    push_usage_detail(
        &mut details,
        "totalTokens",
        "Total Tokens",
        json_i64_at(body, &["usage", "total", "total_tokens"]).map(|value| value.to_string()),
    );
    push_usage_detail(
        &mut details,
        "totalCost",
        "Total Cost",
        json_f64_at(body, &["usage", "total", "cost"]).map(format_usage_number),
    );

    CodexModelProviderUsageSummary {
        mode: json_string_at(body, &["mode"]),
        is_valid: json_bool_at(body, &["is_active"]).or_else(|| json_bool_at(body, &["isValid"])),
        status: json_string_at(body, &["status"]),
        plan_name: json_string_at(body, &["planName"]),
        remaining: json_f64_at(body, &["remaining"]),
        balance: json_f64_at(body, &["balance"]),
        unit: json_string_at(body, &["unit"]).or_else(|| json_string_at(body, &["quota", "unit"])),
        quota_unlimited: json_bool_at(body, &["quota", "unlimited"]),
        quota_limit: json_f64_at(body, &["quota", "limit"]),
        quota_used: json_f64_at(body, &["quota", "used"]),
        quota_remaining: json_f64_at(body, &["quota", "remaining"]),
        today_requests: json_i64_at(body, &["usage", "today", "requests"]),
        today_total_tokens: json_i64_at(body, &["usage", "today", "total_tokens"]),
        today_cost: json_f64_at(body, &["usage", "today", "cost"]),
        total_requests: json_i64_at(body, &["usage", "total", "requests"]),
        total_total_tokens: json_i64_at(body, &["usage", "total", "total_tokens"]),
        total_cost: json_f64_at(body, &["usage", "total", "cost"]),
        model_stats_count,
        latency_ms,
        details,
    }
}

fn summarize_new_api_model_provider_usage(
    subscription: &Value,
    usage: &Value,
    token_usage: Option<&Value>,
    latency_ms: u64,
) -> CodexModelProviderUsageSummary {
    let raw_quota_limit = json_f64_at(subscription, &["hard_limit_usd"])
        .or_else(|| json_f64_at(subscription, &["soft_limit_usd"]))
        .or_else(|| json_f64_at(subscription, &["system_hard_limit_usd"]));
    let quota_used = json_f64_at(usage, &["total_usage"]).map(|value| value / 100.0);
    let token_data = token_usage.and_then(|value| value.get("data"));
    let quota_unlimited = token_data
        .and_then(|value| json_bool_at(value, &["unlimited_quota"]))
        .unwrap_or_else(|| {
            let hard = json_f64_at(subscription, &["hard_limit_usd"]);
            let soft = json_f64_at(subscription, &["soft_limit_usd"]);
            let system = json_f64_at(subscription, &["system_hard_limit_usd"]);
            matches!(
                (hard, soft, system),
                (Some(h), Some(s), Some(sys))
                    if (h - 100_000_000.0).abs() < f64::EPSILON
                        && (s - 100_000_000.0).abs() < f64::EPSILON
                        && (sys - 100_000_000.0).abs() < f64::EPSILON
            )
        });
    let quota_limit = if quota_unlimited {
        None
    } else {
        raw_quota_limit
    };
    let quota_remaining = match (quota_limit, quota_used) {
        (Some(limit), Some(used)) => Some((limit - used).max(0.0)),
        _ => None,
    };
    let mut details = Vec::new();
    push_usage_detail(
        &mut details,
        "hardLimitUsd",
        "Hard Limit USD",
        json_f64_at(subscription, &["hard_limit_usd"]).map(format_usage_number),
    );
    push_usage_detail(
        &mut details,
        "softLimitUsd",
        "Soft Limit USD",
        json_f64_at(subscription, &["soft_limit_usd"]).map(format_usage_number),
    );
    push_usage_detail(
        &mut details,
        "systemHardLimitUsd",
        "System Hard Limit USD",
        json_f64_at(subscription, &["system_hard_limit_usd"]).map(format_usage_number),
    );
    push_usage_detail(
        &mut details,
        "accessUntil",
        "Access Until",
        json_i64_at(subscription, &["access_until"]).map(|value| value.to_string()),
    );
    push_usage_detail(
        &mut details,
        "quotaUnlimited",
        "Unlimited Quota",
        Some(quota_unlimited.to_string()),
    );
    if let Some(token_data) = token_data {
        push_usage_detail(
            &mut details,
            "totalGranted",
            "Total Granted",
            json_f64_at(token_data, &["total_granted"]).map(format_usage_number),
        );
        push_usage_detail(
            &mut details,
            "totalAvailable",
            "Total Available",
            json_f64_at(token_data, &["total_available"]).map(format_usage_number),
        );
        push_usage_detail(
            &mut details,
            "expiresAt",
            "Expires At",
            json_i64_at(token_data, &["expires_at"]).map(|value| value.to_string()),
        );
        push_usage_detail(
            &mut details,
            "modelLimitsEnabled",
            "Model Limits",
            json_bool_at(token_data, &["model_limits_enabled"]).map(|value| value.to_string()),
        );
    }
    push_usage_detail(
        &mut details,
        "totalUsage",
        "Total Usage",
        json_f64_at(usage, &["total_usage"]).map(format_usage_number),
    );

    CodexModelProviderUsageSummary {
        mode: Some("new_api".to_string()),
        is_valid: None,
        status: None,
        plan_name: None,
        remaining: quota_remaining,
        balance: None,
        unit: Some("USD".to_string()),
        quota_unlimited: Some(quota_unlimited),
        quota_limit,
        quota_used,
        quota_remaining,
        today_requests: None,
        today_total_tokens: None,
        today_cost: None,
        total_requests: None,
        total_total_tokens: None,
        total_cost: quota_used,
        model_stats_count: 0,
        latency_ms,
        details,
    }
}

pub async fn test_connection(
    base_url: String,
    api_key: String,
    wire_api: Option<String>,
) -> Result<CodexLocalAccessTestResult, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Ok(codex_model_provider_failure(
            "missing_api_key",
            "credential",
            "MISSING_API_KEY".to_string(),
            "add_api_key",
            None,
            None,
        ));
    }

    let url = match codex_model_provider_models_url(&base_url) {
        Ok(url) => url,
        Err(error) => {
            return Ok(codex_model_provider_failure(
                "invalid_base_url",
                "url",
                error,
                "check_base_url",
                None,
                None,
            ));
        }
    };

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(CODEX_MODEL_PROVIDER_TEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("CREATE_HTTP_CLIENT_FAILED: {}", e))?;
    let started = Instant::now();
    let response = match client
        .get(&url)
        .bearer_auth(key)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
    {
        Ok(response) => response,
        Err(error) => {
            return Ok(codex_model_provider_failure(
                "network_failed",
                "network",
                error.to_string(),
                "check_network",
                None,
                Some(format!("GET {}", url)),
            ));
        }
    };
    let latency_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        let suggestion = if status == reqwest::StatusCode::UNAUTHORIZED
            || status == reqwest::StatusCode::FORBIDDEN
        {
            "check_api_key"
        } else if status == reqwest::StatusCode::NOT_FOUND {
            "check_base_url"
        } else {
            "check_provider_status"
        };
        return Ok(codex_model_provider_failure(
            "provider_http_failed",
            "models",
            "HTTP_STATUS".to_string(),
            suggestion,
            Some(status.as_u16()),
            Some(text.chars().take(1000).collect()),
        ));
    }

    let parsed = match serde_json::from_str::<Value>(&text) {
        Ok(value) => value,
        Err(error) => {
            return Ok(codex_model_provider_failure(
                "response_parse_failed",
                "parse",
                error.to_string(),
                "check_openai_compatible_models",
                Some(status.as_u16()),
                Some(text.chars().take(1000).collect()),
            ));
        }
    };
    let (model_id, output) = summarize_model_provider_models(&parsed);
    let protocol = wire_api.unwrap_or_else(|| "auto".to_string());
    Ok(CodexLocalAccessTestResult {
        model_id,
        latency_ms: Some(latency_ms),
        output: output.or_else(|| Some(format!("{} connection ok", protocol))),
        failure: None,
    })
}

pub async fn list_models(
    base_url: String,
    api_key: String,
) -> Result<CodexModelProviderModelsResult, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("MISSING_API_KEY".to_string());
    }
    let url = codex_model_provider_models_url(&base_url)?;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(CODEX_MODEL_PROVIDER_TEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("CREATE_HTTP_CLIENT_FAILED: {}", e))?;
    let started = Instant::now();
    let response = client
        .get(&url)
        .bearer_auth(key)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|e| format!("PROVIDER_MODELS_NETWORK_FAILED: {}", e))?;
    let latency_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!(
            "PROVIDER_MODELS_HTTP_{}: {}",
            status.as_u16(),
            text.chars().take(300).collect::<String>()
        ));
    }
    let parsed = serde_json::from_str::<Value>(&text)
        .map_err(|e| format!("PROVIDER_MODELS_PARSE_FAILED: {}", e))?;
    Ok(CodexModelProviderModelsResult {
        models: list_model_provider_models(&parsed),
        latency_ms,
    })
}

pub async fn query_usage(
    base_url: String,
    api_key: String,
    integration_type: Option<String>,
) -> Result<CodexModelProviderUsageSummary, String> {
    let key = api_key.trim();
    if key.is_empty() {
        return Err("MISSING_API_KEY".to_string());
    }
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(CODEX_MODEL_PROVIDER_TEST_TIMEOUT_SECS))
        .build()
        .map_err(|e| format!("CREATE_HTTP_CLIENT_FAILED: {}", e))?;

    let requested_type = integration_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    match requested_type {
        Some("new_api") => query_new_api_model_provider_usage(&client, &base_url, key).await,
        Some("sub2api") => query_sub2api_model_provider_usage(&client, &base_url, key).await,
        Some(value) => Err(format!("PROVIDER_USAGE_TYPE_UNSUPPORTED: {}", value)),
        None => {
            let new_api_error =
                match query_new_api_model_provider_usage(&client, &base_url, key).await {
                    Ok(summary) => return Ok(summary),
                    Err(error) => error,
                };
            match query_sub2api_model_provider_usage(&client, &base_url, key).await {
                Ok(summary) => Ok(summary),
                Err(sub2api_error) => Err(format!(
                    "PROVIDER_USAGE_DETECT_FAILED: new_api: {}; sub2api: {}",
                    new_api_error, sub2api_error
                )),
            }
        }
    }
}

async fn query_new_api_model_provider_usage(
    client: &reqwest::Client,
    base_url: &str,
    key: &str,
) -> Result<CodexModelProviderUsageSummary, String> {
    let subscription_url =
        codex_model_provider_new_api_billing_url(base_url, "dashboard/billing/subscription")?;
    let usage_url = codex_model_provider_new_api_billing_url(base_url, "dashboard/billing/usage")?;
    let token_usage_url = codex_model_provider_new_api_api_url(base_url, "api/usage/token/")?;
    let started = Instant::now();
    let subscription_response = client
        .get(&subscription_url)
        .bearer_auth(key)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|e| format!("PROVIDER_USAGE_NETWORK_FAILED: {}", e))?;
    let subscription_status = subscription_response.status();
    let subscription_text = subscription_response.text().await.unwrap_or_default();
    if !subscription_status.is_success() {
        return Err(format!(
            "PROVIDER_USAGE_HTTP_{}: {}",
            subscription_status.as_u16(),
            subscription_text.chars().take(300).collect::<String>()
        ));
    }
    let usage_response = client
        .get(&usage_url)
        .bearer_auth(key)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|e| format!("PROVIDER_USAGE_NETWORK_FAILED: {}", e))?;
    let latency_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    let usage_status = usage_response.status();
    let usage_text = usage_response.text().await.unwrap_or_default();
    if !usage_status.is_success() {
        return Err(format!(
            "PROVIDER_USAGE_HTTP_{}: {}",
            usage_status.as_u16(),
            usage_text.chars().take(300).collect::<String>()
        ));
    }
    let subscription = serde_json::from_str::<Value>(&subscription_text)
        .map_err(|e| format!("PROVIDER_USAGE_PARSE_FAILED: {}", e))?;
    let usage = serde_json::from_str::<Value>(&usage_text)
        .map_err(|e| format!("PROVIDER_USAGE_PARSE_FAILED: {}", e))?;
    let token_usage = match client
        .get(&token_usage_url)
        .bearer_auth(key)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
    {
        Ok(response) if response.status().is_success() => {
            let text = response.text().await.unwrap_or_default();
            serde_json::from_str::<Value>(&text).ok()
        }
        _ => None,
    };
    Ok(summarize_new_api_model_provider_usage(
        &subscription,
        &usage,
        token_usage.as_ref(),
        latency_ms,
    ))
}

async fn query_sub2api_model_provider_usage(
    client: &reqwest::Client,
    base_url: &str,
    key: &str,
) -> Result<CodexModelProviderUsageSummary, String> {
    let url = codex_model_provider_usage_url(base_url)?;
    let started = Instant::now();
    let response = client
        .get(&url)
        .bearer_auth(key)
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await
        .map_err(|e| format!("PROVIDER_USAGE_NETWORK_FAILED: {}", e))?;
    let latency_ms = started.elapsed().as_millis().try_into().unwrap_or(u64::MAX);
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!(
            "PROVIDER_USAGE_HTTP_{}: {}",
            status.as_u16(),
            text.chars().take(300).collect::<String>()
        ));
    }
    let parsed = serde_json::from_str::<Value>(&text)
        .map_err(|e| format!("PROVIDER_USAGE_PARSE_FAILED: {}", e))?;
    Ok(summarize_model_provider_usage(&parsed, latency_ms))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn models(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn responses_native_chat_test_prefers_gpt_55_over_image_model() {
        let catalog = models(&["gpt-image-2", "gpt-5.5", "gpt-5.4"]);

        assert_eq!(
            select_model_provider_chat_test_model("responses", None, &catalog).as_deref(),
            Some("gpt-5.5")
        );
    }

    #[test]
    fn responses_native_chat_test_skips_image_model_when_preferred_missing() {
        let catalog = models(&["gpt-image-2", "custom-text-model"]);

        assert_eq!(
            select_model_provider_chat_test_model("responses", None, &catalog).as_deref(),
            Some("custom-text-model")
        );
    }

    #[test]
    fn chat_completions_chat_test_keeps_catalog_order() {
        let catalog = models(&["provider-default", "gpt-5.5"]);

        assert_eq!(
            select_model_provider_chat_test_model("chat_completions", None, &catalog).as_deref(),
            Some("provider-default")
        );
    }

    #[test]
    fn explicit_chat_test_model_wins_over_responses_preference() {
        let catalog = models(&["gpt-image-2", "gpt-5.5"]);

        assert_eq!(
            select_model_provider_chat_test_model("responses", Some("custom-model"), &catalog)
                .as_deref(),
            Some("custom-model")
        );
    }
}
