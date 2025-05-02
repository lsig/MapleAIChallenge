use anyhow::{Context, Result, bail};
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    AssistantObject, AssistantTools, CreateAssistantRequestArgs, CreateFileRequestArgs,
    CreateMessageRequestArgs, CreateThreadRequestArgs, FileInput, FilePurpose, MessageAttachment,
    MessageAttachmentTool, MessageObject, MessageRole, OpenAIFile, ThreadObject,
};

use bytes::Bytes;

pub async fn upload_content_as_file(
    client: &Client<OpenAIConfig>,
    filename: &str,
    content: String,
) -> Result<OpenAIFile> {
    println!(
        "Uploading file '{}' ({} bytes) to OpenAI...",
        filename,
        content.len()
    );

    let file_input = FileInput::from_bytes(filename.to_string(), Bytes::from(content));
    let request = CreateFileRequestArgs::default()
        .purpose(FilePurpose::Assistants) // Must set the purpose for Assistants
        .file(file_input) // Provide filename and content as Bytes
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

pub async fn create_assistant(
    client: &Client<OpenAIConfig>,
    model_name: &str,
) -> Result<AssistantObject> {
    println!(
        "Creating Assistant with model '{}' and file_search tool...",
        model_name
    );

    let tools = vec![AssistantTools::FileSearch(Default::default())];
    let request = CreateAssistantRequestArgs::default()
        .name("Website Q&A Assistant")
        .instructions(
            "You are a helpful assistant designed to answer questions based *only* \
            on the content of the files attached to the user's message or thread. \
            Carefully search the provided file content to find the answer. \
            If the answer is not found within the provided file(s), explicitly state that \
            'The answer could not be found in the provided document(s).' \
            Do not use your general knowledge.",
        )
        .model(model_name)
        .tools(tools)
        .build()
        .context("Failed to build OpenAI assistant creation request")?;

    let assistant = client
        .assistants()
        .create(request)
        .await
        .context("OpenAI assistant creation API call failed")?;

    println!(
        "Successfully created Assistant. Assistant ID: {}",
        assistant.id
    );
    Ok(assistant)
}

pub async fn create_thread(client: &Client<OpenAIConfig>) -> Result<ThreadObject> {
    println!("Creating new conversation thread...");

    let request = CreateThreadRequestArgs::default()
        .build()
        .context("Failed to build create thread request")?;

    let thread = client
        .threads()
        .create(request)
        .await
        .context("OpenAI thread creation API call failed")?;

    println!("Successfully created thread. Thread ID: {}", thread.id);
    Ok(thread)
}

pub async fn user_query_to_thread(
    client: &Client<OpenAIConfig>,
    thread_id: &str,
    user_query: &str,
    file_id: &str,
) -> Result<MessageObject> {
    println!(
        "Adding message to thread '{}' with file attachment '{}'...",
        thread_id, file_id
    );
    println!("  User Query: '{}'", user_query);

    let attachment = MessageAttachment {
        file_id: file_id.to_string(),
        tools: vec![MessageAttachmentTool::FileSearch],
    };

    let message_request = CreateMessageRequestArgs::default()
        .role(MessageRole::User)
        .content(user_query)
        .attachments(vec![attachment])
        .build()
        .context("Failed to build create message request")?;

    let message_object = client
        .threads()
        .messages(thread_id)
        .create(message_request)
        .await
        .context("OpenAI message creation API call failed")?;

    println!(
        "Successfully added message {} to thread {}.",
        message_object.id, thread_id
    );
    Ok(message_object)
}
