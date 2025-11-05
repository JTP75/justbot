pub mod client;
// pub mod manager;

use serde::{Deserialize, Serialize};

use crate::connection::anthropic_client::ToolDefinition;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: serde_json::Value,
}

impl Into<ToolDefinition> for McpTool {
    fn into(self) -> ToolDefinition {
        ToolDefinition { 
            name: self.name, 
            description: self.description, 
            input_schema: self.input_schema
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct McpToolResult {
    pub content: Vec<McpContent>,
    #[serde(rename = "isError")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpContent {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct McpServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: Option<std::collections::HashMap<String,String>>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct McpConfig {
    pub mcp_servers: Vec<McpServerConfig>,
}