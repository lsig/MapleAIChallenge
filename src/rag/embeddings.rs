use anyhow::{Context, Result, bail};
use async_openai::Client;
use async_openai::config::OpenAIConfig;
use async_openai::types::{
    AssistantObject, AssistantTools, CreateAssistantRequestArgs, CreateFileRequestArgs,
    CreateMessageRequestArgs, CreateRunRequestArgs, CreateThreadRequestArgs, FileInput,
    FilePurpose, MessageAttachment, MessageAttachmentTool, MessageContent, MessageObject,
    MessageRole, OpenAIFile, RunObject, RunStatus, ThreadObject,
};

use bytes::Bytes;
use serde_json::json;
use tokio::time::Duration;

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

pub async fn get_assistant_response_for_thread(
    client: &Client<OpenAIConfig>,
    thread_id: &str,
    assistant_id: &str,
) -> Result<String> {
    println!(
        "Starting run for Assistant '{}' on Thread '{}'",
        assistant_id, thread_id
    );

    let run_request = CreateRunRequestArgs::default()
        .assistant_id(assistant_id)
        .build()
        .context("Failed to build create run request")?;

    let run = client
        .threads()
        .runs(thread_id)
        .create(run_request)
        .await
        .context("OpenAI run creation API call failed")?;

    let run_id = run.id.clone();
    println!("Run created: {}. Initial Status: {:?}", run_id, run.status);

    let completed_run = poll_run_completion(client, thread_id, &run_id)
        .await
        .context(format!("Polling failed for run {}", run_id))?;

    println!("Run {} completed successfully.", completed_run.id);

    let response_text = get_latest_assistant_response(client, thread_id)
        .await
        .context(format!(
            "Failed to get final response from thread {}",
            thread_id
        ))?;

    println!("Assistant Response Retrieved.");
    Ok(response_text)
}

async fn poll_run_completion(
    client: &Client<OpenAIConfig>,
    thread_id: &str,
    run_id: &str,
) -> Result<RunObject> {
    let mut attempts = 0;
    let max_attempts = 20;
    let poll_interval_secs = 5;

    loop {
        attempts += 1;
        if attempts > max_attempts {
            bail!("Run {} timed out after {} attempts.", run_id, max_attempts);
        }

        let run = client
            .threads()
            .runs(thread_id)
            .retrieve(run_id)
            .await
            .context(format!("Polling failed: Could not retrieve run {}", run_id))?;

        println!(
            "Polling Run {}: Status {:?}. (Attempt {}/{})",
            run_id, run.status, attempts, max_attempts
        );

        match run.status {
            RunStatus::Queued | RunStatus::InProgress => {
                tokio::time::sleep(Duration::from_secs(poll_interval_secs)).await;
            }
            RunStatus::Completed => {
                return Ok(run);
            }
            RunStatus::Failed
            | RunStatus::Incomplete
            | RunStatus::Cancelled
            | RunStatus::Expired
            | RunStatus::RequiresAction
            | RunStatus::Cancelling => {
                let error_message = run
                    .last_error
                    .map_or("No error details provided.".to_string(), |e| {
                        format!("Code: {:?}, Message: {:?}", e.code, e.message)
                    });
                bail!(
                    "Run {} ended in terminal state: {:?}. Error: {}",
                    run_id,
                    run.status,
                    error_message
                );
            }
        }
    }
}

async fn get_latest_assistant_response(
    client: &Client<OpenAIConfig>,
    thread_id: &str,
) -> Result<String> {
    println!(
        "Retrieving latest assistant message from thread '{}'...",
        thread_id
    );

    let messages_response = client
        .threads()
        .messages(thread_id)
        .list(&json!({}))
        .await
        .context(format!(
            "Retrieval failed: Could not list messages for thread {}",
            thread_id
        ))?;

    let assistant_message = messages_response
        .data
        .iter()
        .find(|msg| msg.role == MessageRole::Assistant)
        .context(format!(
            "Retrieval failed: No assistant message found in recent messages for thread {}",
            thread_id
        ))?;

    let text_content = assistant_message
        .content
        .iter()
        .find_map(|part| match part {
            MessageContent::Text(text_block) => Some(text_block.text.value.clone()),
            _ => None,
        })
        .context("Retrieval failed: Assistant message has no text content")?;

    Ok(text_content)
}
