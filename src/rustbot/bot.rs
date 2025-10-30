use std::fs;
use std::path::PathBuf;

use anthropic::types::{ContentBlock, Message, MessageBuilder, Role};
use chrono::{self, Local};

use crate::commands::{Command, REGISTRY};
use crate::connection::anthropic_client::AnthropicClient;
use crate::connection::qdrant_client::QdrantClient;
use crate::connection::voyage_client::VoyageClient;
use crate::rustbot::session::SessionManager;

#[derive(Debug)]
pub struct RustBot {

    // immut fields
    name: String,

    chat_client: AnthropicClient,
    vdb_client: QdrantClient,
    embedding_client: VoyageClient,

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

            chat_client: AnthropicClient::new().unwrap(),
            vdb_client: QdrantClient::new().unwrap(),
            embedding_client: VoyageClient::new().unwrap(),
            
            topic: "".into(),
            messages: vec![],
            motd: (Local::now().date_naive(), None),
            _date: Local::now().date_naive(),

            _input_tokens: vec![],
            _output_tokens: vec![],
            _total_tokens: vec![],
        }
    }

    pub fn get_name(&self) -> String { self.name.clone() }
    pub fn get_topic(&self) -> String { self.topic.clone() }
    pub fn get_messages(&self) -> Vec<Message> { self.messages.clone() }
    pub fn get_motd(&self) -> (chrono::NaiveDate, Option<String>) { self.motd.clone() }
    pub fn get_chat_client(&self) -> &AnthropicClient { &self.chat_client }
    pub fn _get_vdb_client(&self) -> &QdrantClient { &self.vdb_client }

    pub fn store_file(&self, collection_name: &str, path: PathBuf) -> Result<(),Box<dyn std::error::Error>> {
        tokio::runtime::Runtime::new()?
            .block_on(self.embed_file(collection_name, path))?;
        Ok(())
    }
    pub fn query_with_rag(&self, collection_name: &str, query: &str)
    -> Result<Message, Box<dyn std::error::Error>> {

        let runtime = tokio::runtime::Runtime::new()?;
        let json_context = runtime.block_on(self.query_vdb(collection_name, query))?;
        let context = serde_json::to_string_pretty(&json_context)?;

        // build and return anthropic message object
        let message = MessageBuilder::default()
            .role(Role::User)
            .content(vec![
                ContentBlock::Text { text: query.into() },
                ContentBlock::Text { text: context }
            ])
            .build()?;

        Ok(message)
    }
    

    pub fn set_topic(&mut self, topic: impl Into<String>) -> () { self.topic = topic.into() }    
    pub fn set_messages(&mut self, messages: Vec<Message>) -> () { self.messages = messages }    
    pub fn set_motd(&mut self, motd: (chrono::NaiveDate, Option<String>)) -> () { self.motd = motd }

    pub fn push_message(&mut self, message: Message) -> () { self.messages.push(message) }
    pub fn handle_command(&mut self, sm: &mut SessionManager, input: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let (command,args) = self.parse_command(input)?;
        command.exec(sm, self, &args)
    }

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

    // this should probably be somewhere else
    async fn embed_file(&self, collection_name: &str, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&path)?;
        let path_str = match path.to_str() {
            Some(s) => s,
            None => { return Err(format!("Error converting path <{}> to &str", path.display()).into()) }
        };
        
        // embed content and path
        //      fixme theres a better way to group embeddings...
        // let json_data = serde_json::json!({"file_path": path_str, "content": content});
        // let text_data = json_data.as_str().ok_or("failed to generate json string")?;
        let text_data = format!("{{\"file_path\": \"{}\", \"content\": \"{}\"}}", path_str, content);
        let embedding = self.embedding_client.get_embedding(&text_data, "document").await?;

        // store content to
        self.vdb_client.insert_to_collection(collection_name, embedding, path_str, &content).await?;

        Ok(())
    }

    // and this
    async fn query_vdb(&self, collection_name: &str, query: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {

        // vectorize query
        let qvec = self.embedding_client.get_embedding(query, "query").await?;

        // search db
        let search_limit = crate::common::config
            ::get_config("vectordb_config.json", "default_search_limit")?;
        let search_result = self.vdb_client.search_collection(collection_name, qvec, search_limit).await?;

        // build context (json)
        Ok(serde_json::json!({
            "relevant_files_from_rag": search_result.iter().map(|sp| {
                serde_json::json!({
                    "file_name": sp.payload
                        .get("file_name"),
                    "content": sp.payload
                        .get("content"),
                    "score": sp.score,
                })
            }).collect::<Vec<_>>()
        }))
    }
}

#[cfg(test)]
mod tests {
    use directories::ProjectDirs;

    use super::*;

    #[tokio::test]
    async fn test_embed_a_file() {
        let bot = RustBot::new("testbot");
        let coll_name = "512_test_collection";

        let project_dirs = ProjectDirs::from("com", "Justin Inc.", "rustbot").unwrap();
        let path = project_dirs.cache_dir().join("tmp.md");

        let _result = bot._get_vdb_client().add_collection(coll_name).await;
        // assert!(result.is_ok(), "{}", result.unwrap_err());

        let result = bot.embed_file(coll_name, path).await;
        assert!(result.is_ok(), "{}", result.unwrap_err())
    }
}