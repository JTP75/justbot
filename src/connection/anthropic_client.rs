use anthropic::{client::{Client, ClientBuilder}, types::{Message, MessagesRequest, MessagesRequestBuilder, MessagesResponse}};
use tokio;
use dotenvy;

#[derive(Debug)]
pub struct AnthropicClient {
    client: Client,
    model: String,
    max_tokens: usize,
}

/**
 * temperature adjusts randomness, defaults to 1.0
 * stream allows to stream response
 */

impl AnthropicClient {

    // public

    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let env_path = exe_dir.join(".env");
                dotenvy::from_path(env_path).ok();
            }
        }

        let api_key = std::env::var("API_KEY")?;

        Ok(Self {
            client: ClientBuilder::default()
                .api_key(api_key)
                .build()?,
            model: "claude-sonnet-4-5-20250929".into(),
            max_tokens: 4096,
        })
    }

    pub fn send_message(&self, messages: &Vec<Message>, sys_prompt: &str, randomness: f64) -> Result<MessagesResponse,Box<dyn std::error::Error>> {
        let request = MessagesRequestBuilder::default()
            .model(&self.model)
            .max_tokens(self.max_tokens)
            .temperature(randomness)
            .messages(&messages[..])
            .system(sys_prompt)
            .build()?;
        let response = self.call_api(request)?;
        Ok(response)
    }

    // private

    fn call_api(&self, request: MessagesRequest) -> Result<MessagesResponse, Box<dyn std::error::Error>> {
        let response = tokio::runtime::Runtime::new()?
            .block_on(self.call_api_future(request))?;
        Ok(response)
    }

    async fn call_api_future(&self, request: MessagesRequest) -> Result<MessagesResponse, Box<dyn std::error::Error>> {
        let response = self.client.messages(request).await?;
        Ok(response)
    }
}