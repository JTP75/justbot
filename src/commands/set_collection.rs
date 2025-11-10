use crate::app::{bot::RustBot, session::SessionManager};

use super::{Command, REGISTRY};

pub struct SetCollectionCommand;

impl Command for SetCollectionCommand {
    fn name(&self) -> &str { "set-collection" }
    fn aliases(&self) -> Vec<&str> { vec!["setc"] }
    fn desc(&self) -> &str { "Set the currently selected collection. The default collection will be used otherwise." }
    fn help(&self) -> &str { "Usage: set-collection [<collection_name>]\nCollection name must consist of alphanumerics, dashes, and underscores only.\nLeaving the field blank will set the current collection to None" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        bot.set_current_collection(args.first().map(|s| s.to_string()));
        Ok(None)
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "set-collection".into(), 
        || Box::new(SetCollectionCommand),
    );
}