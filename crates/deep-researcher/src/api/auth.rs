// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::error::AppError;
use crate::storage::{hash_api_key, Storage};
use axum::{
    extract::{Query, State},
    http::{header, HeaderMap},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct AuthQueryParams {
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginPayload {
    pub password: Option<String>,
    pub username: Option<String>,
    pub key: Option<String>,
    pub passkey: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordPayload {
    pub current_password: Option<String>,
    pub new_password: String,
    pub passkey: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub authenticated: bool,
    pub token: Option<String>,
    pub expires_in: u64,
    pub message: String,
}

/// Helper to parse cookie string by key
pub fn extract_cookie_token(cookie_header: &str) -> Option<String> {
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            let key = k.trim().to_lowercase();
            if key == "auth_session" || key == "dr_token" || key == "token" || key == "idtoken" || key == "api_key" {
                let v = v.trim();
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

/// Extract token from Authorization Bearer header, X-API-Key header, Cookie, or ?api_key query parameter
pub fn extract_token(headers: &HeaderMap, query_token: Option<&str>) -> Option<String> {
    // 1. Authorization: Bearer <token>
    if let Some(auth_val) = headers.get("authorization") {
        if let Ok(auth_str) = auth_val.to_str() {
            let auth_str = auth_str.trim();
            if auth_str.to_lowercase().starts_with("bearer ") {
                let token = auth_str[7..].trim();
                if !token.is_empty() {
                    return Some(token.to_string());
                }
            } else if !auth_str.is_empty() {
                return Some(auth_str.to_string());
            }
        }
    }

    // 2. X-API-Key: <token>
    if let Some(key_val) = headers.get("x-api-key") {
        if let Ok(key_str) = key_val.to_str() {
            let key_str = key_str.trim();
            if !key_str.is_empty() {
                return Some(key_str.to_string());
            }
        }
    }

    // 3. Cookie: auth_session=<token> or idToken=<token>
    if let Some(cookie_val) = headers.get("cookie") {
        if let Ok(cookie_str) = cookie_val.to_str() {
            if let Some(tok) = extract_cookie_token(cookie_str) {
                return Some(tok);
            }
        }
    }

    // 4. Query parameter ?api_key=<token> (useful for browser SSE EventSource and WebSockets)
    if let Some(q_tok) = query_token {
        let q_tok = q_tok.trim();
        if !q_tok.is_empty() {
            return Some(q_tok.to_string());
        }
    }

    None
}

/// Retrieve configured master password from environment variables if set
pub fn get_configured_password() -> Option<String> {
    for key in &[
        "APP_PASSWORD",
        "ADMIN_PASSWORD",
        "AUTH_PASSWORD",
        "DEEP_RESEARCHER_PASSWORD",
        "PASSWORD",
    ] {
        if let Ok(val) = std::env::var(key) {
            let val = val.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Retrieve configured passkey from environment variables if set
pub fn get_configured_passkey() -> Option<String> {
    for key in &["PASSKEY", "ADMIN_PASSKEY", "MASTER_PASSKEY", "DASHBOARD_PASSKEY"] {
        if let Ok(val) = std::env::var(key) {
            let val = val.trim();
            if !val.is_empty() {
                return Some(val.to_string());
            }
        }
    }
    None
}

/// Check if password-based authentication is explicitly configured in env
pub fn is_password_protected() -> bool {
    get_configured_password().is_some() || get_configured_passkey().is_some()
}

/// Check if password or passkey protection is active (including dynamic database configuration)
pub async fn is_instance_protected(storage: &Storage) -> bool {
    if is_password_protected() {
        return true;
    }
    if let Ok(Some(hash)) = storage.get_config("app_password_hash").await {
        if !hash.trim().is_empty() {
            return true;
        }
    }
    if let Ok(Some(hash)) = storage.get_config("passkey_hash").await {
        if !hash.trim().is_empty() {
            return true;
        }
    }
    false
}

/// Constant-time SHA-256 comparison for passwords to eliminate timing attacks
pub fn verify_password_constant_time(provided: &str, expected: &str) -> bool {
    use sha2::{Digest, Sha256};
    let h1 = Sha256::digest(provided.as_bytes());
    let h2 = Sha256::digest(expected.as_bytes());
    h1 == h2
}

/// Check if authentication is strictly enforced
pub fn is_auth_required() -> bool {
    let check_flag = |key: &str| {
        std::env::var(key)
            .map(|v| {
                let v = v.trim().to_lowercase();
                v == "true" || v == "1" || v == "yes" || v == "required"
            })
            .unwrap_or(false)
    };

    let check_token = |key: &str| {
        std::env::var(key)
            .map(|v| !v.trim().is_empty())
            .unwrap_or(false)
    };

    is_password_protected()
        || check_flag("REQUIRE_AUTH")
        || check_flag("AUTH_REQUIRED")
        || check_token("ADMIN_API_KEY")
        || check_token("DEEP_RESEARCHER_API_KEY")
        || check_token("AUTH_TOKEN")
}

/// Validate credentials against environment password, passkey, dynamic database password hash, and API keys
pub async fn verify_credential_candidate(candidate: &str, storage: &Storage) -> bool {
    let candidate = candidate.trim();
    if candidate.is_empty() {
        return false;
    }

    // 1. Direct password match from env
    if let Some(expected_pw) = get_configured_password() {
        if verify_password_constant_time(candidate, &expected_pw) {
            return true;
        }
    }

    // 2. Direct passkey match from env
    if let Some(expected_pk) = get_configured_passkey() {
        if verify_password_constant_time(candidate, &expected_pk) {
            return true;
        }
    }

    // 3. Dynamic database password hash check
    let cand_hash = hash_api_key(candidate);
    if let Ok(Some(stored_hash)) = storage.get_config("app_password_hash").await {
        if verify_password_constant_time(&cand_hash, &stored_hash) {
            return true;
        }
    }

    // 4. Dynamic database passkey hash check
    if let Ok(Some(stored_pk_hash)) = storage.get_config("passkey_hash").await {
        if verify_password_constant_time(&cand_hash, &stored_pk_hash) {
            return true;
        }
    }

    // 5. Persistent API keys & master tokens in storage
    if let Ok(is_valid) = storage.verify_api_key(candidate).await {
        if is_valid {
            return true;
        }
    }

    false
}

/// Validate request authorization against persistent storage, session tokens, and passwords
pub async fn validate_request(
    headers: &HeaderMap,
    query_token: Option<&str>,
    storage: &Storage,
) -> Result<(), AppError> {
    let token_opt = extract_token(headers, query_token);
    let protected = is_instance_protected(storage).await || is_auth_required();

    if protected {
        let token = match token_opt {
            Some(t) => t,
            None => {
                let msg = if is_instance_protected(storage).await {
                    "Authentication required. Please enter password or provide credentials via Authorization header / Cookie."
                } else {
                    "Authentication required. Please supply a valid API key via Authorization: Bearer <token>, X-API-Key header, or ?api_key parameter."
                };
                return Err(AppError::Auth(msg.to_string()));
            }
        };

        if !verify_credential_candidate(&token, storage).await {
            return Err(AppError::Auth("Invalid, expired, or revoked credentials.".to_string()));
        }
    } else if let Some(token) = token_opt {
        // Even in open mode, verify credentials if provided
        let _ = storage.verify_api_key(&token).await;
    }

    Ok(())
}

/// POST /api/auth/login and /v1/auth/login
/// Authenticate via password, passkey, or key and establish a web session cookie
pub async fn login_handler(
    State(storage): State<Storage>,
    Json(payload): Json<LoginPayload>,
) -> Result<impl IntoResponse, AppError> {
    let candidate = payload
        .password
        .or(payload.passkey)
        .or(payload.key)
        .unwrap_or_default();
    let candidate = candidate.trim();

    let protected = is_instance_protected(&storage).await || is_auth_required();

    let is_valid = if protected {
        verify_credential_candidate(candidate, &storage).await
    } else {
        // Open mode allows direct login
        true
    };

    if !is_valid {
        let msg = if is_instance_protected(&storage).await {
            "Incorrect password or passkey. Please verify and try again."
        } else {
            "Invalid credentials. Please provide a valid API key or password."
        };
        return Err(AppError::Auth(msg.to_string()));
    }

    // Issue a session token recorded in persistent storage
    let session_key = storage
        .create_api_key(&format!(
            "Session Login ({})",
            chrono::Utc::now().format("%Y-%m-%d %H:%M")
        ))
        .await
        .map_err(|e| AppError::Storage(format!("Failed to initialize session: {}", e)))?;

    let cookie_val = format!(
        "auth_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
        session_key
    );

    let res = Json(serde_json::json!({
        "success": true,
        "authenticated": true,
        "token": session_key,
        "expires_in": 86400,
        "message": "Authenticated successfully"
    }));

    Ok((
        [(header::SET_COOKIE, cookie_val)],
        res,
    ))
}

/// POST /api/auth/reset-password and /v1/auth/reset-password
/// Reset or update the dashboard access password and optional passkey
pub async fn reset_password_handler(
    State(storage): State<Storage>,
    headers: HeaderMap,
    Json(payload): Json<ResetPasswordPayload>,
) -> Result<impl IntoResponse, AppError> {
    let new_pw = payload.new_password.trim();
    if new_pw.len() < 4 {
        return Err(AppError::Config(
            "New password must be at least 4 characters long.".to_string(),
        ));
    }

    let protected = is_instance_protected(&storage).await;
    if protected {
        // Check current password or existing session token
        let mut verified = false;
        if let Some(curr) = payload.current_password.as_deref() {
            if verify_credential_candidate(curr, &storage).await {
                verified = true;
            }
        }
        if !verified {
            if let Some(tok) = extract_token(&headers, None) {
                if verify_credential_candidate(&tok, &storage).await {
                    verified = true;
                }
            }
        }

        if !verified {
            return Err(AppError::Auth(
                "Current password or passkey verification failed. Access denied.".to_string(),
            ));
        }
    }

    // Persist new password hash
    let new_hash = hash_api_key(new_pw);
    storage
        .set_config("app_password_hash", &new_hash)
        .await
        .map_err(|e| AppError::Storage(format!("Failed to store new password: {}", e)))?;

    // Optional passkey update
    if let Some(pk) = payload.passkey {
        let pk = pk.trim();
        if !pk.is_empty() {
            let pk_hash = hash_api_key(pk);
            let _ = storage.set_config("passkey_hash", &pk_hash).await;
        }
    }

    // Issue refreshed session cookie
    let session_key = storage
        .create_api_key(&format!(
            "Session Password Reset ({})",
            chrono::Utc::now().format("%Y-%m-%d %H:%M")
        ))
        .await
        .map_err(|e| AppError::Storage(format!("Failed to initialize session: {}", e)))?;

    let cookie_val = format!(
        "auth_session={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=86400",
        session_key
    );

    Ok((
        [(header::SET_COOKIE, cookie_val)],
        Json(serde_json::json!({
            "success": true,
            "message": "Access password updated successfully.",
            "token": session_key
        })),
    ))
}

/// POST /api/auth/logout and /v1/auth/logout
/// Invalidate session cookie
pub async fn logout_handler() -> impl IntoResponse {
    let cookie_val = "auth_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    (
        [(header::SET_COOKIE, cookie_val)],
        Json(serde_json::json!({
            "success": true,
            "authenticated": false,
            "message": "Logged out successfully"
        })),
    )
}

/// GET /api/auth/status and /v1/auth/status
/// Inspects active credentials and returns auth state for the web UI
pub async fn auth_status_handler(
    State(storage): State<Storage>,
    headers: HeaderMap,
    Query(query): Query<AuthQueryParams>,
) -> impl IntoResponse {
    let token_opt = extract_token(&headers, query.api_key.as_deref());
    let protected = is_instance_protected(&storage).await || is_auth_required();

    let (authenticated, key_prefix) = if let Some(ref token) = token_opt {
        let valid = verify_credential_candidate(token, &storage).await;
        let prefix = if token.len() >= 7 { &token[0..7] } else { token };
        (valid, Some(prefix.to_string()))
    } else {
        (!protected, None)
    };

    let has_passkey = get_configured_passkey().is_some()
        || storage
            .get_config("passkey_hash")
            .await
            .unwrap_or(None)
            .is_some();

    let auth_mode = if protected { "password" } else { "open" };

    Json(serde_json::json!({
        "authenticated": authenticated,
        "require_auth": protected,
        "password_protected": protected,
        "has_passkey": has_passkey,
        "auth_mode": auth_mode,
        "key_prefix": key_prefix,
        "provided_token": token_opt.is_some()
    }))
}
