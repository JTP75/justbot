use directories::ProjectDirs;
use dotenvy;

use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};

use crate::common::{config::{APPLICATION, ORGANIZATION, QUALIFIER}, types::{ContentBlock, Message, MessagesResponse}};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

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

    // todo we can probably collapse these two functions into one with an Option<Vec<ToolDefinition>>

    /// Sends a list of messages, system prompt, and randomness (temperature) to the LLM and returns the response
    pub async fn call_model(
        &self, 
        messages: &Vec<Message>, 
        sys_prompt: &str, 
        tools: Option<&Vec<ToolDefinition>>, 
        randomness: f64
    ) 
    -> Result<MessagesResponse,Box<dyn std::error::Error>> {

        let sp = if sys_prompt.is_empty() { 
            serde_json::json!("") 
        } else { 
            build_ephemeral_sys_prompt(sys_prompt) 
        };

        let mut token_estimate = self.estimate_token_count_text(messages)
                + sys_prompt.len()/4;

        let t = if let Some(t) = tools {
            token_estimate += t.iter().map(|t| t.description.len()/4).sum::<usize>();
            Some(build_ephemeral_tools(&t))
        } else {
            None
        };

        let request_json = serde_json::json!({
            "model": &self.model, 
            "max_tokens": self.max_tokens, 
            "temperature": randomness,
            "messages": &messages[..],
            "stream": false, 
            "system": sp,
            "tools": t
        });

        log::info!("Input tokens (approx): {}", token_estimate);

        let response_json = self.client
            .post(format!("{}/messages", &self.url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("content-type", "application/json")
            .json(&request_json)
            .send().await?
            .json::<serde_json::Value>().await?;

        let response_map = response_json.as_object()
            .ok_or("Response was null")?;

        if response_map.contains_key("error") {
            Err(format!("{}", response_map.get("error").ok_or("Error is null")?).into())
        } else {
            let t_in = response_map.get("usage").unwrap().get("input_tokens").unwrap().as_u64().unwrap();
            let t_out = response_map.get("usage").unwrap().get("output_tokens").unwrap().as_u64().unwrap();
            log::info!("Input tokens:  {}", t_in);
            log::info!("Output tokens: {}", t_out);
            let response = serde_json::from_value(response_json)?;
            Ok(response)
        }
    }
}

fn build_ephemeral_sys_prompt(sys_prompt: &str) -> serde_json::Value {
    serde_json::json!([{
        "type": "text",
        "text": sys_prompt,
        "cache_control": { "type": "ephemeral" }
    }])
}

fn build_ephemeral_tools(tools: &[ToolDefinition]) -> serde_json::Value {
    let mut tool_defs: Vec<serde_json::Value> = tools.iter()
        .map(|t| serde_json::json!(t))
        .collect();

    if let Some(obj) = tool_defs.last_mut().and_then(|v| v.as_object_mut()) {
        obj.insert(
            "cache_control".to_string(),
            serde_json::json!({ "type": "ephemeral" }),
        );
    }

    serde_json::Value::Array(tool_defs)
}