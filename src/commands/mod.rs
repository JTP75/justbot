use std::collections::HashMap;
use once_cell::sync::Lazy;

use crate::rustbot::{bot::RustBot, session::SessionManager};

pub trait Command {
    fn name(&self) -> &str;
    fn aliases(&self) -> Vec<&str>;
    fn desc(&self) -> &str;
    fn help(&self) -> &str;
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

// session
pub mod save;
pub mod load;
pub mod list;

// terminating
pub mod exit;
pub mod wexit;