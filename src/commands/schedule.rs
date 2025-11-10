use chrono::Local;

use crate::{connection::anthropic_client::{ContentBlock, Message, Role}, app::{bot::RustBot, session::SessionManager}};

use super::{Command, REGISTRY};

pub struct ScheduleCommand;

impl Command for ScheduleCommand {
    fn name(&self) -> &str { "schedule" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Send a message to claude using anthropic api with tools enable and scheduling default sys prompt" }
    fn help(&self) -> &str { "Usage: schedule <message>\nMessage does not need to be in quotation marks" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let user_message = Message{
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: args.join(" ")
            }]
        };
        bot.push_message(user_message);

        let sys_prompt: String = crate::common::config
            ::get_config("bot_config.json", "tool_sys_prompt")?;

        let instructions = serde_json::to_string_pretty(&serde_json::json!({
            "general instructions": "These are default instructions for creating calendar events if the user does not specify certain fields. The goal is to make it so the user does not need to be specific when scheduling events. There are a few fields that are mandatory.",
            "mandatory fields": ["title", "date"],
            "useful info": [
                format!("The current datetime (rfc3339) is {}", Local::now().to_rfc3339()),
                format!("The user's current time zone is {}", Local::now().offset()),
            ],
            "default behaviors": {
                "start_time": "If the user does not specify a start_time, assume it is an all-day event.",
                "duration or end_time": "If the user does not specify a duration or end_time, default to 30 minutes after start_time.",
                "participants": "If there isnt enough information to determine participants, leave the field empty.",
                "location": "If the user does not specify a location, leave the field empty.",
                "reminders": "If the user does not specify reminders, set a default reminder for 10 minutes before the event.",
            }
        }))?;

        let response = bot.query_llm_with_tools(&bot.get_messages(), &format!("{}\n{{CALENDAR EVENT CREATION INSTRUCTIONS}}\n{}", sys_prompt, instructions), 0.75)?;

        let mut agent_message = Message {
            role: Role::Assistant,
            content: response.content
        };

        let response_text: String = match agent_message.content.first() {
            Some(ContentBlock::Text { text }) => text.into(),
            Some(_) => "Unexpected content block type".into(),
            None => {
                agent_message.content.push(ContentBlock::Text { text: "Null".into() });
                "Null response from agent".into()
            }
        };

        bot.push_message(agent_message.clone());

        Ok(Some(response_text))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "schedule".into(), 
        || Box::new(ScheduleCommand),
    );
}