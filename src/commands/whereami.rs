use crate::app::{puetce::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct WhereAmICommand;

impl Command for WhereAmICommand {
    fn name(&self) -> &str { "whereami" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Prints the cwd of rustbot" }
    fn help(&self) -> &str { "Usage: whereami\nTakes no arguments" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, _args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some(format!("{}", bot.get_cwd().display())))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "whereami".into(), 
        || Box::new(WhereAmICommand),
    );
}