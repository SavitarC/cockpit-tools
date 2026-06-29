use crate::models::codex::CodexAccount;
use crate::models::github_copilot::GitHubCopilotAccount;
use crate::modules::{codex_account, codex_oauth, logger};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|item| item == &path) {
        paths.push(path);
    }
}

fn get_opencode_auth_json_path_candidates() -> Result<Vec<PathBuf>, String> {
    let mut candidates: Vec<PathBuf> = Vec::new();

    if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
        let trimmed = xdg_data_home.trim();
        if !trimmed.is_empty() {
            push_unique_path(
                &mut candidates,
                PathBuf::from(trimmed).join("opencode").join("auth.json"),
            );
        }
    }

    if let Some(home) = dirs::home_dir() {
        push_unique_path(
            &mut candidates,
            home.join(".local")
                .join("share")
                .join("opencode")
                .join("auth.json"),
        );
    }

    if let Some(data_dir) = dirs::data_dir() {
        push_unique_path(&mut candidates, data_dir.join("opencode").join("auth.json"));
    }

    if candidates.is_empty() {
        return Err("无法推断 OpenCode auth.json 路径".to_string());
    }

    Ok(candidates)
}

pub fn get_opencode_auth_json_path() -> Result<PathBuf, String> {
    let candidates = get_opencode_auth_json_path_candidates()?;
    Ok(candidates
        .first()
        .cloned()
        .ok_or_else(|| "无法推断 OpenCode auth.json 路径".to_string())?)
}

fn atomic_write(path: &PathBuf, content: &str) -> Result<(), String> {
    crate::modules::atomic_write::write_string_atomic(path, content)
        .map_err(|error| format!("写入 auth.json 失败: {}", error))
}

fn build_openai_payload(account: &CodexAccount) -> Result<serde_json::Value, String> {
    let refresh = account
        .tokens
        .refresh_token
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| "Codex refresh_token 缺失，无法同步到 OpenCode".to_string())?;
    let expires = decode_token_exp_ms(&account.tokens.access_token)
        .ok_or_else(|| "Codex access_token 缺少 exp，无法同步到 OpenCode".to_string())?;

    let mut payload = json!({
        "type": "oauth",
        "access": account.tokens.access_token,
        "refresh": refresh,
        "expires": expires,
    });

    if let Some(account_id) = account.account_id.clone() {
        payload["accountId"] = json!(account_id);
    }

    Ok(payload)
}

fn build_github_copilot_payload(
    account: &GitHubCopilotAccount,
) -> Result<serde_json::Value, String> {
    let token = account.github_access_token.trim().to_string();
    if token.is_empty() {
        return Err("GitHub Copilot access_token 缺失，无法同步到 OpenCode".to_string());
    }

    Ok(json!({
        "type": "oauth",
        "access": token,
        "refresh": token,
        "expires": 0,
    }))
}

fn decode_token_exp_ms(access_token: &str) -> Option<i64> {
    let payload = codex_account::decode_jwt_payload(access_token).ok()?;
    payload.exp.map(|exp| exp * 1000)
}

fn replace_provider_entry(provider_key: &str, payload: serde_json::Value) -> Result<(), String> {
    let auth_paths = get_opencode_auth_json_path_candidates()?;
    let target_auth_path = get_opencode_auth_json_path()?;
    let source_auth_path = auth_paths.iter().find(|path| path.exists()).cloned();

    let mut auth_json = if let Some(source_path) = source_auth_path.as_ref() {
        let content = fs::read_to_string(source_path).map_err(|error| {
            format!(
                "读取 OpenCode auth.json 失败 ({}): {}",
                source_path.display(),
                error
            )
        })?;
        serde_json::from_str::<serde_json::Value>(&content).map_err(|error| {
            format!(
                "解析 OpenCode auth.json 失败 ({}): {}",
                source_path.display(),
                error
            )
        })?
    } else {
        json!({})
    };

    if !auth_json.is_object() {
        auth_json = json!({});
    }

    if let Some(map) = auth_json.as_object_mut() {
        map.insert(provider_key.to_string(), payload);
    }

    let content = serde_json::to_string_pretty(&auth_json)
        .map_err(|error| format!("序列化 OpenCode auth.json 失败: {}", error))?;
    atomic_write(&target_auth_path, &content)?;

    for extra_path in &auth_paths {
        if extra_path == &target_auth_path || !extra_path.exists() {
            continue;
        }
        if let Err(error) = atomic_write(extra_path, &content) {
            logger::log_warn(&format!(
                "同步 OpenCode 备用 auth.json 失败 ({}): {}",
                extra_path.display(),
                error
            ));
        }
    }

    if let Some(source_path) = source_auth_path {
        if source_path != target_auth_path {
            logger::log_info(&format!(
                "OpenCode auth.json 已迁移: {} -> {}",
                source_path.display(),
                target_auth_path.display()
            ));
        }
    }

    logger::log_info(&format!(
        "已更新 OpenCode auth.json 中的 {} 记录: {}",
        provider_key,
        target_auth_path.display()
    ));
    Ok(())
}

pub fn replace_openai_entry_from_codex(account: &CodexAccount) -> Result<(), String> {
    if codex_oauth::is_token_expired(&account.tokens.access_token) {
        return Err("Codex access_token 已过期，无法同步到 OpenCode".to_string());
    }

    let openai_payload = build_openai_payload(account)?;
    replace_provider_entry("openai", openai_payload)
}

pub fn replace_github_copilot_entry_from_account(
    account: &GitHubCopilotAccount,
) -> Result<(), String> {
    let payload = build_github_copilot_payload(account)?;
    replace_provider_entry("github-copilot", payload)
}
