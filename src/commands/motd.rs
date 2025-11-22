use crate::app::{puetce::PuetceApp, session::SessionManager};

use super::{Command, REGISTRY};

pub struct MotdCommand;

impl Command for MotdCommand {
    fn name(&self) -> &str { "motd" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Display today's motd (Message of the day)" }
    fn help(&self) -> &str { "Usage: motd [<options>]\n--reroll\t- Generate a new motd" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, _args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let motd = bot.get_motd();

         Ok(motd.1)
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "motd".into(), 
        || Box::new(MotdCommand),
    );
}