use std::collections::HashMap;

use crate::{connection::anthropic_client::ToolDefinition, mcp::{McpContent, McpTool, client::McpClient}};

#[derive(Debug)]
pub struct McpManager {
    clients: HashMap<String, McpClient>,
    tools: Vec<McpTool>
}

impl McpManager {

    pub fn new() -> Self {
        Self {
            clients: HashMap::new(),
            tools: Vec::new()
        }
    }

    pub fn register_server(&mut self, name: String, command: &str, args: &[&str]) 
    -> Result<(), Box<dyn std::error::Error>> {
        let mut client = McpClient::new(command, args)?;
        let tools = client.list_tools()?;

        log::info!("Registered {} tools from MCP server '{}'", tools.len(), name);

        self.tools.extend(tools);
        self.clients.insert(name, client);

        Ok(())
    }

    pub fn get_tools_as_anthropic(&self) -> Vec<ToolDefinition> {
        self.tools.iter()
            .map(|t| ToolDefinition {
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: t.input_schema.clone(),
            })
            .collect()
    }

    pub fn execute_tool(&mut self, name: &str, args: serde_json::Value) 
    -> Result<Option<String>, Box<dyn std::error::Error>> {
        for client in self.clients.values_mut() {
            if let Ok(result) = client.call_tool(name, args.clone()) {
                let text = result
                    .content
                    .iter()
                    .filter_map(|c| match c {
                        McpContent::Text { text } => Some(text.clone()),
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                return Ok(Some(text));
            }
        }
        Err("No tool found".into())
    }

}