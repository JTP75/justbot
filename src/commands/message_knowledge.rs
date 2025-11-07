use crate::{common::types::{ContentBlock, Message, Role}, rustbot::{bot::RustBot, session::SessionManager}};

use super::{Command, REGISTRY};

pub struct MessageKnowledgeCommand;

impl Command for MessageKnowledgeCommand {
    fn name(&self) -> &str { "message-knowledge" }
    fn aliases(&self) -> Vec<&str> { vec!["msg-knowledge", "msgk", "knowledge"] }
    fn desc(&self) -> &str { "Send a message to claude using anthropic api with tools enabled with a sys prompt telling claude to use the knowledge" }
    fn help(&self) -> &str { "Usage: message-tool <message>\nMessage does not need to be in quotation marks" }
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
        let sys_prompt = format!(
            "{}\n{{KNOWLEDGE GRAPH MODE}}\n{}", 
            sys_prompt,
            r#"You have been called in knowledge graph mode. Whatever the user asks, you 
            should search knowledge graph first. The contents of your response should prioritize
            knowledge graph info first and general knowledge second.
            
            If the user gives you a greeting in this mode, check the knowledge graph. It may
            contain the user's name along with some personal information to use 
            conversationally."#
        );
        let response = bot.query_llm_with_tools(&bot.get_messages(), &sys_prompt, 0.75)?;

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
        "message-knowledge".into(), 
        || Box::new(MessageKnowledgeCommand),
    );
}