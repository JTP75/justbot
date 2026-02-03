use crate::{app::{puetce::PuetceApp, session::SessionManager}, common::config_const::prompts::SYSTEM_BASE, connection::{ContentBlock, Message, Role}};

use super::{Command, REGISTRY};

pub struct MessageCommand;

impl Command for MessageCommand {
    fn name(&self) -> &str { "message" }
    fn aliases(&self) -> Vec<&str> { vec!["msg"] }
    fn desc(&self) -> &str { "Send a message to claude using anthropic api" }
    fn help(&self) -> &str { "Usage: message <message>\nMessage does not need to be in quotation marks" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let user_message = Message{
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: args.join(" ")
            }]
        };
        bot.push_message(user_message);

        let sys_prompt: String = crate::common::config
            ::get_prompt(SYSTEM_BASE)?;
        let sys_prompt = format!(
            "{}\n{{MESSAGE MODE}}\n{}",
            sys_prompt,
            r#"You are being called in message mode. You do not have any tools available. If 
            the user's query is tool-related, you should state that you currently operating in
            message mode, and suggested that they try again using the 'msgt' command."#
        );
        let response = bot.query_llm(&bot.get_messages(), &sys_prompt, 0.75)?;

        let agent_message = Message {
            role: Role::Assistant,
            content: response.content
        };
        bot.push_message(agent_message.clone());

        let response_text: String = match agent_message.content.first() {
            Some(ContentBlock::Text { text }) => text.into(),
            Some(_) => "Unexpected content block type".into(),
            None => "Null response from agent".into()
        };

        Ok(Some(response_text))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "message".into(), 
        || Box::new(MessageCommand),
    );
}