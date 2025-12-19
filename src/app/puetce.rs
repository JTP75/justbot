use std::{env, fs};
use std::path::{Path, PathBuf};

use chrono::{self, Local};

use crate::app::http::HttpClient;
use crate::app::session::SessionManager;
use crate::commands::{self, Command};
use crate::common::config;
use crate::common::config_const::json::{ANTHROPIC_CONFIG, BOT_CONFIG};
use crate::common::config_const::keys::{DEFAULT_MODEL, MOTD_FILENAME};
use crate::connection::anthropic_client::{AnthropicToolDefinition, ContentBlock, Message, MessagesResponse, Role, Source, ToolResultContentBlock};

#[derive(Debug)]
pub struct PuetceApp {

    // immut fields
    pub name: String,
    pub http_client: HttpClient,

    // state
    topic: String,
    model: String,
    messages: Vec<Message>,
    cwd: PathBuf,
    collection: Option<String>
}

impl PuetceApp {

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
            http_client: HttpClient::new(),
            
            topic: "".into(),
            model: crate::common::config
                ::get_config(ANTHROPIC_CONFIG, DEFAULT_MODEL)
                .expect("failed to retrieve from config"),
            messages: vec![],
            cwd: env::current_dir().unwrap_or(PathBuf::new()),
            collection: None
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
    pub fn get_messages(&self) -> Vec<Message> { self.messages.clone() }

    /// Retrieve current motd from file
    /// 
    /// - Returns None if motd is empty
    /// 
    /// # Examples
    /// 
    /// ```
    /// let motd = bot.get_motd();
    /// ```
    pub fn get_motd(&self) -> (chrono::NaiveDate, Option<String>) {
        let filename: String = crate::common::config
            ::get_config(BOT_CONFIG, MOTD_FILENAME)
            .expect("failed to get config");
        let value = fs::read_to_string(config::PROJECT_DIRS.data_dir().join(filename));
        match value {
            Ok(value) => serde_json::from_str(&value)
                .unwrap_or((Local::now().date_naive(), None)),
            Err(_) => (Local::now().date_naive(), None)
        }
    }

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
    pub fn get_existing_collections(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let collections = tokio::runtime::Runtime::new()?
            .block_on(self.http_client.list_collections())?;
        Ok(collections)
    }

    /// todo
    pub fn get_current_tools(&self) -> Vec<AnthropicToolDefinition> { 
        self.http_client.list_tools().expect("Failed to retrieve tools")
    }

    /// todo
    pub fn get_model(&self) -> String {
        self.model.clone()
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
            .block_on(self.http_client.add_collection(collection_name))?;
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
    pub fn store_file(&self, collection_name: &str, path: &Path) -> Result<(),Box<dyn std::error::Error>> {
        tokio::runtime::Runtime::new()?
            .block_on(self.http_client.embed_files(collection_name, vec![path]))?;
        Ok(())
    }

    /// Store multiple files to locally hosted vector database
    /// 
    /// - collection_name must match the name of a valid collection
    /// - paths is a slice of Path references
    /// 
    /// # Examples
    /// 
    /// ```
    /// let paths = vec![Path::new("file1.txt"), Path::new("file2.txt")];
    /// bot.store_files("512_test_collection", &paths);
    /// ```
    pub fn store_files(&self, collection_name: &str, paths: Vec<&Path>) -> Result<(),Box<dyn std::error::Error>> {
        tokio::runtime::Runtime::new()?
            .block_on(self.http_client.embed_files(collection_name, paths))?;
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
            .block_on(self.http_client.send_message(messages, sys_prompt, false, Some(self.model.clone()), randomness))
    }

    /// Sends a list of messages to claude and awaits a response (blocking)
    /// All tools registered in the McpManager are included in the request
    /// 
    /// # Examples
    /// 
    /// ```
    /// let ans = bot.query_llm_with_tools(conversation, "You are a chatbot. Be nice!", 0.33)
    /// assert!(ans.is_ok());
    /// 
    /// println!("{:?}", ans.unwrap());
    /// // MessagesResponse(... content="Hello how are you?")
    /// ```
    pub fn query_llm_with_tools(&mut self, messages: &Vec<Message>, sys_prompt: &str, randomness: f64) 
    -> Result<MessagesResponse,Box<dyn std::error::Error>> {
        let rt = tokio::runtime::Runtime::new()?;
        let mut messages_copy = messages.clone();

        let consec_tool_limit = 25usize;

        if let Some(collection) = &self.collection {
            self.http_client.set_collection(collection)?; // THIS IS A BAD WORKAROUND
        }

        // only allow ten consecutive tool calls
        let max_tpm = self.http_client.get_max_tpm()?;
        for _ in 0..consec_tool_limit {
            // get ai response
            let response = rt.block_on(self.http_client
                .send_message(&messages_copy, sys_prompt, true, Some(self.model.clone()), randomness))?;

            let tpm = self.http_client.get_tpm()?;
            if tpm.0 > (17 * max_tpm.0 / 20) { log::warn!("Input token rate limit:  {:5} / {:5}", tpm.0, max_tpm.0) }
            if tpm.1 > (17 * max_tpm.1 / 20) { log::warn!("Output token rate limit: {:5} / {:5}", tpm.1, max_tpm.1) }

            let tool_uses: Vec<_> = response.content.iter()
                .filter_map(|cb| match cb {
                    ContentBlock::ToolUse { id, name, input } => Some((
                        id.clone(),
                        name.clone(),
                        input.clone(),
                    )), _ => None,
                }).collect();

            if tool_uses.is_empty() { return Ok(response); }
            
            messages_copy.push(response.into());
            let input_token_est = estimate_token_count_messages(&messages_copy);

            // execute tools

            log::info!("Executing {} tools...", tool_uses.len());

            let mut content: Vec<ContentBlock> = vec![];
            for (id, name, input) in tool_uses.iter() {
                log::info!("Executing tool '{name}': {input}");

                // todo we probably dont want a print statement here
                println!("\r\x1b[1;34m>>\x1b[0m Executing tool: '{name}'");

                let tool_result_block = match self.http_client.execute_tool(name, input) {
                    Ok(result_blocks) => {
                        log::info!("Tool '{}' executed successfully.", name);
                        let new_input_length: usize = result_blocks.iter()
                            .map(|block| match block {
                                ToolResultContentBlock::Text { text } => text.len(),
                                ToolResultContentBlock::Document { source, title: _, context: _context } =>
                                    match source {
                                        Source::Text { media_type: _, data } => data.len(),
                                    },
                                ToolResultContentBlock::Image { source: _, media_type: _, data: _ } => 0,
                            }).sum();

                        ContentBlock::ToolResult { 
                            tool_use_id: id.into(), 
                            content: if result_blocks.is_empty() {
                                // check if result blocks is empty
                                vec![ToolResultContentBlock::Text { 
                                    text: "Tool returned successfully with no content".into() 
                                }]
                            } else if input_token_est + 0 > 190_000 {
                                // check if result blocks will push over the context window limit (200000 tokens)
                                log::warn!("Approaching context window danger zone: {} / 200000", input_token_est);
                                vec![ToolResultContentBlock::Text { 
                                    text: format!("Tool '{}' {} (~{} tokens). The current context window is ~{} {}", 
                                        name, "execution succeeded, but returned too many tokens", new_input_length/4, 
                                        input_token_est, "tokens and the upper limit is 190000 tokens.",)
                                }]
                            } else {
                                result_blocks
                            }, 
                            is_error: false
                        }
                    },
                    Err(e) => {
                        log::error!("Tool '{}' execution failed:\n{}", name, e);
                        ContentBlock::ToolResult {
                            tool_use_id: id.into(), 
                            content: vec![ToolResultContentBlock::Text { 
                                text: format!("Tool exited with error: {}", e) 
                            }], 
                            is_error: true
                        }
                    },
                };

                content.push(tool_result_block);
            }

            let tool_result_msg = Message {
                role: Role::User,
                content,
            };

            messages_copy.push(tool_result_msg);
        }

        log::error!("Too many chained tool execution calls. The limit is {consec_tool_limit} requests per turn");
        Err(format!("Too many chained tool execution calls. The limit is {consec_tool_limit} requests per turn").into())
    }

    /// todo
    pub fn retrieve_from_vdb(&self, collection_name: &str, query: &str, search_limit: Option<u64>)
    -> Result<String, Box<dyn std::error::Error>> {
        let json_results = tokio::runtime::Runtime::new()?
            .block_on(self.http_client.query_vdb(collection_name, query, search_limit))?;
        Ok(serde_json::to_string_pretty(&json_results)?)
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
    /// let user_message = bot.generate_rag_message(
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
    #[deprecated]
    pub fn generate_rag_message(&self, collection_name: &str, query: &str)
    -> Result<Message, Box<dyn std::error::Error>> {
        let runtime = tokio::runtime::Runtime::new()?;
        let json_context = runtime.block_on(self.http_client.query_vdb(collection_name, query, None))?;
        let context = serde_json::to_string_pretty(&json_context)?;

        Ok(Message {
            role: Role::User,
            content: vec![
                ContentBlock::Text { text: query.into() },
                ContentBlock::Text { text: context }
            ]
        })
    }
    
    /// Sets the topic field
    pub fn set_topic(&mut self, topic: impl Into<String>) -> () { self.topic = topic.into() }   
    
    /// Overwrite the current conversation with a new conversation
    pub fn set_messages(&mut self, messages: Vec<Message>) -> () { self.messages = messages }

    /// Set the cwd
    pub fn set_cwd(&mut self, new_cwd: impl Into<PathBuf>) -> () { self.cwd = new_cwd.into() }

    /// Set the currently selected VectorDB collection
    pub fn set_current_collection(&mut self, collection: Option<String>) -> () { self.collection = collection }

    /// todo
    pub fn set_model(&mut self, model: impl Into<String>) -> () {
        self.model = model.into()
    }

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

    /// todo move this somewhere else
    /// 
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

        let command = commands::REGISTRY.lock().unwrap()
            .get(&command_name)
            .ok_or_else(|| format!("Unknown command: {command_name}"))?;

        let args = tokenized[1..].iter().map(|s| s.to_string()).collect::<Vec<_>>();

        Ok(( command, args ))
    }
}

fn estimate_token_count_messages(messages: &Vec<Message>) -> usize {
    messages.iter()
        .map(|m| m.content.iter()
            .map(|c| match c {
                ContentBlock::Text { text } => text.len()/4,
                _ => 0,
            }).sum::<usize>()
        ).sum()
}

#[cfg(test)]
mod tests {
}