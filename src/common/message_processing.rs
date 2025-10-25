use std::fs;

use anthropic::types::{Message, Role, ContentBlock};
use tokio;
use chrono;

use crate::common::data_manager::{get_save_dir, load_messages_from_json, save_messages_as_json};

static mut CUMULATIVE_TOKENS: usize = 0;

mod system_prompts {
    pub const BASE_GENERAL: &str = "Your name is rustbot. Your name officially has no meaning in particular. You are being called for general use. Your response should always be written in english.";
}

pub fn print_initial_message() {
    println!("Hi, I'm \x1b[0;33mrustbot\x1b[0m! Type 'help' to see what I can do.\n");
}

pub fn route_command(input: &str, messages: &mut Vec<Message>) -> String {
    
    let tokenized: Vec<&str> =  input.split_whitespace().collect();
    match tokenized[0].to_lowercase().as_str() {
        "hello" => "Hi there!".to_string(),
        "help" => "Available commands: hello, help, general <message>, save [<filename>], load <filename>, list, exit".to_string(),
        "general" => {
            if tokenized.len() < 2 {
                return "Usage: general <message>".to_string();
            }

            match call_anthropic(tokenized, messages, system_prompts::BASE_GENERAL) {
                Ok(response) => response,
                Err(e) => format!("Error communicating with Claude: {}", e)
            }
        },
        "save" => {
            let topic = if tokenized.len() < 2 {
                match generate_topic(messages) {
                    Ok(topic) => topic,
                    Err(e) => format!("Error communicating with Claude: {}", e)
                }
            } else {
                tokenized[1..].join("_")
            };
            let filename = create_file_name(topic);

            match save_messages_as_json(filename.clone(), messages.to_vec()) {
                Ok(_) => format!("Saved conversation as\n{}", get_save_dir().join(filename).display()),
                Err(e) => format!("Failed to save conversation: {e}\n{}", get_save_dir().join(filename).display())
            }
        },
        "load" => {
            if tokenized.len() < 2 {
                return "Usage: load <filename> (use list to see available files)".to_string();
            }
            
            let filename = tokenized[1..].join("");
            let new_messages = load_messages_from_json(filename.clone()).unwrap_or(vec![]);

            if new_messages.is_empty() {
                format!("Failed to load messages from\n{}", get_save_dir().join(filename).display())
            } else {
                *messages = new_messages;
                format!("Successfully to loaded conversation from\n{}", get_save_dir().join(filename).display())
            }
        },
        "list" => {
            let files = match fs::read_dir(get_save_dir()) {
                Ok(iterator) => iterator
                    .filter_map(|entry| entry.ok())
                    .filter(|entry| entry.path().is_file())
                    .map(|entry| entry.path())
                    .collect(),
                Err(_) => vec![],
            };
            let files_s = files
                .iter()
                .map(|entry| entry.display().to_string())
                .collect::<Vec<_>>()
                .join("\n");
            format!("{}", files_s)
        },
        _ => "Unknown command. Type 'help' for a list of commands.".to_string(),
    }
}

fn call_anthropic(tokenized_input: Vec<&str>, messages: &mut Vec<Message>, sys_prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
    
    let user_message = Message {
        role: Role::User,
        content: vec![ContentBlock::Text {
            text: tokenized_input[1..].join(" ")
        }]
    };

    // push user message
    messages.push(user_message);

    // get response (blocking)
    let assistant_message = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_claude_response(messages, sys_prompt))?;

    // push assistant message
    messages.push(assistant_message.clone());

    match &assistant_message.content.first() {
        Some(ContentBlock::Text {text}) => { Ok(text.into()) },
        Some(ContentBlock::Image { source: _, media_type: _, data: _ }) => Err("Unexpected content block type".into()),
        None => Err("Response content is empty".into())
    }
}

fn create_file_name(topic: String) -> String {
    // todo handle duplicates
    
    let title = topic.trim().replace(" ","_").to_ascii_lowercase();
    let date = chrono::Local::now().date_naive().format("%m%d%Y");
    format!("{title}_{date}.json").into()
}

fn generate_topic(messages: &Vec<Message>) -> Result<String, Box<dyn std::error::Error>> {
    
    let mut messages = messages.clone();
    messages.push(Message {
        role: Role::User,
        content: vec![ContentBlock::Text {
            text: "What is the topic of this conversation, in ten words or less? Write your response with as few words as possible. Response with these ten or less words only.".into()
        }]
    });

    let assistant_message = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(get_claude_response(&messages, ""))?;

    match &assistant_message.content.first() {
        Some(ContentBlock::Text {text}) => { Ok(text.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect()) },
        Some(ContentBlock::Image { source: _, media_type: _, data: _ }) => Err("Unexpected content block type".into()),
        None => Err("Response content is empty".into())
    }
}

async fn get_claude_response(messages: &Vec<anthropic::types::Message>, sys_prompt: &str) -> Result<Message, Box<dyn std::error::Error>> {

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

    Ok(Message {
        role: Role::Assistant,
        content: response.content
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_generate_topic_runs() {

        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent().unwrap().parent() {
                let env_path = exe_dir.join(".env");
                dotenvy::from_path(env_path).ok();
            }
        }

        let messages = vec![
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text {
                    text: "Hello, I'm Justin! Whats your name?".into(),
                }],
            },
            Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Text {
                    text: "Hello Justin! My name is rustbot. Nice to meet you! How can I help you today?".into(),
                }],
            },
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text {
                    text: "Nice to meet you. Do you remember my name?".into(),
                }],
            },
            Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Text {
                    text: "Yes, I remember! Your name is Justin. Is there anything you'd like to talk about or any way I can assist you?".into(),
                }],
            },
        ];

        let res = generate_topic(&messages);

        println!("{}", res.is_ok());

        assert!(res.is_ok(), "generate_topic returned error: {:?}", res.err());
        let topic = res.unwrap();
        assert!(!topic.trim().is_empty(), "topic should not be empty");
        eprintln!("Generated topic: {}", topic);
    }

    #[test]
    fn test_create_filename_runs() {

        if let Ok(exe_path) = env::current_exe() {
            if let Some(exe_dir) = exe_path.parent().unwrap().parent() {
                let env_path = exe_dir.join(".env");
                dotenvy::from_path(env_path).ok();
            }
        }

        let messages = vec![
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text {
                    text: "Hello, I'm Justin! Whats your name?".into(),
                }],
            },
            Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Text {
                    text: "Hello Justin! My name is rustbot. Nice to meet you! How can I help you today?".into(),
                }],
            },
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text {
                    text: "Nice to meet you. Do you remember my name?".into(),
                }],
            },
            Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Text {
                    text: "Yes, I remember! Your name is Justin. Is there anything you'd like to talk about or any way I can assist you?".into(),
                }],
            },
        ];

        let topic = generate_topic(&messages).unwrap();
        let fname = create_file_name(topic);

        println!("{}", fname);
        
    }
}
