use std::{env, path::PathBuf};

use crate::app::{bot::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct CdCommand;

impl Command for CdCommand {
    fn name(&self) -> &str { "cd" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Changes the cwd of rustbot to a new directory" }
    fn help(&self) -> &str { "Usage: cd <new_directory>\nNew directory to navigate to." }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let path_str = args.first().ok_or("Must specify new location")?;
        let path = PathBuf::try_from(path_str)?;
        let path = path.canonicalize()?;
        if !path.is_dir() { 
            return Err(format!("'{}' is not a directory", path.display()).into()); 
        }
        env::set_current_dir(&path)?;
        bot.set_cwd(path);
        Ok(Some(format!("Now in {}", bot.get_cwd().display())))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "cd".into(), 
        || Box::new(CdCommand),
    );
}