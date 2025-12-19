use std::fs;

use crate::{app::{puetce::PuetceApp, session::SessionManager}, common::config_const::{json::{EMBEDDING_CONFIG, VECTORDB_CONFIG}, keys::{DEFAULT_COLLECTION, ENABLE_PDF_EMBEDDING}}};

use super::{Command, REGISTRY};

pub struct StoreCommand;

impl Command for StoreCommand {
    fn name(&self) -> &str { "store" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Store a path to the vector database\n\n\t- if the path resolves to a file, that file will be stored. \n\t- if it resolves to a directory, this command will recursively store all files in the directory to the vector database.\n\nThis currently works for UTF-8 encoded files." }
    fn help(&self) -> &str { "Usage: store <full/path/to/file>\nFile must contain readable text" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let collection_name: String = match bot.get_current_collection() {
            Some(cn) => cn,
            None => crate::common::config
                ::get_config(VECTORDB_CONFIG, DEFAULT_COLLECTION)?,
        };
        let default_path = ".".to_string(); 
        let path_str = args.first().unwrap_or(&default_path);
        let path = bot.resolve_file_path_str(path_str)?;

        // try to add collection (if it already exists, ignore error)
        bot.add_collection(&collection_name).unwrap_or(());

        if path.is_dir() {
            log::info!("This is a directory. {:?}", path);


            let convert_pdfs: bool = crate::common::config
                ::get_config(EMBEDDING_CONFIG, ENABLE_PDF_EMBEDDING)?;

            // get list of files in directory
            let paths = fs::read_dir(path)?
                .filter_map(|rslt| rslt.ok())
                .map(|dir_entry| dir_entry.path())
                .filter(|path| path.is_file())
                .filter(|path| 
                    (convert_pdfs && path.extension().and_then(|ext| ext.to_str()) == Some("pdf")) ||
                    String::from_utf8(fs::read(path).unwrap_or("".into())).is_ok()
                )
                .collect::<Vec<_>>();

            let paths_disp = 
                paths.iter().map(|p| p.to_str().unwrap())
                    .collect::<Vec<_>>()
                    .join("\n\t");
            log::info!("Files:\n\t{}", paths_disp);

            // store files
            bot.store_files(&collection_name, paths.iter().map(|path| path.as_path()).collect())?;

            Ok(Some(format!(
                "Successfully stored the files to collection: {}.\n\t{}", 
                collection_name,
                paths_disp
            )))
        } else if path.is_file() {
            log::info!("This is a file. {:?}", path);

            // store file
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