// Copyright 2026 Zyvor AI Labs · https://zyvor.dev
// SPDX-License-Identifier: LicenseRef-Zyvor-Production-2.0

use anyhow::{anyhow, Result};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

pub async fn dispatch(url: &str, headers: HashMap<String, String>) -> Result<Vec<u8>> {
    let client = reqwest::Client::builder()
        .timeout(DEFAULT_REQUEST_TIMEOUT)
        .build()?;

    let mut request = client.get(url);

    for (key, value) in headers {
        request = request.header(key, value);
    }

    let response = request
        .send()
        .await
        .map_err(|e| anyhow!("Could not complete HTTP request: {}", e))?;

    if response.status() != reqwest::StatusCode::OK {
        return Err(anyhow!("Non-200 status code: {}", response.status()));
    }

    let body = response
        .bytes()
        .await
        .map_err(|e| anyhow!("Failed to read response body: {}", e))?;

    Ok(body.to_vec())
}

pub fn json_response<T: Serialize>(data: &T) -> Response {
    match serde_json::to_string(data) {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to serialize JSON: {}", e),
        )
            .into_response(),
    }
}
