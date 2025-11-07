use crate::{common::types::{ContentBlock, Message, Role}, rustbot::{bot::RustBot, session::SessionManager}};

use super::{Command, REGISTRY};

pub struct NewsCommand;

impl Command for NewsCommand {
    fn name(&self) -> &str { "news" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "Use claude to retrieve news related to a topic" }
    fn help(&self) -> &str { "Usage: news [<query>]\nQuery is optional. Use it to find news articles related to a particular or general topic, focus on particular article, or retrieve info about a certain article" }
    fn exec(&self, _sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let user_message = Message{
            role: Role::User,
            content: vec![ContentBlock::Text {
                text: format!(
                    "Instructions:\n{}Preferred News Sources\n{}\nQuery:\n{}", 
                    r#"Use the rss tool to find news articles and show them to the user. Here are a few criteria and instructions:
                    - prioritize using the user's preferred news sources over others
                    - prioritize more recent news
                    - if the user doesnt specify a topic, pick articles related to US politics, science, and international news
                    - if the user doesnt specify how many articles to find, retrieve 15 headlines
                    - if the user specifies a topic that isnt covered by the list of preferred sources, choose one"#,
                    serde_json::to_string_pretty(&serde_json::json!({
                        "New York Times: US News": "https://rss.nytimes.com/services/xml/rss/nyt/US.xml",
                        "New York Times: US Politics": "https://rss.nytimes.com/services/xml/rss/nyt/Politics.xml",
                        "MIT Technology Review": "https://www.technologyreview.com/feed/",
                        "Nature Materials": "https://www.nature.com/nmat.rss"
                    }))?,
                    if !args.is_empty() { args.join(" ") } else { 
                        r#"**This is the default query** Retrieve 15 news articles related to US politics (5), science (5), and
                        international news (5) in the last week."#.into()
                    }
                )
            }]
        };
        bot.push_message(user_message);

        let sys_prompt: String = crate::common::config
            ::get_config("bot_config.json", "tool_sys_prompt")?;
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
        "news".into(), 
        || Box::new(NewsCommand),
    );
}