use crate::rustbot::bot::RustBot;

use super::{Command, REGISTRY};

pub struct HelloCommand;

impl Command for HelloCommand {
    fn name(&self) -> &str {
        "hello"
    }

    fn desc(&self) -> &str {
        "Print a friendly greeting"
    }

    fn help(&self) -> &str {
        "Usage: hello\nTakes no arguments"
    }

    fn exec(&self, bot: &mut RustBot, _args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("Hello there! My name is {}.", bot.get_name())))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "hello".into(), 
        || Box::new(HelloCommand),
    );
}