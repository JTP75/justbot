use anthropic::types::Message;

use crate::commands::{Command, REGISTRY};
use crate::connection::anthropic_client::AnthropicClient;
use crate::rustbot::session::Session;

pub const DEFAULT_NAME: &str = "\x1b[0;33mrustbot\x1b[0m";

#[derive(Debug)]
pub struct RustBot {
    name: String,
    client: AnthropicClient,

    // state data
    topic: String,
    messages: Vec<Message>,
    input_tokens: Vec<usize>,
    output_tokens: Vec<usize>,
    total_tokens: Vec<usize>,
}

impl RustBot {

    // public

    pub fn new(name: impl Into<String>) -> Self {
        Self { 
            name: name.into(), 
            client: AnthropicClient::new().unwrap(),
            topic: "".into(),
            messages: vec![],
            input_tokens: vec![],
            output_tokens: vec![],
            total_tokens: vec![],
        }
    }

    pub fn handle_command(&mut self, input: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let (command,args) = self.parse_command(input)?;
        command.exec(self, &args)
    }

    pub fn get_name(&self) -> String { self.name.clone() }

    pub fn get_topic(&self) -> String { self.topic.clone() }

    pub fn set_topic(&mut self, topic: impl Into<String>) -> () { self.topic = topic.into(); }

    pub fn get_messages(&self) -> Vec<Message> { self.messages.clone() }
    
    pub fn set_messages(&mut self, messages: Vec<Message>) -> () { self.messages = messages; }

    pub fn push_message(&mut self, message: Message) -> () { self.messages.push(message) }

    pub fn get_anthropic_client(&self) -> &AnthropicClient { &self.client }

    // private

    fn parse_command(&self, input: &str) -> Result<(Box<dyn Command>, Vec<String>), Box<dyn std::error::Error>> {
        let tokenized: Vec<&str> =  input.split_whitespace().collect();
        let command_name = tokenized.first().ok_or("User input empty")?;

        let command = REGISTRY.lock().unwrap()
            .get(&command_name)
            .ok_or_else(|| format!("Unknown command: {command_name}"))?;

        let args = tokenized[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>();

        Ok(( command, args ))
    }
}