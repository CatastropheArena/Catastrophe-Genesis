// Copyright (c), Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use clap::{Parser, Subcommand};
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info};

use nautilus_server::app::process_data;
use nautilus_server::catastrophe::{
    auth_middleware, generate_avatar, 
    handle_session_token,
    handle_encrypted_session_token,
    handle_create_profile,
    handle_get_profile
};
use nautilus_server::common::{get_attestation, health_check};
use nautilus_server::keys::{handle_fetch_key, handle_get_service};
use nautilus_server::ws::register_ws_routes;
use nautilus_server::{init_tracing_logger, AppState};

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
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    // Configure public routes without authentication
    let public_routes = Router::new()
        .route("/process_data", post(process_data))
        .route("/v1/fetch_key", post(handle_fetch_key))
        .route("/v1/service", get(handle_get_service))
        .route("/get_attestation", get(get_attestation))
        .route("/auth/session_token", post(handle_session_token))
        .route("/auth/encrypted_session_token", post(handle_encrypted_session_token))
        .route("/user/avatar", get(generate_avatar))
        .route("/test/create_profile", post(handle_create_profile))
        .route("/test/get_profile", post(handle_get_profile)); 

    // Configure protected routes that require JWT authentication
    let protected_routes = Router::new()
        .route("/health", get(health_check))
        .route_layer(middleware::from_fn_with_state(
            state_arc.clone(),
            auth_middleware,
        ));

    // Merge routes
    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .with_state(state_arc.clone())
        .layer(cors);

    // Integrate WebSocket routes
    let app = register_ws_routes(app);

    info!("Server started, WebSocket functionality integrated");

    // Start server
    serve(app).await
}

/// Start server
pub async fn serve(app: Router) -> Result<()> {
    debug!("listening on http://localhost:{}", DEFAULT_PORT);
    // Start server
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", DEFAULT_PORT))
        .await
        .unwrap();
    axum::serve(listener, app)
        .await
        .map_err(|e| anyhow::anyhow!("Server Launch Error: {}", e))
}
