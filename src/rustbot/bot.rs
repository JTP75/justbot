use std::{env, fs};
use std::path::{Path, PathBuf};

use anthropic::types::{ContentBlock, Message, MessageBuilder, Role};
use chrono::{self, Local};

use crate::commands::{Command, REGISTRY};
use crate::connection::anthropic_client::AnthropicClient;
use crate::connection::qdrant_client::QdrantClient;
use crate::connection::voyage_client::VoyageClient;
use crate::rustbot::session::SessionManager;

// structs

#[derive(Debug)]
pub struct RustBot {

    // immut fields
    name: String,

    client_mgr: ClientManager,

    // state
    topic: String,
    messages: Vec<Message>,
    motd: (chrono::NaiveDate, Option<String>),
    _date: chrono::NaiveDate,
    cwd: PathBuf,
    collection: Option<String>,

    _input_tokens: Vec<usize>,
    _output_tokens: Vec<usize>,
    _total_tokens: Vec<usize>,
}

#[derive(Debug)]
pub struct ClientManager {
    pub chat_client: AnthropicClient,
    pub vdb_client: QdrantClient,
    pub embedding_client: VoyageClient,
}

// impls

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

            client_mgr: ClientManager {
                chat_client: AnthropicClient::new().unwrap(),
                vdb_client: QdrantClient::new().unwrap(),
                embedding_client: VoyageClient::new().unwrap(),
            },
            
            topic: "".into(),
            messages: vec![],
            motd: (Local::now().date_naive(), None),
            _date: Local::now().date_naive(),
            cwd: env::current_dir().unwrap_or(PathBuf::new()),
            collection: None,

            _input_tokens: vec![],
            _output_tokens: vec![],
            _total_tokens: vec![],
        }
    }

    /// Get the name of this instance
    /// 
    /// # Examples
    /// 
    /// ```
    /// let bot_name = bot.get_name();
    /// assert_eq!(bot_name,"rustbot");
    /// ```
    pub fn get_name(&self) -> String { self.name.clone() }

    /// Get the current working directory of this rust program
    /// 
    /// # Examples
    /// 
    /// ```
    /// let cwd: PathBuf = bot.get_cwd();
    /// ```
    pub fn get_cwd(&self) -> PathBuf { self.cwd.clone() }

    /// Get the topic of the current conversation
    /// 
    /// # Examples
    /// 
    /// ```
    /// // e.g. messaging about cheese
    /// let topic = bot.get_topic();
    /// assert_eq!(topic,"cheese");
    /// ```
    pub fn get_topic(&self) -> String { self.topic.clone() }

    /// Get a copy of the current conversation's message log
    /// 
    /// - Returns a Vec of anthropic messages
    /// - Does not generate a topic if topic is empty
    /// 
    /// # Examples
    /// 
    /// ```
    /// let messages = bot.get_messages();
    /// ```
    pub fn get_messages(&self) -> Vec<anthropic::types::Message> { self.messages.clone() }

    /// Get a copy of the current motd
    /// 
    /// - Returns None if motd is empty
    /// 
    /// # Examples
    /// 
    /// ```
    /// let motd = bot.get_motd();
    /// ```
    pub fn get_motd(&self) -> (chrono::NaiveDate, Option<String>) { self.motd.clone() }

    /// Get a reference to the LLM chat client (anthropic in this case)
    /// 
    /// # Examples
    /// 
    /// ```
    /// let anthropic_client = bot.get_chat_client();
    /// ```
    pub fn get_chat_client(&self) -> &AnthropicClient { &self.client_mgr.chat_client }

    /// Get a reference to the vector database client (Qdrant in this case)
    /// 
    /// # Examples
    /// 
    /// ```
    /// let vdb_client = bot._get_vdb_client();
    /// ```
    pub fn _get_vdb_client(&self) -> &QdrantClient { &self.client_mgr.vdb_client }

    /// todo write desc
    pub fn get_current_collection(&self) -> Option<String> { self.collection.clone() }

    /// Get a list of all collections
    /// 
    /// # Examples
    /// 
    /// ```
    /// // returns result of a vec of strings
    /// let collections = bot.get_existing_collections().unwrap();
    /// ```
    #[allow(unused)]
    pub fn get_existing_collections(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let collections = tokio::runtime::Runtime::new()?
            .block_on(self.client_mgr.vdb_client.list_collections())?;
        Ok(collections)
    }

    /// Add a named collection to the vector database
    /// 
    /// - will error if collection already exists, but this can be ignored
    /// 
    /// ```
    /// bot.add_collection("my_new_collection").unwrap();
    /// ```
    pub fn add_collection(&self, collection_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let result = tokio::runtime::Runtime::new()?
            .block_on(self.client_mgr.vdb_client.add_collection(collection_name))?;
        Ok(result)
    }

    /// Store a file to locally hosted vector database
    /// 
    /// - collection_name must match the name of a valid collection
    /// - path must resolve to the location of a file
    /// 
    /// # Examples
    /// 
    /// ```
    /// bot.store_file("512_test_collection", "path/to/some/file.md")
    /// ```
    pub fn store_file(&self, collection_name: &str, path: PathBuf) -> Result<(),Box<dyn std::error::Error>> {
        tokio::runtime::Runtime::new()?
            .block_on(self.client_mgr.embed_file(collection_name, path))?;
        Ok(())
    }

    /// todo write desc
    pub fn query_with_rag(&self, collection_name: &str, query: &str)
    -> Result<Message, Box<dyn std::error::Error>> {

        let runtime = tokio::runtime::Runtime::new()?;
        let json_context = runtime.block_on(self.client_mgr.query_vdb(collection_name, query))?;
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
    
    /// todo write desc
    pub fn set_topic(&mut self, topic: impl Into<String>) -> () { self.topic = topic.into() }   
    
    /// todo write desc 
    pub fn set_messages(&mut self, messages: Vec<Message>) -> () { self.messages = messages }    
    
    /// todo write desc
    pub fn set_motd(&mut self, motd: (chrono::NaiveDate, Option<String>)) -> () { self.motd = motd }

    /// todo write desc
    pub fn set_current_collection(&mut self, collection: Option<String>) -> () { self.collection = collection }

    /// todo write desc
    pub fn push_message(&mut self, message: Message) -> () { self.messages.push(message) }
    
    /// todo write desc
    pub fn handle_command(&mut self, sm: &mut SessionManager, input: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let (command,args) = self.parse_command(input)?;
        let output = command.exec(sm, self, &args);
        if let Ok(Some(response)) = &output {
            if let Some(i) = args.iter().enumerate()
                .find_map(|(i,arg)| if *arg==">" {Some(i)} else {None}) 
            {
                log::info!("Pipelining output...");

                // Check if filename argument exists
                let filename = args.get(i + 1)
                    .ok_or("Missing filename after '>'")?;

                match self.validate_filename(&filename) {
                    Ok(canonical_path) => {
                        fs::write(&canonical_path, response)?;
                        log::info!("Pipeline output success!")
                    },
                    Err(e) => 
                        log::warn!("Pipeline to file failed: {e}")
                }
            }
        }
        output
    }

    /// todo write desc
    pub fn validate_filename(&self, filename: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        if filename.trim().is_empty() {
            return Err("Filename cannot be empty".into());
        }

        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\0'];
        if filename.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(format!("Filename contains invalid characters: {}", filename).into());
        }

        let path = Path::new(filename);
        
        let canonical = if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {

                let canonical_parent = parent.canonicalize()
                    .map_err(|_| format!("Directory does not exist: {}", parent.display()))?;
                
                if let Some(file_name) = path.file_name() {
                    canonical_parent.join(file_name)
                } else {
                    return Err("Invalid filename".into());
                }
            } else {
                std::env::current_dir()?.join(path)
            }
        } else {
            std::env::current_dir()?.join(path)
        };

        Ok(canonical)
    }

    // private

    /// todo write desc
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

impl ClientManager {
    
    /// todo write desc
    pub async fn embed_file(&self, collection_name: &str, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let path_str = match path.to_str() {
            Some(s) => s,
            None => { return Err(format!("Error converting path <{}> to &str", path.display()).into()) }
        };
        let content = match path.extension().unwrap().to_str() {
            Some("pdf") => crate::common::pdf
                ::extract_pdf_text(&path)?,
            _ => fs::read_to_string(&path)
                .map_err(|_| format!("File at {} is not valid UTF-8", path.display()))?
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
    
    
    /// todo write desc
    pub async fn query_vdb(&self, collection_name: &str, query: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {

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
        let coll_name = "test_collection_botrs";

        let project_dirs = ProjectDirs::from("com", "Justin Inc.", "rustbot").unwrap();
#[allow(unused)]
        let path = project_dirs.cache_dir().join("tmp.md");

        let path = PathBuf::from("/mnt/c/Users/pacel/northeastern/fall_2025/TELE6510/readings/ieee_the_institute_iot.pdf");
        // let path = PathBuf::from("/mnt/c/Users/pacel/northeastern/fall_2025/TELE6510/homework/hw1.pdf");

        let _result = bot._get_vdb_client().add_collection(coll_name).await;
        // assert!(result.is_ok(), "{}", result.unwrap_err());

        let result = bot.client_mgr.embed_file(coll_name, path).await;
        assert!(result.is_ok(), "{}", result.unwrap_err())
    }
}