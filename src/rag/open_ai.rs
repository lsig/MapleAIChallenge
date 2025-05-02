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
use reqwest::Url;
use serde_json::json;
use spider::hashbrown::HashSet;
use tokio::time::Duration;

use crate::web_scraper::scrape::{crawl_website, parse_website};

pub struct Orchestrator {
    client: Client<OpenAIConfig>,
    assistant_id: String,
    files: HashSet<String>,
}

impl Orchestrator {
    pub async fn new() -> Result<Self> {
        let client = Client::new();
        let assistant = create_assistant(&client, "gpt-3.5-turbo").await?;
        let assistant_id = assistant.id;
        let files = HashSet::new();

        Ok(Self {
            client,
            assistant_id,
            files,
        })
    }

    pub async fn query_url(&mut self, url: &str, query: &str) -> Result<String> {
        let parsed_url = Url::parse(url).context(format!("Failed to parse URL: '{}'", url))?;
        let host = parsed_url.domain().unwrap_or("unknown");

        let website = crawl_website(url).await;
        let content = parse_website(website).unwrap();

        let file = self
            .upload_content(&format!("{}.txt", host), content)
            .await?;

        let thread = self.create_thread().await?;
        self.user_query_to_thread(&thread.id, query, &file.id)
            .await?;

        self.get_assistant_response_for_thread(&thread.id).await
    }

    async fn upload_content(&mut self, filename: &str, content: String) -> Result<OpenAIFile> {
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

        let file_object = self
            .client
            .files()
            .create(request)
            .await
            .context("OpenAI file upload API call failed")?;

        self.files.insert(file_object.id.to_owned());

        println!("Successfully uploaded file. File ID: {}", file_object.id);
        Ok(file_object)
    }

    pub async fn create_thread(&self) -> Result<ThreadObject> {
        println!("Creating new conversation thread...");

        let request = CreateThreadRequestArgs::default()
            .build()
            .context("Failed to build create thread request")?;

        let thread = self
            .client
            .threads()
            .create(request)
            .await
            .context("OpenAI thread creation API call failed")?;

        println!("Successfully created thread. Thread ID: {}", thread.id);
        Ok(thread)
    }

    pub async fn user_query_to_thread(
        &self,
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

        let message_object = self
            .client
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

    pub async fn get_assistant_response_for_thread(&self, thread_id: &str) -> Result<String> {
        println!(
            "Starting run for Assistant '{}' on Thread '{}'",
            self.assistant_id, thread_id
        );

        let run_request = CreateRunRequestArgs::default()
            .assistant_id(self.assistant_id.to_owned())
            .build()
            .context("Failed to build create run request")?;

        let run = self
            .client
            .threads()
            .runs(thread_id)
            .create(run_request)
            .await
            .context("OpenAI run creation API call failed")?;

        let run_id = run.id.clone();
        println!("Run created: {}. Initial Status: {:?}", run_id, run.status);

        let completed_run = self
            .poll_run_completion(thread_id, &run_id)
            .await
            .context(format!("Polling failed for run {}", run_id))?;

        println!("Run {} completed successfully.", completed_run.id);

        let response_text = self
            .get_latest_assistant_response(thread_id)
            .await
            .context(format!(
                "Failed to get final response from thread {}",
                thread_id
            ))?;

        println!("Assistant Response Retrieved.");
        Ok(response_text)
    }

    async fn poll_run_completion(&self, thread_id: &str, run_id: &str) -> Result<RunObject> {
        let mut attempts = 0;
        let max_attempts = 20;
        let poll_interval_secs = 5;

        loop {
            attempts += 1;
            if attempts > max_attempts {
                bail!("Run {} timed out after {} attempts.", run_id, max_attempts);
            }

            let run = self
                .client
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

    async fn get_latest_assistant_response(&self, thread_id: &str) -> Result<String> {
        println!(
            "Retrieving latest assistant message from thread '{}'...",
            thread_id
        );

        let messages_response = self
            .client
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
}

async fn create_assistant(
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
