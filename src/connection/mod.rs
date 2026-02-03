use std::{any::Any, fmt::Debug};

use serde::{Deserialize, Serialize};

// anthropic core

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ContentBlock {
    Text { text: String },
    Image { source: String, media_type: String, data: String },
    ToolUse { id: String, name: String, input: serde_json::Value },
    ToolResult { tool_use_id: String, content: Vec<ToolResultContentBlock>, is_error: bool },
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    #[default]
    User,
    Assistant,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Serialize)]
pub struct MessagesResponse {
    pub id: String,
    pub r#type: String,
    pub role: Role,
    pub content: Vec<ContentBlock>,
    pub model: String,
    pub stop_sequence: Option<String>,
    // TODO add usage here
}

// anthropic tools

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AnthropicToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum ToolResultContentBlock {
    Text { text: String },
    Image { source: String, media_type: String, data: String },
    Document { source: Source, title: Option<String>, context: Option<String> },
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum Source {
    Text {
        media_type: String,
        data: String,
    },
}

// traits

#[async_trait::async_trait]
pub trait EmbeddingClient: Debug + Sync + Send {

    /// Get the embeddings for multiple input texts
    /// 
    /// - `input_text` can be either "document" or "query"
    ///     - use "document" to embed the contents of a file
    ///     - use "query" to get the query vector for a search query
    async fn get_embeddings(&self, texts: Vec<&str>, input_type: &str) 
    -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>>;

    /// Returns object as a dyn Any
    fn as_any(&self) -> &dyn Any;
}

#[async_trait::async_trait]
pub trait ChatClient: Debug + Sync + Send {

    /// sends a messages list, system prompt, tools list, model id, and 
    /// temperature to an LLM API service and awaits a response
    async fn call_model(
        &self, 
        messages: &Vec<Message>, 
        sys_prompt: &str, 
        tools: Option<&Vec<AnthropicToolDefinition>>, 
        model: Option<String>,
        randomness: f64
    )
    -> Result<MessagesResponse,Box<dyn std::error::Error>>;

    /// retrieve the currently recorded tok/min
    fn get_tpm(&self) -> (usize,usize);

    /// retrieve the service/model tok/min rate limit
    fn get_max_tpm(&self) -> (usize,usize);

    /// Returns object as a dyn Any
    fn as_any(&self) -> &dyn Any;
}

pub mod anthropic_client;
pub mod qdrant_client;
pub mod voyage_client;
pub mod local_embedding_client;