//! PennyKite reverse proxy — binary entry point.
//!
//! Starts an axum HTTP server that intercepts agent requests, enforces
//! budget policy, and forwards approved requests to the upstream API.

use alloy::{primitives::Address, signers::local::PrivateKeySigner};
use anyhow::Result;
use clap::Parser;
use pennykite_kite::{AlloyKiteRpc, KiteClient};
use pennykite_providers::policy::load_policy;
use pennykite_proxy::{
    app,
    handler::{DecisionAttestor, KiteDecisionAttestor, ReqwestUpstream},
    AppState,
};
use std::{
    collections::HashMap,
    env,
    str::FromStr,
    sync::{Arc, Mutex},
};
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

    /// Private key used to sign EIP-3009 payment authorizations.
    #[arg(
        long,
        default_value = "0x59c6995e998f97a5a0044966f0945389d358f57d07535c8be9e515a7c99316c5"
    )]
    private_key: String,

    /// USDC contract address used as the EIP-712 verifying contract.
    #[arg(long, default_value = "0x036CbD53842c5426634e7929541eC2318f3dCF7e")]
    usdc_contract: String,

    /// Kite JSON-RPC URL used for on-chain attestations.
    #[arg(long)]
    kite_rpc_url: Option<String>,

    /// Kite chain ID used for on-chain attestations.
    #[arg(long)]
    kite_chain_id: Option<u64>,

    /// Deployed PennyKiteAttestor address.
    #[arg(long)]
    attestor_address: Option<String>,

    /// Private key used to sign Kite attestation transactions. Defaults to --private-key.
    #[arg(long)]
    kite_private_key: Option<String>,
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

    let policy = load_policy(&args.policy)?;
    let signer = PrivateKeySigner::from_str(&args.private_key)?;
    let usdc_contract = Address::from_str(&args.usdc_contract)?;
    let kite_attestor = build_kite_attestor(&args)?;
    let state = AppState {
        upstream: args.upstream,
        db_path: args.db,
        policy: Arc::new(policy),
        upstream_client: Arc::new(ReqwestUpstream::new()),
        ledger_lock: Arc::new(Mutex::new(())),
        loop_detectors: Arc::new(Mutex::new(HashMap::new())),
        signer,
        usdc_contract,
        kite_attestor,
    };
    let app = app(state);

    let listener = tokio::net::TcpListener::bind(&args.listen).await?;
    info!("listening on {}", args.listen);
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_kite_attestor(args: &Args) -> Result<Option<Arc<dyn DecisionAttestor>>> {
    let rpc_url = args
        .kite_rpc_url
        .clone()
        .or_else(|| env::var("KITE_RPC_URL").ok());
    let attestor_address = args
        .attestor_address
        .clone()
        .or_else(|| env::var("PENNYKITE_ATTESTOR_ADDRESS").ok());
    let private_key = args
        .kite_private_key
        .clone()
        .or_else(|| env::var("KITE_PRIVATE_KEY").ok())
        .unwrap_or_else(|| args.private_key.clone());
    let chain_id = args
        .kite_chain_id
        .or_else(|| {
            env::var("KITE_CHAIN_ID")
                .ok()
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(2368);

    let (Some(rpc_url), Some(attestor_address)) = (rpc_url, attestor_address) else {
        info!(
            "kite attestation disabled; set KITE_RPC_URL and PENNYKITE_ATTESTOR_ADDRESS to enable"
        );
        return Ok(None);
    };

    let attestor_address = Address::from_str(&attestor_address)?;
    let rpc = AlloyKiteRpc::new(
        rpc_url,
        chain_id,
        attestor_address,
        Address::ZERO,
        private_key,
    );
    Ok(Some(Arc::new(KiteDecisionAttestor::new(KiteClient::new(
        rpc,
    )))))
}
