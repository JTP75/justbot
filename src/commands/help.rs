use std::collections::HashSet;

use crate::app::{bot::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct HelpCommand;

impl Command for HelpCommand {
    fn name(&self) -> &str { "help" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Get help on command usage" }
    fn help(&self) -> &str { "Usage: help [<command>]\nShows help and desc for specified command. If no command is specified, lists available commands." }
    fn exec(&self, _sm: &mut SessionManager, _bot: &mut PuetceApp, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        match args.first() {
            Some(command_name) => {
                let command = match REGISTRY.lock().unwrap().get(&command_name) {
                    Some(command) => command,
                    None => {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("command '{}' not found", command_name),
                        )));
                    }
                };
                Ok(Some(format!("{}\n\nDESC\n{}\n\nHELP\n{}\n", command_name, command.desc(), command.help())))
            },
            None => {
                let mut seen = HashSet::new();
                let commands = REGISTRY.lock().unwrap()
                    .get_commands()
                    .iter()
                    .filter(|command| seen.insert(command.name().to_string()))
                    .map(|command| format!("{:20}\taliases=({})", command.name(), command.aliases().join(" | ")))
                    .collect::<Vec<_>>()
                    .join("\n\t");
                Ok(Some(format!("Available commands are: \n\t{}", commands)))
            },
        }
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "help".into(), 
        || Box::new(HelpCommand),
    );
}