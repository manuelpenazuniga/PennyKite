//! PennyKite reverse proxy — binary entry point.
//!
//! Starts an axum HTTP server that intercepts agent requests, enforces
//! budget policy, and forwards approved requests to the upstream API.

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "pennykite-proxy", version = "0.1.0")]
struct Args {
    /// Address to listen on.
    #[arg(long, default_value = "127.0.0.1:8787")]
    listen: String,

    /// Path to the policy YAML file.
    #[arg(long, default_value = "policy/examples/conservative.yaml")]
    policy: String,

    /// Upstream API base URL.
    #[arg(long, default_value = "http://localhost:4100")]
    upstream: String,

    /// Path to the SQLite ledger database.
    #[arg(long, default_value = "pennykite.db")]
    db: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let args = Args::parse();
    info!(
        listen = %args.listen,
        policy = %args.policy,
        upstream = %args.upstream,
        db = %args.db,
        "PennyKite proxy starting"
    );

    let app = axum::Router::new().route("/health", axum::routing::get(health));

    let listener = tokio::net::TcpListener::bind(&args.listen).await?;
    info!("listening on {}", args.listen);
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}
