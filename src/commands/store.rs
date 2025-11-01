use std::{fs, path::PathBuf};

use crate::rustbot::{bot::RustBot, session::SessionManager};

use super::{Command, REGISTRY};

pub struct StoreCommand;

impl Command for StoreCommand {
    fn name(&self) -> &str { "store" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Store a path to the vector database\n\n\t- if the path resolves to a file, that file will be stored. \n\t- if it resolves to a directory, this command will recursively store all files in the directory to the vector database.\n\nThis currently works for UTF-8 encoded files." }
    fn help(&self) -> &str { "Usage: store <full/path/to/file>\nFile must contain readable text" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let collection_name: String = match bot.get_current_collection() {
            Some(cn) => cn,
            None => crate::common::config
                ::get_config("bot_config.json", "default_collection")?,
        };
        let default_path = ".".to_string(); 
        let path_str = args.first().unwrap_or(&default_path);
        let path = PathBuf::from(path_str).canonicalize()?;

        bot.add_collection(&collection_name).unwrap_or(()); // ignore errors

        if path.is_dir() {
            // THIS MIGHT TAKE A LONG TIME WITHOUT CHUNKED CALLS, DONT IMPLEMENT YET
            // ALSO NEED TO CHECK TO MAKE SURE THIS ISNT RUNNING IN LARGE DIRECTORIES (i.e "/")
            // 
            // TLDR: MAKE THIS KILLABLE
            //
            // should probably implement a function in bot.rs like "store_files" or "store_bulk"
            //
            // current solution is to have this be non recursive
            log::info!("This is a directory. {:?}", path);

            let paths = fs::read_dir(path)?
                .filter_map(|rslt| rslt.ok())
                .map(|dir_entry| dir_entry.path())
                .filter(|path| path.is_file())
                .filter(|path| 
                    path.extension().and_then(|ext| ext.to_str()) == Some("pdf") ||
                    String::from_utf8(fs::read(path).unwrap_or("".into())).is_ok()
                )
                .collect::<Vec<_>>();

            log::info!(
                "Files:\n\t{}", 
                paths.iter().map(|p| p.to_str().unwrap())
                    .collect::<Vec<_>>()
                    .join("\n\t")
            );

            // Ok(Some(format!(
            //     "Successfully stored the files to collection: {}.\n\t{}", 
            //     collection_name,
            //     paths.iter.map().join("\n\t")
            // )))
            Err("Not implemented for directories".into())
        } else if path.is_file() {
            let path = bot.resolve_file_path_str(path_str)?;
            log::info!("This is a file. {:?}", path);
            bot.store_file(&collection_name, &path)?;
            Ok(Some(format!(
                "Successfully stored file to collection: {}.\n\t{}", 
                collection_name, 
                path.display()
            )))
        } else {
            Err("Canonicalized path is not valid".into())
        }
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "store".into(), 
        || Box::new(StoreCommand),
    );
}