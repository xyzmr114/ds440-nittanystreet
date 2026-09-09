use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use tower::ServiceExt;

use taintbox::api::routes::{create_router, AppState};
use taintbox::store::SessionManager;

#[tokio::test]
async fn test_health_endpoint() {
    let state = AppState::new(SessionManager::new());
    let app = create_router(state);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_and_delete_sandbox() {
    let state = AppState::new(SessionManager::new());
    let app = create_router(state.clone());

    // 1. Create sandbox
    let create_req = Request::builder()
        .method("POST")
        .uri("/v1/sandboxes")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"description": "integration-test"}"#))
        .unwrap();

    let res = app.clone().oneshot(create_req).await.unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let sandbox_id = body["sandbox_id"].as_str().unwrap();
    assert!(sandbox_id.starts_with("sbx_"));

    // 2. Delete sandbox
    let del_req = Request::builder()
        .method("DELETE")
        .uri(format!("/v1/sandboxes/{}", sandbox_id))
        .body(Body::empty())
        .unwrap();

    let del_res = app.oneshot(del_req).await.unwrap();
    assert_eq!(del_res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_api_tool_write_and_read() {
    let state = AppState::new(SessionManager::new());
    let app = create_router(state.clone());

    // Create session
    let sid = state.session_manager.create_session("write-test").await.unwrap();

    // Write tool
    let write_req = Request::builder()
        .method("POST")
        .uri(format!("/v1/sandboxes/{}/tools/write", sid))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"path": "test.txt", "content": "hello rust"}"#))
        .unwrap();

    let write_res = app.clone().oneshot(write_req).await.unwrap();
    assert_eq!(write_res.status(), StatusCode::OK);

    // Read tool
    let read_req = Request::builder()
        .method("POST")
        .uri(format!("/v1/sandboxes/{}/tools/read", sid))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"path": "test.txt"}"#))
        .unwrap();

    let read_res = app.oneshot(read_req).await.unwrap();
    assert_eq!(read_res.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(read_res.into_body(), usize::MAX).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["output"], "hello rust");
    assert_eq!(body["status"], "SUCCESS");
}
