use std::time::Duration;

use scraper::{Html, Selector};
use spider::website::Website;

pub async fn crawl_website(url: &str) -> Website {
    let mut website: Website = Website::new(url);

    website.configuration.respect_robots_txt = true;
    website.configuration.depth_distance = 1;
    website.configuration.delay = 15; // Defaults to 250 ms
    website.configuration.user_agent = Some(Box::new("MapleBot".into()));

    website
        .configuration
        .with_crawl_timeout(Some(Duration::from_secs(5)));

    println!("Scraping..");

    website.scrape().await;

    println!("Done..");

    website
}

pub fn parse_website(website: Website) -> Option<String> {
    let tags = "p, h1, h2, h3, h4, h5, h6, article, section";
    let selector = Selector::parse(&tags).ok()?;

    let mut collected_text = String::new();

    for page in website.get_pages()?.iter() {
        let html = page.get_html();
        let document = Html::parse_document(&html);

        for element in document.select(&selector) {
            let text = element.text().collect::<Vec<_>>().join(" ");
            let trimmed_text = text.trim();

            if trimmed_text.is_empty() {
                continue;
            }

            collected_text.push_str(trimmed_text);
        }
    }

    if collected_text.is_empty() {
        None
    } else {
        Some(collected_text)
    }
}
