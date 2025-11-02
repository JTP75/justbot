use std::collections::HashMap;
use once_cell::sync::Lazy;

use crate::rustbot::{bot::RustBot, session::SessionManager};

pub trait Command {
    /// Returns the formal name of the command
    fn name(&self) -> &str;

    /// Returns a Vec of aliases for the command
    fn aliases(&self) -> Vec<&str>;

    /// Returns a short (and AI readable) description for the command
    fn desc(&self) -> &str;

    /// Returns a str containing usage instructions for the command
    fn help(&self) -> &str;

    /// Executes the command
    /// 
    /// Returns response wrapped in a Result and Option
    fn exec(&self, sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>>;
}

pub type CommandFactory = fn() -> Box<dyn Command>;

pub struct CommandRegistry {
    commands: HashMap<String, CommandFactory>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self { commands: HashMap::new() }
    }
    
    pub fn register(&mut self, name: String, factory: CommandFactory) {
        for alias in factory().aliases() {
            self.commands.insert(alias.into(), factory.clone());
        }
        self.commands.insert(name.clone(), factory);
    }

    pub fn get(&self, name: &str) -> Option<Box<dyn Command>> {
        self.commands.get(name).map(|factory| factory())
    }

    pub fn get_commands(&self) -> Vec<Box<dyn Command>> {
        self.commands.values().map(|factory| factory()).collect()
    }
}

pub static REGISTRY: Lazy<std::sync::Mutex<CommandRegistry>> = 
    Lazy::new(|| std::sync::Mutex::new(CommandRegistry::new()));

// register commands

// chatbot
pub mod hello;
pub mod help;
pub mod message;
pub mod message_rag;
pub mod motd;
pub mod store;
pub mod whereami;
pub mod get_collection;
pub mod set_collection;

// session
pub mod save;
pub mod load;
pub mod list;

// terminating
pub mod exit;
pub mod wexit;