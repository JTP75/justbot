//! Integrates MCP Tools and Integrated Tools

use std::{collections::HashMap, sync::Mutex};

use crate::{
    app::connection_manager::ConnectionManager, common::config_const::{json::BOT_CONFIG, keys::ENABLE_CUSTOM_TOOLS}, connection::anthropic_client::{AnthropicToolDefinition, ToolResultContentBlock}, mcp::{McpContent, McpTool, client::McpClient}, tools::{self, Tool}
};

pub struct ToolManager {
    mcp_clients: HashMap<String, Mutex<McpClient>>,
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
        let itools = if crate::common::config
            ::get_config(BOT_CONFIG, ENABLE_CUSTOM_TOOLS).unwrap_or(false) {
            tools::REGISTRY.lock().unwrap().tools()
        } else {
            vec![]
        };
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
        self.mcp_clients.insert(name, Mutex::new(mcp_client));
        self.mcp_tools.extend(mcp_tools);

        Ok(())
    }

    pub fn kill_mcp_servers(&mut self) -> () {
        for (name,client_mutex) in self.mcp_clients.iter() {
            let mut client = client_mutex.lock().unwrap();
            if let Err(e) = client.kill() {
                log::error!("Failed to kill process for '{}': {}", name, e);
                eprintln!("Failed to kill process for '{}': {}", name, e);
            }            
        }
    }

    pub fn get_mcp_tooldefs(&self) -> Vec<AnthropicToolDefinition> {
        self.mcp_tools.iter()
            .map(|t| t.clone().into())
            .collect()
    }

    pub fn get_integrated_tooldefs(&self) -> Vec<AnthropicToolDefinition> {
        self.integrated_tools.iter()
            .map(|t| t.as_tooldef())
            .collect()
    }

    pub fn get_tools_as_tooldefs(&self) -> Vec<AnthropicToolDefinition> {
        let mut tooldefs = self.get_mcp_tooldefs();
        let itooldefs = self.get_integrated_tooldefs();
        tooldefs.extend(itooldefs);
        tooldefs
    }

    /// 
    /// 
    /// - this only supports tools that return text for now
    /// - this will prioritize integrated tools if there are conflicting names
    pub fn execute_tool(&self, cm: &ConnectionManager, tool_name: &str, args: &serde_json::Value) 
    -> Result<Vec<ToolResultContentBlock>, Box<dyn std::error::Error>> {
        if self.get_integrated_tooldefs().iter().any(|it| it.name==tool_name) {
            log::info!("Executing integrated tool '{}'", tool_name);
            let result = self.execute_integrated_tool(cm, tool_name, args)
                .map_err(|e| format!("Execution failed: {e}"));
            match result { 
                Ok(rslt) => Ok(rslt),
                Err(e) => {
                    log::error!("{}", e);
                    Err(e.into())
                }
            }
        } else if self.get_mcp_tooldefs().iter().any(|mt| mt.name==tool_name) {
            log::info!("Executing MCP tool '{}' ", tool_name);
            let result = self.execute_mcp_tool(tool_name, args)
                .map_err(|e| format!("Execution failed: {e}"));
            match result { 
                Ok(rslt) => Ok(rslt),
                Err(e) => {
                    log::error!("{}", e);
                    Err(e.into())
                }
            }
        } else {
            log::error!("Tool '{}' not found in mcp or integrated tooldefs", tool_name);
            Err("Tool not found in tooldefs".into())
        }
    }

    pub fn execute_mcp_tool(&self, t_name: &str, args: &serde_json::Value) 
    -> Result<Vec<ToolResultContentBlock>, Box<dyn std::error::Error>> {
        let c_name = self.tool_client_map.get(t_name)
            .ok_or(format!("No registered client for tool '{t_name}'"))?;
        let client_mutex = self.mcp_clients.get(c_name)
            .ok_or(format!("No valid client with name '{c_name}'"))?;
        let mut client = client_mutex.lock().expect("Failed to acquire mutex");
        
        let tool_rslt = client.call_tool(t_name, args)?;

        let content: Vec<ToolResultContentBlock> = tool_rslt.content.iter()
            .filter_map(|block| match block {
                McpContent::Text { text } => Some(ToolResultContentBlock::Text { text: text.to_string() }),
            })
            .collect();

        if tool_rslt.is_error.is_some() && tool_rslt.is_error.unwrap() {
            match content.get(0) {
                Some(ToolResultContentBlock::Text { text }) => Err(text.as_str().into()),
                Some(_) => Err("This tool returned a non-text content block with the is_error flag set".into()),
                None => Err("This tool returned no content with the is_error flag set".into())
            }
        } else {
            Ok(content)
        }
    }

    pub fn execute_integrated_tool(&self, cm: &ConnectionManager, t_name: &str, args: &serde_json::Value)
    -> Result<Vec<ToolResultContentBlock>, Box<dyn std::error::Error>> {
        let tool = self.integrated_tools.iter().find(|it| it.name()==t_name)
            .ok_or(format!("No registered tool with name '{t_name}'"))?;
        tool.exec(cm, args)
    }
}