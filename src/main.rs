extern crate spider;
use anyhow::{Context, Result};
use axum::{Router, routing::post};

mod api;
mod rag;
mod web_scraper;

use api::model::AppState;
use api::query::handle_query;
use dotenvy::dotenv;
use rag::open_ai::Orchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().context("Could not find .env")?;

    let orchestrator = Orchestrator::new()
        .await
        .expect("Failed to initialize Orchestrator");

    let app_state = AppState::new(orchestrator);
    let app = Router::new()
        .route("/query", post(handle_query))
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
