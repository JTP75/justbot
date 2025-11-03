use anthropic::{client::{Client, ClientBuilder}, types::{ContentBlock, Message, MessagesRequestBuilder, MessagesResponse}};
use directories::ProjectDirs;
use dotenvy;

use crate::common::config::{APPLICATION, ORGANIZATION, QUALIFIER};

#[derive(Debug)]
pub struct AnthropicClient {
    client: Client,
    model: String,
    max_tokens: usize,
}

/**
 * temperature adjusts randomness, defaults to 1.0
 * stream allows to stream response
 */

impl AnthropicClient {

    // public

    /// Create a new `AnthropicClient` instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap();
        let env_path = project_dirs.config_dir().join(".env");
        dotenvy::from_path(env_path).ok();

        let api_key = std::env::var("ANTHROPIC_API_KEY")?;
        let model: String = crate::common::config
            ::get_config("anthropic_config.json", "default_model")?;
        let max_tokens: usize = crate::common::config
            ::get_config("anthropic_config.json", "max_tokens")?;

        Ok(Self {
            client: ClientBuilder::default()
                .api_key(api_key)
                .build()?,
            model: model.as_str().into(),
            max_tokens: max_tokens,
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
        let response = self.client.messages(request).await?;
        Ok(response)
    }
}