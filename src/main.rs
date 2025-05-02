extern crate spider;
use anyhow::{Context, Result};
use spider::tokio;

mod rag;
mod web_scraper;
use dotenvy::dotenv;
use rag::{embeddings, preprocessing::content_to_chunks};
use web_scraper::scrape::{crawl_website, parse_website};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().context("Could not find .env")?;

    let target = "https://spider.cloud";
    let website = crawl_website(&target).await;
    let content = parse_website(website).unwrap();
    let chunks = content_to_chunks(&content);

    for c in &chunks {
        println!("{:?}", c);
        println!();
        println!();
    }
    // let embeddings = openai_embeddings(&chunks).await?;
    // println!("{:?}", embeddings);

    Ok(())
}
