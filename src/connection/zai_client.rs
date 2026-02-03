use std::any::Any;

use reqwest::{Client, ClientBuilder};

use crate::connection::{AnthropicToolDefinition, ChatClient, Message, MessagesResponse};



#[derive(Debug)]
pub struct ZaiClient {
    client: Client,
    api_key: String,
    url: String,
    api_version: String,
    default_model: String,
    max_tokens: usize,
}

impl ZaiClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let env_path = crate::common::config::PROJECT_DIRS.config_dir().join(".env");
        dotenvy::from_path(env_path).ok();

        let api_key = std::env::var("ZAI_API_KEY")?;
        Ok(Self {
            client: ClientBuilder::default().build()?,
            api_key: api_key,
            url: "".into(),
            api_version: "".into(),
            default_model: "".into(),
            max_tokens: 0,
        })
    }
}

#[async_trait::async_trait]
impl ChatClient for ZaiClient {
    async fn call_model(
        &self, 
        messages: &Vec<Message>, 
        sys_prompt: &str, 
        tools: Option<&Vec<AnthropicToolDefinition>>, 
        model: Option<String>,
        randomness: f64
    )
    -> Result<MessagesResponse,Box<dyn std::error::Error>> {
        let m = match model {
            Some(model) => model,
            None => self.default_model.clone()
        };

        todo!()
    }

    fn get_tpm(&self) -> (usize,usize) {
        todo!()
    }

    fn get_max_tpm(&self) -> (usize,usize) {
        todo!()
    }

    fn as_any(&self) ->  &dyn Any {
        todo!()
    }
}