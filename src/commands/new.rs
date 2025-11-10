use std::path::PathBuf;

use crate::app::{bot::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct NewCommand;

impl Command for NewCommand {
    fn name(&self) -> &str { "new" }
    fn aliases(&self) -> Vec<&str> { vec!["clear"] }
    fn desc(&self) -> &str { "Clear the anthropic messages and create a new session. This also clears the topic." }
    fn help(&self) -> &str { "Usage: new [<topic_name>]\nOptionally specify a topic name for conversation." }
    fn exec(&self, sm: &mut SessionManager, bot: &mut PuetceApp, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        log::info!("Clearing current session");
        bot.set_topic( match args.first() {Some(tn) => tn, None => ""});
        bot.set_messages(Vec::new());
        sm.current = PathBuf::new();
        log::info!("New session created");
        Ok(None)
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "new".into(), 
        || Box::new(NewCommand),
    );
}