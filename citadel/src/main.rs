// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use fastcrypto::traits::KeyPair;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info};
use http::Method;
use http::header;
use http::HeaderName;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use time::Duration;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use nautilus_server::app::process_data;
use nautilus_server::catastrophe::{
    generate_avatar, 
    handle_create_profile,
    handle_get_profile,
    handle_get_user_profile,
    register_catastrophe_routes
};
use nautilus_server::common::{get_attestation, health_check};
use nautilus_server::keys::{handle_fetch_key, handle_get_service};
use nautilus_server::ws::register_ws_routes;
use nautilus_server::{init_tracing_logger, AppState};
use nautilus_server::profile::register_profile_routes;
use nautilus_server::session_login::{auth_middleware, register_auth_routes};

const DEFAULT_PORT: u16 = 3000;

/// Nautilus tool - Server and CLI functionality
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    command: Option<Command>,
}

/// Available subcommands
#[derive(Subcommand, Debug)]
enum Command {
    /// Start Nautilus server (default behavior)
    Server {
        /// Server listening port
        #[arg(long, default_value_t = DEFAULT_PORT)]
        port: u16,
    },

    /// Run CLI tool
    Cli {
        #[command(subcommand)]
        cli_command: nautilus_server::cli::Command,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing_logger();

    // Parse command line arguments
    let args = Arguments::parse();
    info!("Parsed command line arguments: {:?}", args);
    match args.command {
        // If no command is specified or the Server command is specified, start the server
        None | Some(Command::Server { port: _ }) => {
            info!("Starting Nautilus server mode");
            start_server().await
        }

        // If a CLI command is specified, run CLI functionality
        Some(Command::Cli { cli_command }) => {
            info!("Starting Nautilus CLI mode");
            nautilus_server::cli::run_cli_command(cli_command).await
        }
    }
}

/// Start server functionality
async fn start_server() -> Result<()> {
    let mut state = AppState::new().await;
    AppState::spawn_profile_updater(&mut state, None).await;
    AppState::spawn_latest_checkpoint_timestamp_updater(&mut state, None).await;
    AppState::spawn_reference_gas_price_updater(&mut state, None).await;
    AppState::spawn_package_id_updater(&mut state, None).await;

    let state_arc = Arc::new(state);

    // Define CORS strategy
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS, Method::PUT, Method::DELETE, Method::PATCH, Method::HEAD, Method::TRACE, Method::CONNECT])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::CONTENT_TYPE,
            HeaderName::from_static("request-id"),
            HeaderName::from_static("client-sdk-type"),
            HeaderName::from_static("client-sdk-version"),
        ])
        .allow_origin([
            "http://127.0.0.1:5173".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(),
        ])
        .allow_credentials(true);

    // 创建 session store
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_expiry(Expiry::OnInactivity(Duration::days(1))); // 设置为24小时

    // Configure public routes without authentication
    let public_routes = Router::new()
        .route("/process_data", post(process_data))
        .route("/v1/fetch_key", post(handle_fetch_key))
        .route("/v1/service", get(handle_get_service))
        .route("/get_attestation", get(get_attestation));

    let public_routes = register_auth_routes(public_routes);
    let public_routes = register_profile_routes(public_routes);
    let public_routes = register_catastrophe_routes(public_routes);

    // Configure protected routes that require JWT authentication
    let protected_routes = Router::new()
        .route("/health", get(health_check))
        .route_layer(middleware::from_fn_with_state(
            state_arc.clone(),
            auth_middleware,
        ));

    // Merge routes and add layers in the correct order
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state_arc.clone());
    // Integrate WebSocket routes
    let app = register_ws_routes(app);
    
    info!("Server started, WebSocket and Profile functionality integrated");
    // integrate cors and session
    let app = app
        .layer(session_layer) // 添加 session 支持
        .layer(cors) // 添加 CORS 支持
        .layer(TraceLayer::new_for_http());
    serve(app).await
}

/// Start server
pub async fn serve(app: Router) -> Result<()> {
    debug!("listening on http://localhost:{}", DEFAULT_PORT);
    // Start server
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", DEFAULT_PORT))
        .await
        .unwrap();
    // Print cool banner
    info!("\n
 ░▒▓██████▓▒░░▒▓█▓▒░▒▓████████▓▒░▒▓██████▓▒░░▒▓███████▓▒░░▒▓████████▓▒░▒▓█▓▒░        
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░  ░▒▓█▓▒░  ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░        
░▒▓█▓▒░      ░▒▓█▓▒░  ░▒▓█▓▒░  ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░        
░▒▓█▓▒░      ░▒▓█▓▒░  ░▒▓█▓▒░  ░▒▓████████▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓██████▓▒░ ░▒▓█▓▒░        
░▒▓█▓▒░      ░▒▓█▓▒░  ░▒▓█▓▒░  ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░        
░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░  ░▒▓█▓▒░  ░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░░▒▓█▓▒░▒▓█▓▒░      ░▒▓█▓▒░        
 ░▒▓██████▓▒░░▒▓█▓▒░  ░▒▓█▓▒░  ░▒▓█▓▒░░▒▓█▓▒░▒▓███████▓▒░░▒▓████████▓▒░▒▓████████▓▒░ 
🚀 Server is ready to launch at http://localhost:{}! 🚀\n", listener.local_addr().unwrap().port()); //端口可能会变!
        // Start server
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server Launch Error: {}", e))
}
