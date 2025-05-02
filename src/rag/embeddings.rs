use anyhow::{Context, Result, bail};
use async_openai::config::OpenAIConfig;
use async_openai::{Client, types::CreateEmbeddingRequestArgs};

use async_openai::types::{CreateFileRequestArgs, FilePurpose, OpenAIFile};

pub async fn openai_embeddings(chunks: &Vec<String>) -> Result<Vec<Vec<f32>>> {
    let client = Client::new();

    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-small")
        .input(chunks)
        .build()
        .context("Failed to build OpenAI embedding request")?;

    println!(
        "Embedding Client: Sending {} chunks to OpenAI...",
        chunks.len()
    );

    let response = client
        .embeddings()
        .create(request)
        .await
        .context("OpenAI API call failed")?;

    let embeddings: Vec<Vec<f32>> = response
        .data
        .into_iter()
        .map(|embedding_obj| embedding_obj.embedding)
        .collect();

    if embeddings.len() != chunks.len() {
        bail!(
            "API response mismatch: Expected {} embeddings, got {}",
            chunks.len(),
            embeddings.len()
        );
    }

    println!(
        "Embedding Client: Received {} embeddings.",
        embeddings.len()
    );

    Ok(embeddings)
}

pub async fn upload_content_as_file(
    client: &Client<OpenAIConfig>,
    filename: &str,
    content: String, // Takes ownership of the content
) -> Result<OpenAIFile> {
    println!(
        "Uploading file '{}' ({} bytes) to OpenAI...",
        filename,
        content.len()
    );

    let request = CreateFileRequestArgs::default()
        .purpose(FilePurpose::Assistants) // Must set the purpose for Assistants
        .file((filename.to_string(), Bytes::from(content))) // Provide filename and content as Bytes
        .build()
        .context("Failed to build OpenAI file upload request")?;

    let file_object = client
        .files()
        .create(request)
        .await
        .context("OpenAI file upload API call failed")?;

    println!("Successfully uploaded file. File ID: {}", file_object.id);
    Ok(file_object)
}
