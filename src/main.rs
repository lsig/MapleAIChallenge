extern crate spider;
use anyhow::{Context, Result};
use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::post};

mod api;
mod endpoints;
mod rag;
mod web_scraper;

use api::AppState;
use dotenvy::dotenv;
use endpoints::query::handle_query;
use rag::open_ai::Orchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().context("Could not find .env")?;

    // Create the orchestrator instance (handles its own init failures)
    let orchestrator = Orchestrator::new()
        .await
        .expect("Failed to initialize Orchestrator"); // Or handle error gracefully

    // Create the shared state
    let app_state = AppState::new(orchestrator);
    // Build the Axum application router (define routes next)
    let app = Router::new()
        .route("/query", post(handle_query)) // Define POST endpoint
        // Add CORS layer if needed for browser clients
        // .layer(tower_http::cors::CorsLayer::permissive())
        // Add tracing layer
        // .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(app_state); // Provide the state to the router

    // Run the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    println!("Server listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
