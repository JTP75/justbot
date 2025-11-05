//! Integrates MCP Tools and Integrated Tools

use std::collections::HashMap;

use crate::{connection::anthropic_client::ToolDefinition, mcp::{McpContent, McpTool, client::McpClient}, rustbot::bot::RustBot, tools::{self, Tool}};

pub struct ToolManager {
    mcp_clients: HashMap<String, McpClient>,
    mcp_tools: Vec<McpTool>,
    integrated_tools: Vec<Box<dyn Tool>>,
    tool_client_map: HashMap<String, String>
}

impl std::fmt::Debug for ToolManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolManager")
            .field("mcp_clients", &self.mcp_clients)
            .field("mcp_tools", &self.mcp_tools)
            .field("integrated_tools", &"<Vec<Box<dyn Tool>>>")
            .finish()
    }
}

impl ToolManager {

    pub fn new() -> Self {
        let itools = tools::REGISTRY.lock().unwrap().tools();
        Self {
            mcp_clients: HashMap::new(),
            mcp_tools: Vec::new(),
            integrated_tools: itools,
            tool_client_map: HashMap::new()
        }
    }

    pub fn register_mcp_server(&mut self, name: String, command: &str, args: &[&str], env: Option<HashMap<String, String>>) 
    -> Result<(), Box<dyn std::error::Error>> {
        let mut mcp_client = McpClient::new(command, args, env)?;
        let mcp_tools = mcp_client.list_tools()?;

        log::info!("Registered {} tools from MCP server '{}'", mcp_tools.len(), name);

        for mcp_tool in mcp_tools.iter() {
            self.tool_client_map.insert(mcp_tool.name.clone(), name.clone());
        }
        self.mcp_clients.insert(name, mcp_client);
        self.mcp_tools.extend(mcp_tools);

        Ok(())
    }

    pub fn get_mcp_tooldefs(&self) -> Vec<ToolDefinition> {
        self.mcp_tools.iter()
            .map(|t| t.clone().into())
            .collect()
    }

    pub fn get_integrated_tooldefs(&self) -> Vec<ToolDefinition> {
        self.integrated_tools.iter()
            .map(|t| t.as_tooldef())
            .collect()
    }

    pub fn get_tools_as_tooldefs(&self) -> Vec<ToolDefinition> {
        let mut tooldefs = self.get_mcp_tooldefs();
        let itooldefs = self.get_integrated_tooldefs();
        tooldefs.extend(itooldefs);
        tooldefs
    }

    /// 
    /// 
    /// - this only supports tools that return text for now
    /// - this will prioritize integrated tools if there are conflicting names
    pub fn execute_tool(&mut self, bot: &mut RustBot, tool_name: &str, args: &serde_json::Value) 
    -> Result<Option<String>, Box<dyn std::error::Error>> {
        if self.get_integrated_tooldefs().iter().any(|it| it.name==tool_name) {
            log::info!("Executing integrated tool '{}'", tool_name);
            Ok(self.execute_integrated_tool(bot, tool_name, args)
                .map_err(|e| format!("Execution failed: {e}"))?)
        } else if self.get_mcp_tooldefs().iter().any(|mt| mt.name==tool_name) {
            log::info!("Executing MCP tool '{}' ", tool_name);
            Ok(self.execute_mcp_tool(bot, tool_name, args)
                .map_err(|e| format!("Execution failed: {e}"))?)
        } else {
            log::error!("Tool '{}' not found in mcp or integrated tooldefs", tool_name);
            Err("Tool not found in tooldefs".into())
        }
    }

    pub fn execute_mcp_tool(&mut self, _bot: &mut RustBot, t_name: &str, args: &serde_json::Value) 
    -> Result<Option<String>, Box<dyn std::error::Error>> {
        let c_name = self.tool_client_map.get(t_name)
            .ok_or(format!("No registered client for tool '{t_name}'"))?;
        let client = self.mcp_clients.get_mut(c_name)
            .ok_or(format!("No valid client with name '{c_name}'"))?;
        
        let tool_rslt = client.call_tool(t_name, args)?;
        let text = tool_rslt.content.iter()
            .filter_map(|c| match c { 
                McpContent::Text { text } => Some(text.clone()), 
            })
            .collect::<Vec<_>>()
            .join("\n");

        Ok(Some(text))
    }

    pub fn execute_integrated_tool(&mut self, bot: &mut RustBot, t_name: &str, args: &serde_json::Value)
    -> Result<Option<String>, Box<dyn std::error::Error>> {
        let tool = self.integrated_tools.iter().find(|it| it.name()==t_name)
            .ok_or(format!("No registered tool with name '{t_name}'"))?;
        tool.exec(bot,args)
    }
}