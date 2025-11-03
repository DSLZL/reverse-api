use std::net::SocketAddr;

use axum::{
    extract::State,
    http::{header::CONTENT_TYPE, Method},
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use reverse_api::Logger;
use tower_http::cors::{Any, CorsLayer};

use super::{dashboard, docs, handlers, state::AppState};

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST, Method::GET, Method::OPTIONS, Method::DELETE])
        .allow_headers([CONTENT_TYPE, axum::http::header::AUTHORIZATION]);

    Router::new()
        .route("/v1/threads", post(handlers::create_thread))
        .route("/v1/threads", get(handlers::list_threads))
        .route(
            "/v1/threads/{thread_id}/messages",
            post(handlers::add_message),
        )
        .route(
            "/v1/threads/{thread_id}/messages",
            get(handlers::list_messages),
        )
        .route("/v1/threads/{thread_id}", get(handlers::get_thread))
        .route("/v1/threads/{thread_id}", delete(handlers::delete_thread))
        .route("/v1/responses", post(handlers::create_response))
        .route("/v1/config/deepseek", post(handlers::configure_deepseek))
        .route("/v1/config/qwen", post(handlers::configure_qwen))
        .route("/v1/files/upload", post(handlers::upload_file_for_qwen))
        .route("/v1/images/generate", post(handlers::generate_image))
        .route("/v1/videos/generate", post(handlers::generate_video))
        .route("/health", get(health_check))
        .route("/v1/models", get(list_models))
        .route("/dashboard", get(dashboard::dashboard))
        .route("/dashboard/stats", get(dashboard::dashboard_stats))
        .route("/dashboard/requests", get(dashboard::dashboard_requests))
        .route("/docs", get(docs::api_docs))
        .with_state(state)
        .layer(cors)
}

async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let threads = state.list_threads().await;
    let proxy_info = state.get_default_proxy().unwrap_or("none");

    Json(serde_json::json!({
        "status": "ok",
        "default_proxy": proxy_info,
        "active_threads": threads.len(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn list_models(State(state): State<AppState>) -> impl IntoResponse {
    let mut static_models = vec![
        serde_json::json!({
            "id": "grok-3-auto",
            "object": "model",
            "created": 1677610602,
            "owned_by": "xai"
        }),
        serde_json::json!({
            "id": "grok-3-turbo",
            "object": "model",
            "created": 1677610602,
            "owned_by": "xai"
        }),
        serde_json::json!({
            "id": "grok-3-mini",
            "object": "model",
            "created": 1677610602,
            "owned_by": "xai"
        }),
        serde_json::json!({
            "id": "chatgpt",
            "object": "model",
            "created": 1677610602,
            "owned_by": "openai"
        }),
        serde_json::json!({
            "id": "gpt-4",
            "object": "model",
            "created": 1677610602,
            "owned_by": "openai"
        }),
        serde_json::json!({
            "id": "glm-4.6",
            "object": "model",
            "created": 1677610602,
            "owned_by": "z.ai"
        }),
        serde_json::json!({
            "id": "deepseek",
            "object": "model",
            "created": 1704067200,
            "owned_by": "deepseek"
        }),
    ];

    // Add Qwen models if available
    if let Some(qwen_models) = state.get_qwen_models().await {
        for model in qwen_models {
            static_models.push(serde_json::json!({
                "id": model.id,
                "object": model.object,
                "name": model.name,
                "owned_by": model.owned_by,
                "info": model.info
            }));
        }
    }

    Json(serde_json::json!({
        "object": "list",
        "data": static_models
    }))
}

pub async fn run(
    host: &str,
    port: u16,
    default_proxy: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;

    let state = AppState::new(default_proxy);
    let app = router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    Logger::success(&format!("🚀 API server listening on http://{}", local_addr));
    Logger::info("📚 API Endpoints:");
    Logger::info("  Health: GET /health");
    Logger::info("  Models: GET /v1/models");
    Logger::info("  Threads: POST /v1/threads, GET /v1/threads");
    Logger::info("  Thread: GET/DELETE /v1/threads/:thread_id");
    Logger::info("  Messages: POST/GET /v1/threads/:thread_id/messages");
    Logger::info(
        "  Response: POST /v1/responses (supports grok, chatgpt, glm, deepseek, qwen models)",
    );
    Logger::info("  Config DeepSeek: POST /v1/config/deepseek");
    Logger::info("  Config Qwen: POST /v1/config/qwen");
    Logger::info("  Dashboard: GET /dashboard");
    Logger::info("  Dashboard Stats: GET /dashboard/stats");
    Logger::info("  Dashboard Requests: GET /dashboard/requests");
    Logger::info("  API Docs: GET /docs");

    axum::serve(listener, app).await?;

    Ok(())
}
