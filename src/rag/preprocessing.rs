use text_splitter::TextSplitter;

pub fn content_to_chunks(website_content: &str) -> Vec<String> {
    let size = 512;
    let splitter = TextSplitter::new(size);
    let chunks: Vec<String> = splitter
        .chunks(website_content)
        .map(String::from)
        .map(|ch| ch.replace('\n', " "))
        .collect();

    chunks
}
