# Rayman (RAG Rust CLI)

This is my attempt of building a rag from scraping a website contents and querying a model.
At first I wanted to get vector embeddings from OpenAI, run my own local model with those
embeddings and additionally have my own vector database for querying. However I didn't have the
time. So this basically is a OpenAI API wrapper that uploads the parsed content as a file,
creates an assistant that can search through that file, making at least somewhat better than
a regular LLM, (Hopefully?, Given the context?).

### How to run

- Make sure you have rust installed (see rustup)
- Create .env file with your OPENAI_API_KEY in the root directory (not src)
- cargo run -- release

Query away.

### Considerations

- Slow: parsing, uploading, querying, its all slow. Ideally I would add a cache for repeated queries
- Assistant: a new assistant is spawned on every run (wasteful), fix make persistant (could just store in file or sqlitedb)
- Extra features: the scraper crawls the website from the root, so more context is provided (yay)
- Also see codeforces for my solutions to the other challenge! :0
