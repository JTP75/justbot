use crate::rustbot::{bot::RustBot, session::SessionManager};

use super::{Command, REGISTRY};

pub struct SaveCommand;

impl Command for SaveCommand {
    fn name(&self) -> &str { "save" }
    fn desc(&self) -> &str { "Save the current claude messages to a file" }
    fn help(&self) -> &str { "Usage: save [<filename>]\nFilename is optional: \n\t- If this a loaded session, the previous save will be overwritten.\n\t- If this is not a loaded session, rustbot will generate a filename." }
    fn exec(&self, sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let filename_arg = match args.first() {
            Some(filename) => Some(filename.to_string()),
            None => None
        };

        match sm.save_session(bot,filename_arg) {
            Ok(_) => Ok(Some(format!("Saved session to {}", sm.current.clone().as_os_str().display()))),
            Err(e) => Err(format!("Failed to save session to {}: {}", sm.current.clone().as_os_str().display(),e).into())
        }        
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "save".into(), 
        || Box::new(SaveCommand),
    );
}