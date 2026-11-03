// SPDX-FileCopyrightText: Copyright (c) 2026, NVIDIA CORPORATION & AFFILIATES. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use crate::error::AppError;
use crate::storage::Storage;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AuthQueryParams {
    pub api_key: Option<String>,
}

/// Extract token from Authorization Bearer header, X-API-Key header, or ?api_key query parameter
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

    // 3. Query parameter ?api_key=<token> (useful for browser SSE EventSource)
    if let Some(q_tok) = query_token {
        let q_tok = q_tok.trim();
        if !q_tok.is_empty() {
            return Some(q_tok.to_string());
        }
    }

    None
}

/// Check if authentication is strictly enforced
pub fn is_auth_required() -> bool {
    std::env::var("REQUIRE_AUTH")
        .map(|v| v == "true" || v == "1")
        .unwrap_or(false)
        || std::env::var("ADMIN_API_KEY").map(|v| !v.trim().is_empty()).unwrap_or(false)
}

/// Validate request authorization against persistent storage and environment masters
pub async fn validate_request(
    headers: &HeaderMap,
    query_token: Option<&str>,
    storage: &Storage,
) -> Result<(), AppError> {
    let token_opt = extract_token(headers, query_token);

    if is_auth_required() {
        let token = match token_opt {
            Some(t) => t,
            None => {
                return Err(AppError::Auth(
                    "Authentication required. Please supply a valid API key via Authorization: Bearer <token>, X-API-Key header, or ?api_key parameter.".to_string(),
                ));
            }
        };

        let is_valid = storage
            .verify_api_key(&token)
            .await
            .map_err(|e| AppError::Storage(format!("Auth verification failed: {}", e)))?;

        if !is_valid {
            return Err(AppError::Auth("Invalid, expired, or revoked API key.".to_string()));
        }
    } else if let Some(token) = token_opt {
        // Even in open mode, if a key is provided, verify it if user asks for verification
        let is_valid = storage.verify_api_key(&token).await.unwrap_or(false);
        if !is_valid {
            tracing::warn!("Unrecognized API key provided in non-enforcing mode: '{}'", &token[0..token.len().min(7)]);
        }
    }

    Ok(())
}

/// GET /api/auth/status
/// Inspects active credentials and returns auth state for the web UI
pub async fn auth_status_handler(
    State(storage): State<Storage>,
    headers: HeaderMap,
    Query(query): Query<AuthQueryParams>,
) -> impl IntoResponse {
    let token_opt = extract_token(&headers, query.api_key.as_deref());
    let require_auth = is_auth_required();

    let (authenticated, key_prefix) = if let Some(ref token) = token_opt {
        let valid = storage.verify_api_key(token).await.unwrap_or(false);
        let prefix = if token.len() >= 7 { &token[0..7] } else { token };
        (valid, Some(prefix.to_string()))
    } else {
        (!require_auth, None)
    };

    Json(serde_json::json!({
        "authenticated": authenticated,
        "require_auth": require_auth,
        "key_prefix": key_prefix,
        "provided_token": token_opt.is_some()
    }))
}
