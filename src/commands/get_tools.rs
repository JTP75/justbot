use crate::app::{bot::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct GetToolsCommand;

impl Command for GetToolsCommand {
    fn name(&self) -> &str { "get-tools" }
    fn aliases(&self) -> Vec<&str> { vec!["tools", "list-tools"] }
    fn desc(&self) -> &str { "List all tools (MCP and Integrated) currently available to anthropic (see message-tool)" }
    fn help(&self) -> &str { "Usage: get-tools\nTakes no arguments" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, _args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let tool_disp = bot.get_current_tools()
            .iter()
            .map(|tooldef| format!("{:30} {}", tooldef.name, tooldef.description))
            .collect::<Vec<_>>()
            .join("\n\t");
        Ok(Some(format!("Currently available tools:\n\t{}", tool_disp)))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "get-tools".into(), 
        || Box::new(GetToolsCommand),
    );
}