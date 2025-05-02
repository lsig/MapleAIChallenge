use crate::rag::open_ai::Orchestrator;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub orchestrator: Arc<Mutex<Orchestrator>>,
}

impl AppState {
    pub fn new(orchestrator: Orchestrator) -> Self {
        let orchestrator = Arc::new(Mutex::new(orchestrator));

        Self { orchestrator }
    }
}

// Define request/response structs for the API
#[derive(Deserialize)]
pub struct QueryRequest {
    pub url: String,
    pub query: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub answer: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}
