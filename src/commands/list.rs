use crate::app::{bot::RustBot, session::SessionManager};

use super::{Command, REGISTRY};

pub struct ListCommand;

impl Command for ListCommand {
    fn name(&self) -> &str { "list" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "List all saved session files in the rustbot data directory" }
    fn help(&self) -> &str { "Usage: list\nTakes no arguments" }
    fn exec(&self, sm: &mut SessionManager, _bot: &mut RustBot, _args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match sm.list_sessions() {
            Ok(sessions) => Ok(Some(format!("Saved sessions in {}\n\t{}", sm.save_dir.display(), sessions[..].join("\n\t")))),
            Err(e) => Err(format!("Failed to load saved sessions in {}: {}", sm.save_dir.display(),e).into())
        }        
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "list".into(), 
        || Box::new(ListCommand),
    );
}