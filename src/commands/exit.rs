use crate::app::{bot::RustBot, session::SessionManager};

use super::{Command, REGISTRY};

pub struct ExitCommand;

impl Command for ExitCommand {
    fn name(&self) -> &str { "exit" }
    fn aliases(&self) -> Vec<&str> { vec![ "q","quit" ] }
    fn desc(&self) -> &str { "Print a friendly farewell" }
    fn help(&self) -> &str { "Usage: exit\nTakes no arguments" }
    fn exec(&self, _sm: &mut SessionManager, _bot: &mut RustBot, _args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("Exiting. Goodbye!")))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "exit".into(), 
        || Box::new(ExitCommand),
    );
}