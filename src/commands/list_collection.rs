use crate::app::{puetce::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct ListCollectionCommand;

impl Command for ListCollectionCommand {
    fn name(&self) -> &str { "list-collection" }
    fn aliases(&self) -> Vec<&str> { vec!["listc"] }
    fn desc(&self) -> &str { "Retrieve a list of currently available vector DB collections" }
    fn help(&self) -> &str { "Usage: list-collection\nTakes no arguments" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, _args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut collection_names = bot.get_existing_collections()?;
        collection_names.sort();

        if let Some(current) = bot.get_current_collection() {
            let collection_names = collection_names.into_iter()
                .map(|cname| {
                    if cname==current {
                        format!("\x1b[1m\x08\x08* {cname}\x1b[0m")
                    } else {
                        cname
                    }
                })
                .collect::<Vec<String>>();
            Ok(Some(format!("Available collections:\n\t{}", collection_names.join("\n\t"))))
        } else {
            Ok(Some(format!("Available collections:\n\t{}", collection_names.join("\n\t"))))
        }

    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "list-collection".into(), 
        || Box::new(ListCollectionCommand),
    );
}