use crate::app::{bot::RustBot, session::SessionManager};

use super::{Command, REGISTRY};

pub struct WExitCommand;

impl Command for WExitCommand {
    fn name(&self) -> &str { "wexit" }
    fn aliases(&self) -> Vec<&str> { vec!["wq"] }
    fn desc(&self) -> &str { "Save the current conversation and print a friendly farewell" }
    fn help(&self) -> &str { "Usage: wexit\nTakes no arguments" }
    fn exec(&self, sm: &mut SessionManager, bot: &mut RustBot, _args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match sm.save_session(bot,None) {
            Ok(_) => Ok(Some(format!("Saved session to {}. Goodbye!", sm.current.clone().as_os_str().display()))),
            Err(e) => Err(format!("Failed to save session to {}: {}. Goodbye!", sm.current.clone().as_os_str().display(),e).into())
        }        
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "wexit".into(), 
        || Box::new(WExitCommand),
    );
}