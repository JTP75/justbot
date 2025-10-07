use anthropic;
use tokio;
use dotenv::dotenv;

pub fn print_initial_message() {
    println!("Rustbot is starting up!");
}

pub fn get_response_for_input(input: &str) -> String {
    let tokenized: Vec<&str> =  input.split_whitespace().collect();

    match tokenized[0].to_lowercase().as_str() {
        "hello" => "Hi there!".to_string(),
        "help" => "Available commands: hello, help, claude <message>, exit".to_string(),
        "claude" => {
            if tokenized.len() < 2 {
                return "Usage: claude <message>".to_string();
            }
            let message = tokenized[1..].join(" ");
            
            let result = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(get_claude_response(&message));

            match result {
                Ok(response) => response,
                Err(e) => format!("Error communicating with Claude: {}", e)
            }
        }
        _ => "Unknown command. Type 'help' for a list of commands.".to_string(),
    }
}

async fn get_claude_response(message: &str) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set in .env file");

    let client = anthropic::client::ClientBuilder::default()
        .api_key(api_key)
        .build()?;

    let request = anthropic::types::MessagesRequestBuilder::default()
        .model("claude-sonnet-4-5-20250929".to_string())
        .max_tokens(1024usize)
        .messages(vec![
            anthropic::types::Message {
                role: anthropic::types::Role::User,
                content: vec![anthropic::types::ContentBlock::Text {
                    text: message.to_string(),
                }],
            }
        ])
        .build()?;

    let response = client.messages(request).await?;

    match &response.content[0] {
        anthropic::types::ContentBlock::Text { text } => Ok(text.clone()),
        _ => Err("Unexpected content block type".into()),
    }
}
