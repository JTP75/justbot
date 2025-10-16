use anthropic;
use tokio;

static mut CUMULATIVE_TOKENS: usize = 0;

mod system_prompts {
    pub const BASE_GENERAL: &str = "Your name is rustbot. Your name officially has no meaning in particular. You are being called for general use; you have no particular purpose/tasks";
}

pub fn print_initial_message() {
    println!("Hi, I'm rustbot! Type 'help' to see what I can do.\n");
}

pub fn get_response_for_input(input: &str, messages: &mut Vec<anthropic::types::Message>) -> String {
    let tokenized: Vec<&str> =  input.split_whitespace().collect();

    match tokenized[0].to_lowercase().as_str() {
        "hello" => "Hi there!".to_string(),
        "help" => "Available commands: hello, help, claude <message>, exit".to_string(),
        "claude" => {
            if tokenized.len() < 2 {
                return "Usage: claude <message>".to_string();
            }

            match call_anthropic(tokenized, messages, system_prompts::BASE_GENERAL) {
                Ok(response) => response,
                Err(e) => {
                    format!("Error communicating with Claude: {}", e)
                }
            }
        }
        _ => "Unknown command. Type 'help' for a list of commands.".to_string(),
    }
}

fn call_anthropic(tokenized_input: Vec<&str>, messages: &mut Vec<anthropic::types::Message>, sys_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    messages.push(anthropic::types::Message {
        role: anthropic::types::Role::User,
        content: vec![anthropic::types::ContentBlock::Text {
            text: tokenized_input[1..].join(" ")
        }]
    });
    tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_claude_response(messages, sys_prompt))
}

async fn get_claude_response(messages: &mut Vec<anthropic::types::Message>, sys_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    let api_key = std::env::var("API_KEY").expect("API_KEY must be set in .env file");

    let client = anthropic::client::ClientBuilder::default()
        .api_key(api_key)
        .build()?;
    
    let request = anthropic::types::MessagesRequestBuilder::default()
        .model("claude-sonnet-4-5-20250929".to_string())
        .max_tokens(1024usize)
        .messages(&messages[..])
        .system(sys_prompt)
        .build()?;

    let response = client.messages(request).await?;

    let input_tokens = response.usage.input_tokens;
    let output_tokens = response.usage.output_tokens;
    let total_tokens = input_tokens + output_tokens;

    println!("Input tokens =  {}\nOutput tokens = {}\nTotal tokens =  {}", input_tokens, output_tokens, total_tokens);

#[allow(static_mut_refs)]
    unsafe {
        CUMULATIVE_TOKENS += total_tokens;
        println!("Cumulative = {}", CUMULATIVE_TOKENS);
    }

    match &response.content[0] {
        anthropic::types::ContentBlock::Text { text } => {
            messages.push(
                anthropic::types::Message {
                    role: anthropic::types::Role::Assistant,
                    content: vec![anthropic::types::ContentBlock::Text {
                        text: text.clone()
                    }]
                }
            );
            Ok(text.clone())
        },
        _ => Err("Unexpected content block type".into()),
    }
}

// maybe go with this: file-based gen ai: analyze a directory, generate README perhaps, generate other files changelog etc whatwever