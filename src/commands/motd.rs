use chrono::Local;

use crate::{connection::anthropic_client::{ContentBlock, Message, Role}, app::{bot::PuetceApp, session::SessionManager}};

use super::{Command, REGISTRY};

pub struct MotdCommand;

impl Command for MotdCommand {
    fn name(&self) -> &str { "motd" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Display today's motd (Message of the day)" }
    fn help(&self) -> &str { "Usage: motd [<options>]\n--reroll\t- Generate a new motd" }
    fn exec(&self, sm: &mut SessionManager, bot: &mut PuetceApp, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        
        let todays_date = Local::now().date_naive();
        let motd = bot.get_motd();

        let reroll = args.first().is_some() && args.first().unwrap().eq("--reroll");

        if motd.0==todays_date && motd.1.is_some() && !reroll {
            Ok(motd.1)
        } else {
            let mut convo_copy = bot.get_messages();
            let motd_prompt: String = crate::common::config
                ::get_config("prompts.json", "motd")?;
            let user_message = Message {
                role: Role::User,
                content: vec![ContentBlock::Text {
                    text: format!("Today is {}. {}", todays_date.format("%A, %m/%d/%Y"), motd_prompt)
                }]
            };
            convo_copy.push(user_message);

            let response = bot.query_llm(&convo_copy, "", 1.0)?;
            let response_text: String = match response.content.first() {
                Some(ContentBlock::Text { text }) => text.into(),
                Some(_) => "Unexpected content block type".into(),
                None => "Null response from agent".into()
            };

            let new_motd = (todays_date, Some(response_text));
            bot.set_motd(new_motd.clone());
            sm.save_motd(bot)?;

            Ok(new_motd.1)
        }
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "motd".into(), 
        || Box::new(MotdCommand),
    );
}