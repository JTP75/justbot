use crate::{connection::anthropic_client::{ContentBlock, Message, Role}, app::{bot::PuetceApp, session::SessionManager}};

use super::{Command, REGISTRY};

pub struct MessageRagCommand;

impl Command for MessageRagCommand {
    fn name(&self) -> &str { "message-rag" }
    fn aliases(&self) -> Vec<&str> { vec!["msg-rag", "rag"] }
    fn desc(&self) -> &str { "Send a message to claude using anthropic api with RAG (retrieval augmented generation)" }
    fn help(&self) -> &str { "Usage: message-rag <message>\nMessage does not need to be in quotation marks. Currently uses default_collection for RAG" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut PuetceApp, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let collection_name: String = match bot.get_current_collection() {
            Some(cn) => cn,
            None => crate::common::config
                ::get_config("bot_config.json", "default_collection")?,
        };

        let user_message = bot.generate_rag_message(&collection_name, &args.join(" "))?;
        bot.push_message(user_message);

        let sys_prompt: String = crate::common::config
            ::get_config("bot_config.json", "rag_sys_prompt")?;
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
        "message-rag".into(), 
        || Box::new(MessageRagCommand),
    );
}