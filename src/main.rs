//! `cargo run --example scrape`
// extern crate env_logger;
extern crate spider;

use scraper::{Html, Selector};
// use env_logger::Env;
use spider::tokio;
use spider::website::Website;

mod preprocessor;
mod web_scraper;
use preprocessor::chunks::content_to_chunks;
use web_scraper::scrape::{crawl_website, parse_website};

#[tokio::main]
async fn main() {
    use std::io::{Write, stdout};

    // let env = Env::default()
    //     .filter_or("RUST_LOG", "info")
    //     .write_style_or("RUST_LOG_STYLE", "always");
    //
    // env_logger::init_from_env(env);
    let target = "https://spider.cloud";
    let website = crawl_website(&target).await;
    let content = parse_website(website).unwrap();
    let chunks = content_to_chunks(&content);

    for c in chunks {
        println!("{:?}", c);
        println!();
        println!();
    }
}
