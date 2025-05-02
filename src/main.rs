extern crate spider;
use std::io::{self, Write};

use anyhow::{Context, Result};

mod rag;
mod web_scraper;

use dotenvy::dotenv;
use rag::open_ai::Orchestrator;

fn read_line() -> Result<String> {
    let mut url = String::new();
    io::stdin().read_line(&mut url)?;
    let url = url.trim();

    Ok(url.to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().context("Could not find .env")?;

    let mut orchestrator = Orchestrator::new()
        .await
        .expect("Failed to initialize Orchestrator");

    loop {
        print!("URL: ");
        io::stdout().flush()?;
        let url = read_line()?;

        if url.eq_ignore_ascii_case("quit") || url.eq_ignore_ascii_case("exit") {
            break;
        }

        if url.is_empty() {
            println!("URL cannot be empty.");
            continue;
        }

        print!("Query: ");
        io::stdout().flush()?;
        let query = read_line()?;

        if query.eq_ignore_ascii_case("quit") || query.eq_ignore_ascii_case("exit") {
            break;
        }

        if query.is_empty() {
            println!("Query cannot be empty.");
            continue;
        }

        println!("\nProcessing query for '{}'...", url);
        match orchestrator.query_url(&url, &query).await {
            Ok(response) => {
                println!("\n✅ --- Assistant Response --- ✅");
                println!("{}", response);
                println!("-----------------------------");
            }
            Err(e) => {
                eprintln!("\n❌ --- Error processing query --- ❌");
                eprintln!("{:?}", e);
                println!("-----------------------------");
            }
        }
        println!();
    }

    println!("Exiting RAG CLI.");

    Ok(())
}
