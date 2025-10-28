use crate::rustbot::{bot::RustBot, session::SessionManager};

use super::{Command, REGISTRY};

pub struct LoadCommand;

impl Command for LoadCommand {
    fn name(&self) -> &str { "load" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Load a session file over the current session" }
    fn help(&self) -> &str { "Usage: load <filename>\nFilename must be specified" }
    fn exec(&self, sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let filename = match args.first() {
            Some(filename) => filename.to_string(),
            None => { return Err(format!("Must specify file to load.").into()) }
        };

        match sm.load_session(bot,filename) {
            Ok(_) => Ok(Some(format!("Loaded session from {}", sm.current.clone().as_os_str().display()))),
            Err(e) => Err(format!("Failed to load session from {}: {}", sm.current.clone().as_os_str().display(),e).into())
        }        
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "load".into(), 
        || Box::new(LoadCommand),
    );
}