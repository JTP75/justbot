use std::{collections::{HashMap, VecDeque}, sync::{Arc, Mutex}, time::{Duration, Instant}};

use dotenvy;

use reqwest::{Client, ClientBuilder};
use serde::{Deserialize, Serialize};

// structs

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

// anthropic usage

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AnthropicUsage {
    cache_creation: CacheUsage,
    cache_creation_input_tokens: usize,
    cache_read_input_tokens: usize,
    input_tokens: usize,
    output_tokens: usize,
    service_tier: String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct CacheUsage {
    ephemeral_1h_input_tokens: usize,
    ephemeral_5m_input_tokens: usize
}

#[derive(Debug, Clone)]
pub struct Usage {
    timestamp: Instant,
    input_tokens: usize,
    output_tokens: usize,
}

// struct impls

impl Into<Message> for MessagesResponse {
    fn into(self) -> Message { 
        Message {
            role: self.role,
            content: self.content
        }
    }
}

impl Into<Usage> for AnthropicUsage {
    fn into(self) -> Usage {
        Usage {
            timestamp: Instant::now(),
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens
        }
    }
}

// objects

#[derive(Debug, Clone)]
pub struct AnthropicUsageMonitor {
    usage_history: Arc<Mutex<VecDeque<Usage>>>,
    window_duration: Duration,
    max_tpm: (usize,usize),
}

#[derive(Debug)]
pub struct AnthropicClient {
    client: Client,
    api_key: String,
    url: String,
    api_version: String,
    default_model: String,
    max_tokens: usize,
    pub usage_monitor: AnthropicUsageMonitor,
}

// object impls

impl AnthropicUsageMonitor {
    pub fn new() -> Self {
        let max_tpm: (usize,usize) = (
            crate::common::config
                ::get_config("anthropic_config.json", "max_input_tpm")
                .expect("failed get from config"),
            crate::common::config
                ::get_config("anthropic_config.json", "max_output_tpm")
                .expect("failed get from config"),
        );
        Self {
            usage_history: Arc::new(Mutex::new(VecDeque::new())),
            window_duration: Duration::from_secs(60),
            max_tpm,
        }
    }

    pub fn record(&self, usage: Usage) -> () {
        let mut history = self.usage_history.lock()
            .expect("failed to acquire mutex");
        history.push_back(usage);
    }

    pub fn max_tpm(&self) -> (usize,usize) { self.max_tpm.clone() }

    pub fn tpm(&self) -> (usize,usize) {
        let mut history = self.usage_history.lock()
            .expect("failed to acquire mutex");
        let now = Instant::now();
        let cutoff = now - self.window_duration;

        while let Some(usage) = history.front() {
            if usage.timestamp < cutoff {
                history.pop_front();
            } else {
                break;
            }
        }

        let input_tokens = history.iter().map(|usage| usage.input_tokens).sum();
        let output_tokens = history.iter().map(|usage| usage.output_tokens).sum();

        (input_tokens, output_tokens)
    }

    #[allow(unused)]
    pub fn tpm_str(&self) -> String {
        let tpm = self.tpm();
        let max_tpm = self.max_tpm();
        let pct_in = 100.0 * tpm.0 as f64 / max_tpm.0 as f64;
        let pct_out = 100.0 * tpm.1 as f64 / max_tpm.1 as f64;
        format!(
            "in: {:5} / {:5} ({:.1}%), out: {:5} / {:5} ({:.1}%)",
            tpm.0, max_tpm.0, pct_in, 
            tpm.1, max_tpm.1, pct_out,
        )
    }
}

impl AnthropicClient {

    // public

    /// Create a new `AnthropicClient` instance
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let env_path = crate::common::config::PROJECT_DIRS.config_dir().join(".env");
        dotenvy::from_path(env_path).ok();

        let api_key = std::env::var("ANTHROPIC_API_KEY")?;
        Ok(Self {
            client: ClientBuilder::default().build()?,
            api_key: api_key,
            url: crate::common::config
                ::get_config("anthropic_config.json", "base_url")?,
            api_version: crate::common::config
                ::get_config("anthropic_config.json", "anthropic_version")?,
            default_model: crate::common::config
                ::get_config("anthropic_config.json", "default_model")?,
            max_tokens: crate::common::config
                ::get_config("anthropic_config.json", "max_tokens")?,
            usage_monitor: AnthropicUsageMonitor::new()
        })
    }

    /// Sends a list of messages, system prompt, and randomness (temperature) to the LLM and returns the response
    pub async fn call_model(
        &self, 
        messages: &Vec<Message>, 
        sys_prompt: &str, 
        tools: Option<&Vec<AnthropicToolDefinition>>, 
        model: Option<String>,
        randomness: f64
    ) 
    -> Result<MessagesResponse,Box<dyn std::error::Error>> {

        let m = match model {
            Some(model) => model,
            None => self.default_model.clone()
        };

        let sp = if !sys_prompt.is_empty() { 
            build_ephemeral_sys_prompt(sys_prompt) 
        } else { 
            serde_json::json!("") 
        };

        let t = if let Some(t) = tools {
            build_ephemeral_tools(t)
        } else {
            serde_json::json!([])
        };

        let request_json = serde_json::json!({
            "model": &m, 
            "max_tokens": self.max_tokens, 
            "temperature": randomness,
            "messages": &messages[..],
            "stream": false, 
            "system": sp,
            "tools": t
        });

        let response_json = self.client
            .post(format!("{}/messages", &self.url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("content-type", "application/json")
            .json(&request_json)
            .send().await?
            .json::<serde_json::Value>().await?;

        let _ = response_json.as_object()
            .ok_or("Response was null")?;

        if let Some(e) = response_json.get("error") {
            Err(format!("{e}").into())
        } else {
            let usage_json = response_json.get("usage")
                .ok_or("Response does not contain usage")?
                .clone();

            let usage: AnthropicUsage = serde_json::from_value(usage_json)?;
            let response = serde_json::from_value(response_json)?;
            self.usage_monitor.record(usage.clone().into());

            // tpm logger message
            let (tpm_in,tpm_out) = self.usage_monitor.tpm();
            let (mtpm_in,mtpm_out) = self.usage_monitor.max_tpm();

            let price_map = HashMap::from([
                ("claude-haiku-4-5-20251001".to_string(),   (1.0f32,  5.0f32)),
                ("claude-sonnet-4-5-20250929".to_string(),  (3.0f32, 15.0f32)),
                ("claude-opus-4-5-20251101".to_string(),    (5.0f32, 25.0f32)),
            ]);

            let (ppmt_in, ppmt_out) = price_map.get(&m)
                .expect("invalid model (not in price map, this should be unreachable)");

            let p_in = (usage.input_tokens as f32) * ppmt_in / 1_000_000f32;
            let p_out = (usage.output_tokens as f32) * ppmt_out / 1_000_000f32;
            log::info!(r#"Usage (for model {}):
input tokens:   {:9} tokens => ${:.6}
output tokens:  {:9} tokens => ${:.6}
cost for this prompt: ${:.6}"#,
                m, usage.input_tokens, p_in,
                usage.output_tokens, p_out,
                p_in+p_out,
            );

            let ppm_in = (tpm_in as f32) * ppmt_in / 1_000_000f32;
            let ppm_out = (tpm_out as f32) * ppmt_out / 1_000_000f32;
            log::info!(r#"Usage per minute (for model {}):
tokens per minute in:     {:6} / {:6} ({:.2}%) => ${:.6} / min
tokens per minute out:    {:6} / {:6} ({:.2}%) => ${:.6} / min
cost for the past minute: ${:.6}"#,
                m, tpm_in, mtpm_in, (tpm_in as f32) / (mtpm_in as f32) * 100f32,
                ppm_in,
                tpm_out, mtpm_out, (tpm_out as f32) / (mtpm_out as f32) * 100f32,
                ppm_out,
                ppm_in+ppm_out,
            );

            Ok(response)
        }
    }
}

// private helpers

fn build_ephemeral_sys_prompt(sys_prompt: &str) -> serde_json::Value {
    serde_json::json!([{
        "type": "text",
        "text": sys_prompt,
        "cache_control": { "type": "ephemeral" }
    }])
}

fn build_ephemeral_tools(tools: &[AnthropicToolDefinition]) -> serde_json::Value {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[tokio::test]
    async fn test_call_model() -> () {
        let _ = env_logger::builder().filter_level(log::LevelFilter::Debug).try_init();

        let client = AnthropicClient::new().expect("error building client");

        thread::sleep(Duration::from_secs(7));

        let _response = client.call_model(
            &vec![
                Message {
                    role: Role::User,
                    content: vec![
                        ContentBlock::Text { text: vec!["Hello! "; 2000].join("")}
                    ]
                }; 8
            ], 
            "This is the sys prompt", 
            None, 
            None, 
            0.0
        ).await
            .expect("anthropic failed");

        thread::sleep(Duration::from_secs(14));
    }
}