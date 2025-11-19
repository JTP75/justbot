use std::fs;

use crate::app::{puetce::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct LsCommand;

impl Command for LsCommand {
    fn name(&self) -> &str { "ls" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "List directories relative to the cwd" }
    fn help(&self) -> &str { "Usage: ls\nTakes no arguments" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, _args: &Vec<String>) 
    -> Result<Option<String>, Box<dyn std::error::Error>> {
        let mut dirs = fs::read_dir(bot.get_cwd())?
            .filter_map(|entry| match entry { Ok(entry)=>Some(entry.path()), _=>None })
            .filter_map(|path| {
                let is_file = path.is_file();
                let is_dir = path.is_dir();
                if let Some(prefix) = bot.get_cwd().parent() {
                    match path.strip_prefix(prefix.to_path_buf()) {
                        Ok(rel_path) => {
                            if is_file {
                                Some(("file",rel_path.to_path_buf()))
                            } else if is_dir {
                                Some(("dir",rel_path.to_path_buf()))
                            } else {
                                Some(("else",rel_path.to_path_buf()))
                            }
                        },
                        _ => None
                    }
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        dirs.sort_by_key(|item| match item.0 {
            "dir" => 0, "file" => 1, _ => 2
        });

        let dirs_disp = dirs.iter()
            .map(|item| match item.0 {
                "dir" => format!("\x1b[0;34m{}\x1b[0m", item.1.display()), 
                "file" => format!("{}", item.1.display()), 
                _ => format!("\x1b[0;32m{}\x1b[0m", item.1.display())
            })
            .collect::<Vec<_>>()
            .join("\n\t");

        Ok(Some(format!("In {}:\n\t{}", bot.get_cwd().display(), dirs_disp)))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "ls".into(), 
        || Box::new(LsCommand),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[ctor::ctor]
    fn setup() {
        log::info!("Starting test server...");
        log::info!("Test server started.");
    }

    #[ctor::dtor]
    fn cleanup() {
        log::info!("Stopping test server...");
        log::info!("Test server stopped.");
    }

    #[test]
    fn test_command() -> () {
        let _cmd = REGISTRY.lock().unwrap().get("ls").unwrap();
        
        // todo this needs a mock bot object
        todo!()
    }
}