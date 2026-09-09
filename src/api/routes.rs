use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::models::{Observation, SnapshotMetadata, ToolResult};
use crate::store::SessionManager;

#[derive(Clone)]
pub struct AppState {
    pub session_manager: SessionManager,
}

#[derive(Debug, Deserialize)]
pub struct CreateSandboxRequest {
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct CreateSandboxResponse {
    pub sandbox_id: String,
    pub status: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct ToolReadRequest {
    pub path: String,
}

#[derive(Debug, Deserialize)]
pub struct ToolWriteRequest {
    pub path: String,
    pub content: String,
    pub source_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ToolFetchRequest {
    pub url: String,
    pub save_as: Option<String>,
    pub mock_content: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ToolExecRequest {
    pub program: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotRequest {
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct RewindRequest {
    pub snapshot_id: String,
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/v1/sandboxes", post(create_sandbox).get(list_sandboxes))
        .route("/v1/sandboxes/:id", delete(delete_sandbox))
        .route("/v1/sandboxes/:id/tools/read", post(tool_read))
        .route("/v1/sandboxes/:id/tools/write", post(tool_write))
        .route("/v1/sandboxes/:id/tools/fetch", post(tool_fetch))
        .route("/v1/sandboxes/:id/tools/exec", post(tool_exec))
        .route("/v1/sandboxes/:id/snapshots", post(create_snapshot))
        .route("/v1/sandboxes/:id/rewind", post(rewind_snapshot))
        .route("/v1/sandboxes/:id/observe", get(observe_sandbox))
        .route("/v1/sandboxes/:id/telemetry", get(export_telemetry))
        .with_state(state)
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "version": "0.1.0",
        "engine": "rust"
    }))
}

async fn create_sandbox(
    State(state): State<AppState>,
    Json(payload): Json<CreateSandboxRequest>,
) -> Result<(StatusCode, Json<CreateSandboxResponse>), StatusCode> {
    match state.session_manager.create_session(&payload.description).await {
        Ok(sid) => Ok((
            StatusCode::CREATED,
            Json(CreateSandboxResponse {
                sandbox_id: sid,
                status: "active".to_string(),
                description: payload.description,
            }),
        )),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn list_sandboxes(State(state): State<AppState>) -> impl IntoResponse {
    let list = state.session_manager.list_sessions().await;
    Json(serde_json::json!({ "sandboxes": list }))
}

async fn delete_sandbox(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if state.session_manager.terminate_session(&id).await {
        Ok(Json(serde_json::json!({ "sandbox_id": id, "status": "terminated" })))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn tool_read(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ToolReadRequest>,
) -> Result<Json<ToolResult>, StatusCode> {
    let harness_arc = state.session_manager.get_session(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    let mut harness = harness_arc.lock().await;
    let res = harness.read(&req.path);
    Ok(Json(res))
}

async fn tool_write(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ToolWriteRequest>,
) -> Result<Json<ToolResult>, StatusCode> {
    let harness_arc = state.session_manager.get_session(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    let mut harness = harness_arc.lock().await;
    let res = harness.write(&req.path, &req.content, req.source_ids);
    Ok(Json(res))
}

async fn tool_fetch(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ToolFetchRequest>,
) -> Result<Json<ToolResult>, StatusCode> {
    let harness_arc = state.session_manager.get_session(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    let mut harness = harness_arc.lock().await;
    let res = harness.fetch(&req.url, req.save_as.as_deref(), req.mock_content.as_deref());
    Ok(Json(res))
}

async fn tool_exec(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ToolExecRequest>,
) -> Result<Json<ToolResult>, StatusCode> {
    let harness_arc = state.session_manager.get_session(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    let mut harness = harness_arc.lock().await;
    let res = harness.exec(&req.program, &req.args);
    Ok(Json(res))
}

async fn create_snapshot(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SnapshotRequest>,
) -> Result<(StatusCode, Json<SnapshotMetadata>), StatusCode> {
    let harness_arc = state.session_manager.get_session(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    let mut harness = harness_arc.lock().await;
    match harness.snapshot(&req.description) {
        Ok(meta) => Ok((StatusCode::CREATED, Json(meta))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn rewind_snapshot(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<RewindRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let harness_arc = state.session_manager.get_session(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    let mut harness = harness_arc.lock().await;
    match harness.rewind(&req.snapshot_id) {
        Ok(_) => Ok(Json(serde_json::json!({
            "sandbox_id": id,
            "snapshot_id": req.snapshot_id,
            "success": true
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "sandbox_id": id,
            "snapshot_id": req.snapshot_id,
            "success": false,
            "error": e.to_string()
        }))),
    }
}

async fn observe_sandbox(
    State(state): State<AppState>,
    Path(id): Path<String>,
    ) -> Result<Json<Observation>, StatusCode> {
    let harness_arc = state.session_manager.get_session(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    let mut harness = harness_arc.lock().await;
    Ok(Json(harness.observe()))
}

async fn export_telemetry(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let harness_arc = state.session_manager.get_session(&id).await.ok_or(StatusCode::NOT_FOUND)?;
    let harness = harness_arc.lock().await;
    let events = harness.get_audit_events();
    Ok(Json(serde_json::json!({
        "sandbox_id": id,
        "count": events.len(),
        "events": events
    })))
}
