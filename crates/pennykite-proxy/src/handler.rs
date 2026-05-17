use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use alloy::{
    primitives::{Address, B256, U256},
    signers::local::PrivateKeySigner,
};
use async_trait::async_trait;
use axum::{
    body::{Body, Bytes},
    extract::{OriginalUri, State},
    http::{HeaderMap, HeaderName, HeaderValue, Method, Response, StatusCode, Uri},
    routing::{any, get},
    Json, Router,
};
use chrono::Utc;
use pennykite_cost::{amount_to_usd, select_requirement};
use pennykite_detect::{LoopDetector, RequestFingerprint};
use pennykite_ledger::{Ledger, LedgerError};
use pennykite_providers::{
    eip3009::{sign_transfer_with_authorization, TransferWithAuthorization},
    x402::parse_payment_required,
};
use pennykite_types::{Decision, PaymentRequirement, Policy, Verdict};
use serde::Serialize;
use serde_json::json;
use sha3::{Digest, Sha3_256};
use tracing::info;
use uuid::Uuid;

const PAYMENT_REQUIRED: &str = "payment-required";
const PAYMENT_SIGNATURE: &str = "payment-signature";
const SESSION_HEADER: &str = "x-pennykite-session";

#[derive(Clone)]
pub struct AppState {
    pub upstream: String,
    pub db_path: String,
    pub policy: Arc<Policy>,
    pub upstream_client: Arc<dyn UpstreamClient>,
    pub ledger_lock: Arc<Mutex<()>>,
    pub loop_detectors: Arc<Mutex<HashMap<String, LoopDetector>>>,
    pub signer: PrivateKeySigner,
    pub usdc_contract: Address,
}

pub struct UpstreamResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Bytes,
}

#[async_trait]
pub trait UpstreamClient: Send + Sync {
    async fn send(
        &self,
        method: &Method,
        url: &str,
        headers: &HeaderMap,
        body: Bytes,
        payment_signature: Option<String>,
    ) -> Result<UpstreamResponse, String>;
}

#[derive(Clone, Default)]
pub struct ReqwestUpstream {
    client: reqwest::Client,
}

impl ReqwestUpstream {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl UpstreamClient for ReqwestUpstream {
    async fn send(
        &self,
        method: &Method,
        url: &str,
        headers: &HeaderMap,
        body: Bytes,
        payment_signature: Option<String>,
    ) -> Result<UpstreamResponse, String> {
        let method = reqwest::Method::from_bytes(method.as_str().as_bytes())
            .map_err(|err| err.to_string())?;
        let mut request = self.client.request(method, url).body(body);
        for (name, value) in headers {
            if should_forward_header(name) {
                request = request.header(name.as_str(), value.as_bytes());
            }
        }
        if let Some(signature) = payment_signature {
            request = request.header(PAYMENT_SIGNATURE, signature);
        }
        let response = request.send().await.map_err(|err| err.to_string())?;
        let status =
            StatusCode::from_u16(response.status().as_u16()).map_err(|err| err.to_string())?;
        let mut response_headers = HeaderMap::new();
        for (name, value) in response.headers() {
            if should_return_header(name.as_str()) {
                response_headers.insert(
                    HeaderName::from_bytes(name.as_str().as_bytes())
                        .map_err(|err| err.to_string())?,
                    HeaderValue::from_bytes(value.as_bytes()).map_err(|err| err.to_string())?,
                );
            }
        }
        let body = response.bytes().await.map_err(|err| err.to_string())?;
        Ok(UpstreamResponse {
            status,
            headers: response_headers,
            body,
        })
    }
}

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/health", axum::routing::get(health))
        .route("/api/decisions", get(decisions_feed))
        .route("/proxy/{*path}", any(proxy))
        .route("/{*path}", any(proxy))
        .with_state(state)
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({"status": "ok"}))
}

#[derive(Serialize)]
struct DecisionFeed {
    summary: FeedSummary,
    decisions: Vec<Decision>,
}

#[derive(Serialize)]
struct FeedSummary {
    session_id: String,
    budget_usd: f64,
    spent_usd: f64,
    decisions_count: usize,
}

async fn decisions_feed(State(state): State<AppState>) -> Response<Body> {
    match load_decision_feed(&state) {
        Ok(feed) => json_response(StatusCode::OK, json!(feed)),
        Err(err) => json_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({"error": "feed_error", "reason": err.to_string()}),
        ),
    }
}

fn load_decision_feed(state: &AppState) -> Result<DecisionFeed, LedgerError> {
    let _guard = state.ledger_lock.lock().expect("ledger lock poisoned");
    let ledger = Ledger::open(&state.db_path)?;
    let decisions = ledger.recent_decisions(100)?;
    let (budget, spent, decisions_count) = ledger.feed_totals()?;
    Ok(DecisionFeed {
        summary: FeedSummary {
            session_id: "all-sessions".into(),
            budget_usd: if budget > 0.0 {
                budget
            } else {
                state.policy.session.budget_usd
            },
            spent_usd: spent,
            decisions_count,
        },
        decisions,
    })
}

async fn proxy(
    State(state): State<AppState>,
    method: Method,
    OriginalUri(uri): OriginalUri,
    headers: HeaderMap,
    body: Bytes,
) -> Response<Body> {
    match proxy_inner(state, method, uri, headers, body).await {
        Ok(response) => response,
        Err(err) => json_response(
            StatusCode::BAD_GATEWAY,
            json!({"error": "proxy_error", "reason": err}),
        ),
    }
}

async fn proxy_inner(
    state: AppState,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<Body>, String> {
    let session_id = session_id(&headers, &state.policy);
    let path_and_query = upstream_path_and_query(&uri);
    let upstream_url = format!("{}{}", state.upstream.trim_end_matches('/'), path_and_query);
    if observe_loop(&state, &session_id, &method, &path_and_query, &body) {
        let decision = record_decision(
            &state,
            &session_id,
            &request_key(&method, &uri),
            0.0,
            Verdict::DenyLoop,
            "loop detected",
        )?;
        return Ok(deny_response(&decision));
    }

    let first = state
        .upstream_client
        .send(&method, &upstream_url, &headers, body.clone(), None)
        .await?;

    if first.status != StatusCode::PAYMENT_REQUIRED {
        return response_from_upstream(first, None).await;
    }

    let Some(header) = first.headers.get(PAYMENT_REQUIRED) else {
        return Err("upstream 402 missing PAYMENT-REQUIRED header".into());
    };
    let header = header
        .to_str()
        .map_err(|_| "PAYMENT-REQUIRED header is not valid UTF-8".to_string())?;
    let payment = parse_payment_required(header).map_err(|err| err.to_string())?;
    let Some(requirement) = select_requirement(
        &payment.requirements,
        &state.policy.networks.allowed,
        &state.policy.networks.assets,
    ) else {
        let decision = record_decision(
            &state,
            &session_id,
            &request_key(&method, &uri),
            0.0,
            Verdict::DenyNetwork,
            "no allowed x402 payment requirement",
        )?;
        return Ok(deny_response(&decision));
    };

    let estimated_cost =
        amount_to_usd(&requirement.amount, requirement.decimals).map_err(|err| err.to_string())?;

    if estimated_cost > state.policy.session.per_request_cap_usd {
        let decision = record_decision(
            &state,
            &session_id,
            &request_key(&method, &uri),
            estimated_cost,
            Verdict::DenyBudget,
            "per-request cap exceeded",
        )?;
        return Ok(deny_response(&decision));
    }

    reserve(&state, &session_id, estimated_cost).map_err(|err| err.to_string())?;
    let approved =
        try_reserve_approved(&state, &session_id, estimated_cost).map_err(|err| err.to_string())?;
    if !approved {
        let decision = record_decision(
            &state,
            &session_id,
            &request_key(&method, &uri),
            estimated_cost,
            Verdict::DenyBudget,
            "session budget exceeded",
        )?;
        return Ok(deny_response(&decision));
    }

    let payment_signature = payment_signature(&state, requirement)?;
    let upstream = state
        .upstream_client
        .send(
            &method,
            &upstream_url,
            &headers,
            body,
            Some(payment_signature),
        )
        .await?;
    let decision = record_decision(
        &state,
        &session_id,
        &request_key(&method, &uri),
        estimated_cost,
        Verdict::Approve,
        "approved",
    )?;

    response_from_upstream(upstream, Some((estimated_cost, decision.id))).await
}

async fn response_from_upstream(
    upstream: UpstreamResponse,
    metadata: Option<(f64, Uuid)>,
) -> Result<Response<Body>, String> {
    let mut builder = Response::builder().status(upstream.status);
    let mut content_type = None;
    for (name, value) in &upstream.headers {
        if metadata.is_some() && name.as_str() == "content-type" {
            content_type = Some(value.clone());
            continue;
        }
        builder = builder.header(name.as_str(), value.as_bytes());
    }
    let mut bytes = upstream.body;
    if let Some((cost, decision_id)) = metadata {
        builder = builder
            .header("x-pennykite-cost-usd", cost.to_string())
            .header("x-pennykite-decision-id", decision_id.to_string());
        if let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(object) = value.as_object_mut() {
                object.insert("_pk_cost_usd".into(), json!(cost));
                object.insert("_pk_decision_id".into(), json!(decision_id.to_string()));
                bytes = Bytes::from(serde_json::to_vec(&value).map_err(|err| err.to_string())?);
                builder = builder.header("content-type", "application/json");
            } else if let Some(value) = content_type {
                builder = builder.header("content-type", value);
            }
        } else if let Some(value) = content_type {
            builder = builder.header("content-type", value);
        }
    }
    builder
        .body(Body::from(bytes))
        .map_err(|err| err.to_string())
}

fn payment_signature(state: &AppState, requirement: &PaymentRequirement) -> Result<String, String> {
    let chain_id = parse_chain_id(&requirement.network)?;
    let to = Address::from_str(&requirement.pay_to).map_err(|err| err.to_string())?;
    let value = U256::from_str(&requirement.amount).map_err(|err| err.to_string())?;
    let now = unix_timestamp();
    let authorization = TransferWithAuthorization {
        from: state.signer.address(),
        to,
        value,
        valid_after: U256::from(now),
        valid_before: U256::from(now + 300),
        nonce: nonce(),
    };
    let signature = sign_transfer_with_authorization(
        authorization,
        chain_id,
        state.usdc_contract,
        &state.signer,
    )
    .map_err(|err| err.to_string())?;
    Ok(format!("0x{}", hex::encode(signature.to_bytes())))
}

fn reserve(state: &AppState, session_id: &str, _estimated_cost: f64) -> Result<(), LedgerError> {
    let _guard = state.ledger_lock.lock().expect("ledger lock poisoned");
    let ledger = Ledger::open(&state.db_path)?;
    ledger.ensure_session(session_id, state.policy.session.budget_usd)
}

fn try_reserve_approved(
    state: &AppState,
    session_id: &str,
    estimated_cost: f64,
) -> Result<bool, LedgerError> {
    let _guard = state.ledger_lock.lock().expect("ledger lock poisoned");
    let ledger = Ledger::open(&state.db_path)?;
    ledger.try_reserve(session_id, estimated_cost)
}

fn record_decision(
    state: &AppState,
    session_id: &str,
    request_key: &str,
    estimated_cost_usd: f64,
    verdict: Verdict,
    reason: &str,
) -> Result<Decision, String> {
    let mut decision = Decision {
        id: Uuid::new_v4(),
        session_id: session_id.into(),
        timestamp: Utc::now(),
        request_key: request_key.into(),
        estimated_cost_usd,
        verdict,
        reason: reason.into(),
        decision_hash: String::new(),
        kite_attestation_tx: None,
    };
    decision.decision_hash = decision_hash(&decision)?;
    let _guard = state.ledger_lock.lock().expect("ledger lock poisoned");
    let ledger = Ledger::open(&state.db_path).map_err(|err| err.to_string())?;
    ledger
        .ensure_session(session_id, state.policy.session.budget_usd)
        .map_err(|err| err.to_string())?;
    ledger
        .record_decision(&decision)
        .map_err(|err| err.to_string())?;
    info!(
        session_id,
        verdict = ?decision.verdict,
        estimated_cost_usd,
        "proxy decision"
    );
    Ok(decision)
}

fn decision_hash(decision: &Decision) -> Result<String, String> {
    let canonical = json!({
        "id": decision.id,
        "session_id": decision.session_id,
        "timestamp": decision.timestamp,
        "request_key": decision.request_key,
        "estimated_cost_usd": decision.estimated_cost_usd,
        "verdict": decision.verdict,
        "reason": decision.reason,
        "kite_attestation_tx": decision.kite_attestation_tx,
    });
    let bytes = serde_json::to_vec(&canonical).map_err(|err| err.to_string())?;
    let hash = Sha3_256::digest(bytes);
    Ok(format!("0x{}", hex::encode(hash)))
}

fn observe_loop(
    state: &AppState,
    session_id: &str,
    method: &Method,
    path: &str,
    body: &Bytes,
) -> bool {
    let fingerprint = RequestFingerprint {
        method: method.to_string(),
        host: state.upstream.clone(),
        path: path.into(),
        body_hash: hash_bytes(body),
    };
    let mut detectors = state
        .loop_detectors
        .lock()
        .expect("loop detector lock poisoned");
    let detector = detectors
        .entry(session_id.to_string())
        .or_insert_with(|| LoopDetector::new(state.policy.loop_detection.clone()));
    detector.observe(fingerprint)
}

fn hash_bytes(bytes: &[u8]) -> String {
    let hash = Sha3_256::digest(bytes);
    format!("0x{}", hex::encode(hash))
}

fn deny_response(decision: &Decision) -> Response<Body> {
    json_response(
        StatusCode::PAYMENT_REQUIRED,
        json!({
            "reason": decision.reason,
            "verdict": decision.verdict,
            "decision_id": decision.id,
            "decision_hash": decision.decision_hash,
            "estimated_cost_usd": decision.estimated_cost_usd,
        }),
    )
}

fn json_response(status: StatusCode, value: serde_json::Value) -> Response<Body> {
    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(value.to_string()))
        .expect("json response must build")
}

fn session_id(headers: &HeaderMap, policy: &Policy) -> String {
    headers
        .get(SESSION_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| policy.kite_passport.session_key.clone())
}

fn upstream_path_and_query(uri: &Uri) -> String {
    let path = uri.path();
    let path =
        path.strip_prefix("/proxy/").map_or(
            path,
            |stripped| {
                if stripped.is_empty() {
                    "/"
                } else {
                    stripped
                }
            },
        );
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    match uri.query() {
        Some(query) => format!("{path}?{query}"),
        None => path,
    }
}

fn request_key(method: &Method, uri: &Uri) -> String {
    format!("{method} {}", upstream_path_and_query(uri))
}

fn parse_chain_id(network: &str) -> Result<u64, String> {
    network
        .strip_prefix("eip155:")
        .ok_or_else(|| format!("unsupported network id: {network}"))?
        .parse()
        .map_err(|err| format!("invalid chain id: {err}"))
}

fn nonce() -> B256 {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(Uuid::new_v4().as_bytes());
    bytes[16..24].copy_from_slice(&unix_timestamp().to_be_bytes());
    B256::from(bytes)
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn should_forward_header(name: &HeaderName) -> bool {
    !matches!(
        name.as_str(),
        "host" | "content-length" | "connection" | PAYMENT_SIGNATURE
    )
}

fn should_return_header(name: &str) -> bool {
    !matches!(name, "content-length" | "connection" | "transfer-encoding")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::Request;
    use tower::ServiceExt;

    fn test_policy(budget_usd: f64, cap_usd: f64) -> Policy {
        serde_yaml::from_str(&format!(
            r#"
version: "1.0"
name: test
session:
  budget_usd: {budget_usd}
  per_request_cap_usd: {cap_usd}
  max_duration_minutes: 30
loop_detection:
  enabled: true
  window_size: 10
  similarity_threshold: 0.85
  max_consecutive_similar: 3
networks:
  allowed: ["eip155:84532"]
  assets: ["USDC"]
kite_passport:
  agent_address: "0x0000000000000000000000000000000000000000"
  session_key: "fallback-session"
  attestation_frequency: every_event
"#
        ))
        .unwrap()
    }

    fn test_state(price: &str, policy: Policy) -> AppState {
        AppState {
            upstream: "http://stub-upstream".into(),
            db_path: format!("/tmp/pennykite-proxy-test-{}.db", Uuid::new_v4()),
            policy: Arc::new(policy),
            upstream_client: Arc::new(StubUpstream {
                price: price.to_string(),
            }),
            ledger_lock: Arc::new(Mutex::new(())),
            loop_detectors: Arc::new(Mutex::new(HashMap::new())),
            signer: "0x59c6995e998f97a5a0044966f0945389d358f57d07535c8be9e515a7c99316c5"
                .parse()
                .unwrap(),
            usdc_contract: Address::from_str("0x036CbD53842c5426634e7929541eC2318f3dCF7e").unwrap(),
        }
    }

    struct StubUpstream {
        price: String,
    }

    #[async_trait]
    impl UpstreamClient for StubUpstream {
        async fn send(
            &self,
            _method: &Method,
            _url: &str,
            _headers: &HeaderMap,
            _body: Bytes,
            payment_signature: Option<String>,
        ) -> Result<UpstreamResponse, String> {
            if payment_signature.is_some() {
                let mut headers = HeaderMap::new();
                headers.insert("payment-response", HeaderValue::from_static("paid"));
                return Ok(UpstreamResponse {
                    status: StatusCode::OK,
                    headers,
                    body: Bytes::from_static(br#"{"ok":true}"#),
                });
            }
            let body = json!({
                "version": "1.0",
                "requirements": [{
                    "network": "eip155:84532",
                    "asset": "USDC",
                    "amount": self.price,
                    "decimals": 6,
                    "pay_to": "0x1111111111111111111111111111111111111111",
                    "facilitator": null
                }],
                "expires_at": null
            });
            let header = base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                body.to_string(),
            );
            let mut headers = HeaderMap::new();
            headers.insert(
                PAYMENT_REQUIRED,
                HeaderValue::from_str(&header).map_err(|err| err.to_string())?,
            );
            Ok(UpstreamResponse {
                status: StatusCode::PAYMENT_REQUIRED,
                headers,
                body: Bytes::from_static(br#"{"error":"payment_required"}"#),
            })
        }
    }

    #[tokio::test]
    async fn approved_path_replays_with_payment_signature() {
        let state = test_state("50000", test_policy(1.0, 0.5));
        let response = app(state)
            .oneshot(
                Request::builder()
                    .uri("/paid")
                    .header(SESSION_HEADER, "session-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().get("payment-response").is_some());
        assert!(response.headers().get("x-pennykite-cost-usd").is_some());
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["_pk_cost_usd"], json!(0.05));
    }

    #[tokio::test]
    async fn decisions_feed_returns_recorded_proxy_decisions() {
        let state = test_state("50000", test_policy(1.0, 0.5));
        let app = app(state);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/proxy/paid")
                    .header(SESSION_HEADER, "feed-session")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/decisions")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["summary"]["decisions_count"], json!(1));
        assert_eq!(body["decisions"][0]["session_id"], json!("feed-session"));
        assert_eq!(body["decisions"][0]["verdict"], json!("approve"));
    }

    #[tokio::test]
    async fn denied_path_returns_pennykite_402_body() {
        let state = test_state("1000000", test_policy(1.0, 0.5));
        let response = app(state)
            .oneshot(
                Request::builder()
                    .uri("/proxy/paid")
                    .header(SESSION_HEADER, "session-1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["verdict"], json!("deny_budget"));
        assert_eq!(body["reason"], json!("per-request cap exceeded"));
    }

    #[tokio::test]
    async fn repeated_identical_requests_return_deny_loop() {
        let state = test_state("50000", test_policy(1.0, 0.5));
        let app = app(state);

        for _ in 0..3 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/proxy/paid")
                        .header(SESSION_HEADER, "loop-session")
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/proxy/paid")
                    .header(SESSION_HEADER, "loop-session")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PAYMENT_REQUIRED);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["verdict"], json!("deny_loop"));
        assert_eq!(body["reason"], json!("loop detected"));
    }
}
