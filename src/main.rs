extern crate spider;
use anyhow::{Context, Result};

mod rag;
mod web_scraper;

use dotenvy::dotenv;
use rag::open_ai::Orchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().context("Could not find .env")?;

    let target = "https://spider.cloud";
    let mut orchestrator = Orchestrator::new().await?;

    let user_query = "What is spider.cloud";

    println!("Message added successfully to thread.");
    println!("\nStarting Assistant Run to answer query: '{}'", user_query);
    match orchestrator.query_url(&target, &user_query).await {
        Ok(response) => {
            println!("\n✅ --- Final Assistant Response --- ✅");
            println!("{}", response);
            println!("--- End Response ---");
        }
        Err(e) => {
            eprintln!("\n❌ --- Error during run or response retrieval --- ❌");
            eprintln!("{:?}", e);
            println!("-----------------------------------------------");
            return Err(e);
        }
    }
    Ok(())
}
