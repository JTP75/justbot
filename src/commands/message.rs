use anthropic::types::{ContentBlock, MessageBuilder, Role};

use crate::rustbot::{bot::RustBot, session::SessionManager};

use super::{Command, REGISTRY};

pub struct MessageCommand;

impl Command for MessageCommand {
    fn name(&self) -> &str { "message" }
    fn aliases(&self) -> Vec<&str> { vec!["msg"] }
    fn desc(&self) -> &str { "Send a message to claude using anthropic api" }
    fn help(&self) -> &str { "Usage: message <message>\nMessage does not need to be in quotation marks" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let user_message = MessageBuilder::default()
            .role(Role::User)
            .content(vec![ContentBlock::Text {
                text: args.join(" ")
            }])
            .build()?;
        bot.push_message(user_message);

        let sys_prompt: String = crate::common::config
            ::get_config("bot_config.json", "base_sys_prompt")?;
        let client = bot.get_chat_client();
        let response = client.call_model(&bot.get_messages(), &sys_prompt, 0.75)?;

        let agent_message = MessageBuilder::default()
            .role(Role::Assistant)
            .content(response.content)
            .build()?;
        bot.push_message(agent_message.clone());

        let response_text: String = match agent_message.content.first() {
            Some(ContentBlock::Text { text }) => text.into(),
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