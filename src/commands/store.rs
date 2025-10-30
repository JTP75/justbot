use std::path::PathBuf;

use crate::rustbot::{bot::RustBot, session::SessionManager};

use super::{Command, REGISTRY};

pub struct StoreCommand;

impl Command for StoreCommand {
    fn name(&self) -> &str { "store" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Store a single document (file) to the vector database" }
    fn help(&self) -> &str { "Usage: store <full/path/to/file>\nFile must contain readable text" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let collection_name: String = crate::common::config
            ::get_config("bot_config.json", "default_collection")?;
        let path_str = args.first().ok_or("Must specify the path to the file to store")?;
        let path = PathBuf::from(path_str);

        bot.store_file(&collection_name, path)?;

        Ok(Some(format!("Successfully stored file to collection: {}.", collection_name)))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "store".into(), 
        || Box::new(StoreCommand),
    );
}