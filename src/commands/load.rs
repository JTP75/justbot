use crate::app::{puetce::PuetceApp, session::SessionManager};
use crate::connection::{ContentBlock, Role};

use super::{Command, REGISTRY};

pub struct LoadCommand;

impl Command for LoadCommand {
    fn name(&self) -> &str { "load" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Load a session file over the current session" }
    fn help(&self) -> &str { "Usage: load <filename>\nFilename must be specified" }
    fn exec(&self, sm: &mut SessionManager, bot: &mut PuetceApp, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let filename = match args.first() {
            Some(filename) => filename.to_string(),
            None => { return Err(format!("Must specify file to load.").into()) }
        };

        match sm.load_session(bot,filename) {
            Ok(_) => {
                let mut past_messages_display = String::new();
                for message in bot.get_messages().iter() {
                    if let Some(cb) = message.content.first() {
                        match cb {
                            ContentBlock::Text { text } => {
                                past_messages_display.push_str(match message.role { 
                                    Role::User => "\n\x1b[1;33m<<\x1b[0m ", 
                                    Role::Assistant => "\n\x1b[1;32m>>\x1b[0m ",
                                });
                                past_messages_display.push_str(&text)
                            },
                            _ => ()
                        }
                    }
                }
                Ok(Some(format!(
                    "Loaded session from {}{}\n", 
                    sm.current.clone().as_os_str().display(),
                    past_messages_display
                )))
            },
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