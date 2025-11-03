use anthropic::types::{ContentBlock, Message, MessagesRequestBuilder, MessagesResponse};
use directories::ProjectDirs;
use dotenvy;

use reqwest::{Client, ClientBuilder};

use crate::common::config::{APPLICATION, ORGANIZATION, QUALIFIER};

#[derive(Debug)]
pub struct AnthropicClient {
    client: Client,
    api_key: String,
    url: String,
    api_version: String,
    model: String,
    max_tokens: usize,
}

impl AnthropicClient {

    // public

    /// Create a new `AnthropicClient` instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap();
        let env_path = project_dirs.config_dir().join(".env");
        dotenvy::from_path(env_path).ok();

        let api_key = std::env::var("ANTHROPIC_API_KEY")?;
        Ok(Self {
            client: ClientBuilder::default().build()?,
            api_key: api_key,
            url: crate::common::config
                ::get_config("anthropic_config.json", "base_url")?,
            api_version: crate::common::config
                ::get_config("anthropic_config.json", "anthropic_version")?,
            model: crate::common::config
                ::get_config("anthropic_config.json", "default_model")?,
            max_tokens: crate::common::config
                ::get_config("anthropic_config.json", "max_tokens")?,
        })
    }

    /// Estimates the token count for a given messages vec
    /// 
    /// Rough estimate: 1 token ~= 4 characters
    #[allow(unused)]
    pub fn estimate_token_count_text(&self, messages: &Vec<Message>) -> usize {
        messages.iter()
            .map(|m| m.content.iter()
                .map(|c| match c {
                    ContentBlock::Text { text } => text.len()/4,
                    _ => 0,
                }).sum::<usize>()
            ).sum()
    }

    /// Sends a list of messages, system prompt, and randomness (temperature) to the LLM and returns the response
    pub async fn call_model(&self, messages: &Vec<Message>, sys_prompt: &str, randomness: f64) -> Result<MessagesResponse,Box<dyn std::error::Error>> {
        let request = MessagesRequestBuilder::default()
            .model(&self.model)
            .max_tokens(self.max_tokens)
            .temperature(randomness)
            .messages(&messages[..])
            .system(sys_prompt)
            .build()?;

        let response_json = self.client
            .post(format!("{}/messages", &self.url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("content-type", "application/json")
            .json(&serde_json::json!(request))
            .send().await?
            .json::<serde_json::Value>().await?;

        let response_map = response_json.as_object()
            .ok_or("Response was null")?;

        if response_map.contains_key("error") {
            Err(format!("{}", response_map.get("error").ok_or("Error is null")?).into())
        } else {
            Ok(serde_json::from_value(response_json)?)
        }
    }
}