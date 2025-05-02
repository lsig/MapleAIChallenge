use axum::{Json, extract::State, response::IntoResponse};
use reqwest::StatusCode;

use crate::api::{AppState, ErrorResponse, QueryRequest, QueryResponse};

pub async fn handle_query(
    State(state): State<AppState>,
    Json(payload): Json<QueryRequest>,
) -> impl IntoResponse {
    println!("Received query for URL: {}", payload.url);

    let mut orchestrator = state.orchestrator.lock().await;

    match orchestrator.query_url(&payload.url, &payload.query).await {
        Ok(answer) => (StatusCode::OK, Json(QueryResponse { answer })).into_response(),
        Err(e) => {
            eprintln!("Error processing query: {:?}", e);
            let error_response = ErrorResponse {
                error: format!("Failed to process query: {}", e),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}
