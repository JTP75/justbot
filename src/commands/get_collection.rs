use crate::app::{puetce::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct GetCollectionCommand;

impl Command for GetCollectionCommand {
    fn name(&self) -> &str { "get-collection" }
    fn aliases(&self) -> Vec<&str> { vec!["getc"] }
    fn desc(&self) -> &str { "Get information about the currently selected vector database collection" }
    fn help(&self) -> &str { "Usage: get-collection\nTakes no arguments" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, _args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match bot.get_current_collection() {
            Some(c) => Ok(Some(format!("The current collection is '{c}'"))),
            None => Ok(Some(format!("No collection is currently selected. Set using set-collection")))
        }
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "get-collection".into(), 
        || Box::new(GetCollectionCommand),
    );
}