// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, info};
use axum::{routing::get, routing::post, Router};
use tower_http::cors::{Any, CorsLayer};
use clap::{Parser, Subcommand};

use nautilus_server::app::process_data;
use nautilus_server::common::{get_attestation, health_check};
use nautilus_server::keys::{handle_fetch_key, handle_get_service};
use nautilus_server::ws::register_ws_routes;
use nautilus_server::{init_tracing_logger, AppState};

const DEFAULT_PORT: u16 = 3000;

/// Nautilus工具 - 服务器和CLI功能
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Command>,
}

/// 可用子命令
#[derive(Subcommand, Debug)]
enum Command {
    /// 启动Nautilus服务器（默认行为）
    Server {
        /// 服务器监听端口
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },
    
    /// 运行CLI工具
    Cli {
        #[command(subcommand)]
        cli_command: nautilus_server::cli::Command,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_logger();
    
    // 解析命令行参数
    let args = Arguments::parse();
    info!("解析命令行参数: {:?}", args);
    match args.command {
        // 如果没有指定子命令或者指定了Server子命令，启动服务器
        None | Some(Command::Server { port: _ }) => {
            info!("启动Nautilus服务器模式");
            start_server().await
        },
        
        // 如果指定了CLI子命令，运行CLI功能
        Some(Command::Cli { cli_command }) => {
            info!("启动Nautilus CLI模式");
            nautilus_server::cli::run_cli_command(cli_command).await
        }
    }
}

/// 启动服务器功能
async fn start_server() -> Result<()> {
    let mut state = AppState::new().await;
    AppState::spawn_latest_checkpoint_timestamp_updater(&mut state, None).await;
    AppState::spawn_reference_gas_price_updater(&mut state, None).await;
    
    let state_arc = Arc::new(state);
    
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
        .with_state(state_arc.clone())
        .layer(cors);
    
    // 集成WebSocket路由
    let app = register_ws_routes(app);
    
    info!("服务器启动，WebSocket功能已集成");
    
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
