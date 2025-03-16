mod agent;
mod auth;
mod config;
mod controller;
mod protocol;
mod states;
mod store;
mod tracing;

use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{Router, routing::post};
use clap::Parser;
use controller::{ask_agent, clear_history, fetch_history, init_session, test_auth};
use states::COMMAND_LINE_ARGS;
use tower_http::cors::CorsLayer;

#[derive(Parser, Debug)]
struct CommandLineArgs {
    #[arg(short = 'p', long = "port")]
    port: usize,
    #[arg(short = 'd', long = "debug")]
    debug: bool,
}

fn main() -> Result<()> {
    // --- sync part ---
    let cli = CommandLineArgs::parse();
    let _guard = tracing::init_tracing(cli.debug);

    indoc_info!("Tracing init completes.");
    indoc_info!(
        "
        Cli argument parsing completes:
        {:?}
        ",
        cli
    );

    // --- async part ---
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        indoc_info!("Async runtime starts.");
        let init_res = states::init_states(cli).await;
        store::init_chat_history_table().await;
        tokio::spawn(async {
            store::block_periodic_clear_history().await;
        });
        let service_res = root_future().await;

        match init_res.and(service_res) {
            Ok(_) => (),
            Err(e) => {
                indoc_error!(
                    "
                    Can't recover from error:
                    {}
                    Process aborts.
                    ",
                    e
                );
            }
        }
    });
    Ok(())
}

async fn root_future() -> Result<()> {
    let cli_args = COMMAND_LINE_ARGS.get().unwrap();
    let port = cli_args.port;
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .with_context(|| "tcp listen port")?;
    indoc_info!("Server listening to port {}...", port);

    let app = Router::new()
        .route("/init-session", post(init_session))
        .route("/fetch-history", post(fetch_history))
        .route("/clear-history", post(clear_history))
        .route("/ask-agent", post(ask_agent))
        .route("/test-auth", post(test_auth))
        .layer(CorsLayer::very_permissive());
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;
    Ok(())
}
