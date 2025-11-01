use std::{env, fs};
use std::path::{Path, PathBuf};

use anthropic::types::{ContentBlock, Message, MessageBuilder, MessagesResponse, Role};
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

    /// Return a copy of the currently selected collection for storage/retrieval
    /// 
    /// # Examples
    /// 
    /// ```
    /// assert!(bot.get_collection().is_some());
    /// ```
    /// 
    /// If no collection is selected
    /// ```
    /// let bot = RustBot::new();
    /// assert!(bot.get_collection().is_none());
    /// ```
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

    /// (Synchronous callback for `ClientManager::call_model_callback()`)
    /// 
    /// Sends a list of messages to claude and awaits a response (blocking)
    /// 
    /// # Examples
    /// 
    /// ```
    /// let ans = bot.query_llm(conversation, "You are a chatbot. Be nice!", 0.33)
    /// assert!(ans.is_ok());
    /// 
    /// println!("{:?}", ans.unwrap());
    /// // MessagesResponse(... content="Hello how are you?")
    /// ```
    pub fn query_llm(&self, messages: &Vec<Message>, sys_prompt: &str, randomness: f64) 
    -> Result<MessagesResponse,Box<dyn std::error::Error>> {
        tokio::runtime::Runtime::new()?
            .block_on(self.client_mgr.call_model_callback(messages, sys_prompt, randomness))
    }

    /// Generate an anthropic message with rag
    /// 
    /// The resulting Message struct has two content blocks:
    /// - `content[0]` contains the user's query
    /// - `content[1]` is a json-formatted list of relevant documents
    /// 
    /// # Examples
    /// 
    /// ```
    /// let user_message = bot.query_with_rag(
    ///     "programming_project_docs", 
    ///     "tell me about the programming project"
    /// )?;
    /// 
    /// // add user message to convo
    /// bot.push_message(user_message);
    /// 
    /// // call llm separately
    /// let ans = bot.query_llm(bot.get_messages(), "You are retrieving documents, say some technical stuff")
    /// // ...
    /// ```
    pub fn generate_rag_query(&self, collection_name: &str, query: &str)
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
    
    /// Sets the topic field
    pub fn set_topic(&mut self, topic: impl Into<String>) -> () { self.topic = topic.into() }   
    
    /// Overwrite the current conversation with a new conversation
    pub fn set_messages(&mut self, messages: Vec<Message>) -> () { self.messages = messages }    
    
    /// Set the message of the day field
    /// 
    /// - expects a NaiveData and Option<String> tuple
    pub fn set_motd(&mut self, motd: (chrono::NaiveDate, Option<String>)) -> () { self.motd = motd }

    /// Set the currently selected VectorDB collection
    pub fn set_current_collection(&mut self, collection: Option<String>) -> () { self.collection = collection }

    /// Push a `anthropic::types::Message` to the end of the messages Vec
    pub fn push_message(&mut self, message: Message) -> () { self.messages.push(message) }
    
    /// Handle any valid input command end-to-end
    /// 
    /// - pipelining process occurs in this function
    /// 
    /// # Examples
    /// 
    /// ## Simple
    /// ```
    /// let mut bot = RustBot::new("pluh");
    /// let mut sm = SessionManager::new();
    /// 
    /// let response = bot.handle_command(&mut sm, "hello").unwrap();
    /// 
    /// assert_eq!(response, Some(String::from("Hello there! My name is pluh.")));
    /// ```
    /// 
    /// ## With Pipelining
    /// ```
    /// let mut bot = RustBot::new("pluh");
    /// let mut sm = SessionManager::new();
    /// 
    /// let response = bot.handle_command(&mut sm, "hello > ./response.txt").unwrap();
    /// // result will be printed to stdout and saved to response.txt
    /// 
    /// assert_eq!(response, Some(String::from("Hello there! My name is pluh.")));
    /// ```
    pub fn handle_command(&mut self, sm: &mut SessionManager, input: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let (command,args) = self.parse_command(input)?;
        let output = command.exec(sm, self, &args);
        if let Ok(Some(response)) = &output {
            if let Some(i) = args.iter().enumerate()
                .find_map(|(i,arg)| if *arg==">" {Some(i)} else {None}) 
            {
                log::info!("Pipelining output...");

                // check if filename argument exists
                let filename = args.get(i + 1)
                    .ok_or("Missing filename after '>'")?;

                match self.resolve_file_path_str(&filename) {
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

    /// Validates a file path string for writing, then returns fully resolved absolute path
    /// 
    /// - The file does not need to exist, but its parent directories must exist
    /// 
    /// Checks:
    /// - `path_str` is not empty
    /// - `path_str` contains only valid chars
    ///     - `invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\0']`
    /// - `path_str` parent directory exists in the file system
    ///     - passes if parent directory is root
    ///     - passes if no parent directory is specified (relative path)
    /// - `path_str` is a file, not a directory
    /// 
    /// # Examples
    /// ```
    /// let bot = RustBot::new("notrustbot");
    /// 
    /// // good absolute path
    /// let rslt = bot.resolve_file_path("/home/pacel/something.txt");
    /// assert!(rslt.is_ok());
    /// 
    /// // good relative path
    /// let rslt = bot.resolve_file_path("something.txt");
    /// assert!(rslt.is_ok());
    /// 
    /// // bad file name
    /// let rslt = bot.resolve_file_path("/home/pacel/something_bad<<<.txt");
    /// assert!(rslt.is_err());
    /// 
    /// // parent dir doesnt exist
    /// let rslt = bot.resolve_file_path("/home/pacel/this_dir_doesnt_exist/something.txt");
    /// assert!(rslt.is_err());
    /// 
    /// // not a file
    /// let rslt = bot.resolve_file_path("/home/pacel/this_is_dir_not_a_file");
    /// assert!(rslt.is_err());
    /// ```
    pub fn resolve_file_path_str(&self, path_str: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        if path_str.trim().is_empty() {
            return Err("path_str cannot be empty".into());
        }

        let invalid_chars = ['<', '>', ':', '"', '|', '?', '*', '\0'];
        if path_str.chars().any(|c| invalid_chars.contains(&c)) {
            return Err(format!("path_str contains invalid characters: {}", path_str).into());
        }

        let path = Path::new(path_str);
        
        let canonical = 
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {

                let canonical_parent = parent.canonicalize()
                    .map_err(|_| format!("Directory does not exist: {}", parent.display()))?;
                
                if let Some(file_name) = path.file_name() {
                    canonical_parent.join(file_name)
                } else {
                    return Err("path_str is not a valid file".into());
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

    fn parse_command(&self, input: &str) -> Result<(Box<dyn Command>, Vec<String>), Box<dyn std::error::Error>> {
        let tokenized: Vec<&str> =  input.split_whitespace().collect();
        let command_name = tokenized.first().ok_or("User input empty")?;

        let command = REGISTRY.lock().unwrap()
            .get(&command_name)
            .ok_or_else(|| format!("Unknown command: {command_name}"))?;

        let args = tokenized[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>();

        Ok(( command, args ))
    }

    fn _get_chat_client(&self) -> &AnthropicClient { &self.client_mgr.chat_client }
    fn _get_vdb_client(&self) -> &QdrantClient { &self.client_mgr.vdb_client }
    fn _get_mbed_client(&self) -> &VoyageClient { &self.client_mgr.embedding_client }
}

impl ClientManager {

    // callbacks

    /// callback for `AnthropicClient::call_model`
    pub async fn call_model_callback(&self, messages: &Vec<Message>, sys_prompt: &str, randomness: f64) 
    -> Result<MessagesResponse,Box<dyn std::error::Error>> {
        self.chat_client.call_model(messages, sys_prompt, randomness).await
    }

    // routines
    
    /// Gets the embedding for a file and stores to the Vector DB
    /// 
    /// - assumes `file_path` is valid
    /// - attempts to convert pdfs to text
    pub async fn embed_file(&self, collection_name: &str, file_path: PathBuf) 
    -> Result<(), Box<dyn std::error::Error>> {
        let path_str = match file_path.to_str() {
            Some(s) => s,
            None => { return Err(format!("Error converting path <{}> to &str", file_path.display()).into()) }
        };
        let content = match file_path.extension().unwrap().to_str() {
            Some("pdf") => crate::common::pdf
                ::extract_pdf_text(&file_path)?,
            _ => fs::read_to_string(&file_path)
                .map_err(|_| format!("Failed to read file {}", file_path.display()))?
        };

        // embed content and path
        //      fixme theres a better way to group embeddings...
        let text_data = format!("{{\"file_path\": \"{}\", \"content\": \"{}\"}}", path_str, content);
        let embedding = self.embedding_client.get_embedding(&text_data, "document").await?;

        // store content to vdb
        self.vdb_client.insert_to_collection(collection_name, embedding, path_str, &content).await?;

        Ok(())
    }
    
    /// Searches the Vector DB with the user's query
    /// 
    /// - returns json result containing a list of the top `n` relevant files
    ///     - where `n` is the search_limit set in vectordb_config.json
    /// - each entry contains the file path, file contents, and search score value
    pub async fn query_vdb(&self, collection_name: &str, query: &str) 
    -> Result<serde_json::Value, Box<dyn std::error::Error>> {

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