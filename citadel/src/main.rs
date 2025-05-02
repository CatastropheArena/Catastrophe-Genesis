// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::sync::Arc;
use tracing::debug;
use axum::{routing::get, routing::post, Router};
use tower_http::cors::{Any, CorsLayer};

use nautilus_server::app::process_data;
use nautilus_server::common::{get_attestation, health_check};
use nautilus_server::keys::{handle_fetch_key, handle_get_service};
use nautilus_server::AppState;

const DEFAULT_PORT: u16 = 3000;

#[tokio::main]
async fn main() -> Result<()> {
    let mut state = AppState::new().await;
    AppState::spawn_latest_checkpoint_timestamp_updater(&mut state, None).await;
    AppState::spawn_reference_gas_price_updater(&mut state, None).await;
    // 定义CORS策略
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);
    // 配置HTTP路由
    let app = Router::new()
        .route("/get_attestation", get(get_attestation))
        .route("/process_data", post(process_data))
        .route("/health", get(health_check))
        .route("/v1/fetch_key", post(handle_fetch_key))
        .route("/v1/service", get(handle_get_service))
        .with_state(Arc::new(state))
        .layer(cors);
    // 启动服务器
    serve(app).await
}

/// 启动服务器
pub async fn serve(app: Router) -> Result<()> {
    debug!("listening on http://localhost:{}", DEFAULT_PORT);
    // 启动服务器
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", DEFAULT_PORT))
        .await
        .unwrap();
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server Launch Error: {}", e))
}
