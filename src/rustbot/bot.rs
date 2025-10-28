use anthropic::types::Message;
use chrono::{self, Local};

use crate::commands::{Command, REGISTRY};
use crate::connection::anthropic_client::AnthropicClient;
use crate::rustbot::session::SessionManager;

pub const DEFAULT_NAME: &str = "\x1b[0;33mrustbot\x1b[0m";

#[derive(Debug)]
pub struct RustBot {

    // immut fields
    name: String,
    client: AnthropicClient,

    // state
    topic: String,
    messages: Vec<Message>,
    motd: (chrono::NaiveDate, Option<String>),
    _date: chrono::NaiveDate,

    _input_tokens: Vec<usize>,
    _output_tokens: Vec<usize>,
    _total_tokens: Vec<usize>,
}

impl RustBot {

    // public

    /// Creates a RustBot with the given name.
    ///
    /// # Examples
    ///
    /// ```
    /// // create a new RustBot instance
    /// use rustbot::RustBot;
    /// let bot = RustBot::new("name");
    /// ```
    pub fn new(name: impl Into<String>) -> Self {
        Self { 
            name: name.into(), 
            client: AnthropicClient::new().unwrap(),
            
            topic: "".into(),
            messages: vec![],
            motd: (Local::now().date_naive(), None),
            _date: Local::now().date_naive(),

            _input_tokens: vec![],
            _output_tokens: vec![],
            _total_tokens: vec![],
        }
    }

    pub fn handle_command(&mut self, sm: &mut SessionManager, input: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let (command,args) = self.parse_command(input)?;
        command.exec(sm, self, &args)
    }

    pub fn get_name(&self) -> String { self.name.clone() }

    pub fn get_topic(&self) -> String { self.topic.clone() }

    pub fn set_topic(&mut self, topic: impl Into<String>) -> () { self.topic = topic.into(); }

    pub fn get_messages(&self) -> Vec<Message> { self.messages.clone() }
    
    pub fn set_messages(&mut self, messages: Vec<Message>) -> () { self.messages = messages; }

    pub fn push_message(&mut self, message: Message) -> () { self.messages.push(message) }

    pub fn get_anthropic_client(&self) -> &AnthropicClient { &self.client }

    pub fn get_motd(&self) -> (chrono::NaiveDate, Option<String>) { self.motd.clone() }
    
    pub fn set_motd(&mut self, motd: (chrono::NaiveDate, Option<String>)) -> () { self.motd = motd }

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