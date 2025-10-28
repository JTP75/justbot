use anthropic::types::{ContentBlock, MessageBuilder, Role};

use crate::rustbot::bot::RustBot;

use super::{Command, REGISTRY};

pub struct MessageCommand;

impl Command for MessageCommand {
    fn name(&self) -> &str {
        "message"
    }

    fn desc(&self) -> &str {
        "Send a message to claude using anthropic api"
    }

    fn help(&self) -> &str {
        "Usage: message <message>\nMessage does not need to be in quotation marks"
    }

    fn exec(&self, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let user_message = MessageBuilder::default()
            .role(Role::User)
            .content(vec![ContentBlock::Text {
                text: args.join(" ")
            }])
            .build()?;
        bot.push_message(user_message);

        let sys_prompt = format!("Your name is {}. Your name officially has no meaning in particular. You are being called for general use. Your response should always be written in english.", bot.get_name());
        let client = bot.get_anthropic_client();
        let response = client.send_message(&bot.get_messages(), &sys_prompt)?;

        let agent_message = MessageBuilder::default()
            .role(Role::Assistant)
            .content(response.content)
            .build()?;
        bot.push_message(agent_message.clone());

        let response_text = match agent_message.content.first() {
            Some(ContentBlock::Text { text }) => text.to_string(),
            Some(ContentBlock::Image { source: _, media_type: _, data: _ }) => "Unexpected content block type".into(),
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