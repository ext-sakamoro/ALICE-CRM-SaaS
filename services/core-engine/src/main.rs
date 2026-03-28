// ALICE-CRM-SaaS core-engine
// License: AGPL-3.0-or-later

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize)]
struct Stats {
    total_requests: u64,
    contacts_requests: u64,
    deals_requests: u64,
    pipeline_requests: u64,
    score_requests: u64,
}

#[derive(Clone)]
struct AppState {
    stats: Arc<Mutex<Stats>>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

#[derive(Deserialize)]
struct CreateContactRequest {
    name: String,
    email: String,
    company: Option<String>,
}

#[derive(Serialize)]
struct Contact {
    id: String,
    name: String,
    email: String,
    company: Option<String>,
    created_at: &'static str,
}

#[derive(Deserialize)]
struct CreateDealRequest {
    title: String,
    value: f64,
    contact_id: String,
    stage: Option<String>,
}

#[derive(Serialize)]
struct Deal {
    id: String,
    title: String,
    value: f64,
    contact_id: String,
    stage: String,
    probability: f64,
}

#[derive(Serialize)]
struct PipelineStage {
    name: &'static str,
    deals: u32,
    total_value: f64,
}

#[derive(Serialize)]
struct PipelineResponse {
    stages: Vec<PipelineStage>,
    total_pipeline_value: f64,
}

#[derive(Deserialize)]
struct ScoreRequest {
    contact_id: String,
    deal_id: Option<String>,
}

#[derive(Serialize)]
struct ScoreResponse {
    id: String,
    contact_id: String,
    score: f64,
    grade: &'static str,
    factors: Vec<&'static str>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "alice-crm-core-engine",
        version: "0.1.0",
    })
}

async fn crm_contacts(
    State(state): State<AppState>,
    body: Option<Json<CreateContactRequest>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.contacts_requests += 1;
    if let Some(Json(req)) = body {
        info!("crm/contacts POST name={}", req.name);
        let contact = Contact {
            id: Uuid::new_v4().to_string(),
            name: req.name,
            email: req.email,
            company: req.company,
            created_at: "2026-03-09T00:00:00Z",
        };
        Ok(Json(serde_json::to_value(contact).unwrap()))
    } else {
        info!("crm/contacts GET");
        Ok(Json(serde_json::json!({ "contacts": [], "total": 0 })))
    }
}

async fn crm_deals(
    State(state): State<AppState>,
    body: Option<Json<CreateDealRequest>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.deals_requests += 1;
    if let Some(Json(req)) = body {
        info!("crm/deals POST title={} value={}", req.title, req.value);
        let deal = Deal {
            id: Uuid::new_v4().to_string(),
            title: req.title,
            value: req.value,
            contact_id: req.contact_id,
            stage: req.stage.unwrap_or_else(|| "prospect".to_string()),
            probability: 0.25,
        };
        Ok(Json(serde_json::to_value(deal).unwrap()))
    } else {
        info!("crm/deals GET");
        Ok(Json(serde_json::json!({ "deals": [], "total": 0 })))
    }
}

async fn crm_pipeline(State(state): State<AppState>) -> Json<PipelineResponse> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.pipeline_requests += 1;
    Json(PipelineResponse {
        stages: vec![
            PipelineStage { name: "prospect",    deals: 12, total_value: 240_000.0 },
            PipelineStage { name: "qualified",   deals: 8,  total_value: 320_000.0 },
            PipelineStage { name: "proposal",    deals: 5,  total_value: 280_000.0 },
            PipelineStage { name: "negotiation", deals: 3,  total_value: 190_000.0 },
            PipelineStage { name: "closed_won",  deals: 2,  total_value: 140_000.0 },
        ],
        total_pipeline_value: 1_170_000.0,
    })
}

async fn crm_score(
    State(state): State<AppState>,
    Json(req): Json<ScoreRequest>,
) -> Result<Json<ScoreResponse>, StatusCode> {
    let mut stats = state.stats.lock().unwrap();
    stats.total_requests += 1;
    stats.score_requests += 1;
    info!("crm/score contact_id={}", req.contact_id);
    Ok(Json(ScoreResponse {
        id: Uuid::new_v4().to_string(),
        contact_id: req.contact_id,
        score: 78.4,
        grade: "B+",
        factors: vec!["email_engagement", "deal_history", "company_size"],
    }))
}

async fn crm_stats(State(state): State<AppState>) -> Json<Stats> {
    let stats = state.stats.lock().unwrap().clone();
    Json(stats)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let state = AppState {
        stats: Arc::new(Mutex::new(Stats::default())),
    };

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/crm/contacts", get(crm_contacts).post(crm_contacts))
        .route("/api/v1/crm/deals", get(crm_deals).post(crm_deals))
        .route("/api/v1/crm/pipeline", get(crm_pipeline))
        .route("/api/v1/crm/score", post(crm_score))
        .route("/api/v1/crm/stats", get(crm_stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9144").await.unwrap();
    info!("alice-crm-core-engine listening on :9144");
    axum::serve(listener, app).await.unwrap();
}
