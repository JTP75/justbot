use std::{fs, path::PathBuf};

use anthropic::types::{ContentBlock, Message, MessageBuilder, Role};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::rustbot::bot::RustBot;

pub const MOTD_FILENAME: &str = "motd.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub topic: String,
    pub messages: Vec<Message>,
}

pub struct SessionManager {
    pub data_dir: PathBuf,
    pub save_dir: PathBuf,
    pub _config_dir: PathBuf,
    pub current: PathBuf,
}

impl SessionManager {
    pub fn new() -> Self {
        let project_dirs = ProjectDirs::from("com", "Justin Inc.", "rustbot").unwrap();
        SessionManager { 
            data_dir: project_dirs.data_dir().join(""),
            save_dir: project_dirs.data_dir().join("sessions/"),
            _config_dir: project_dirs.config_dir().join(""),
            current: PathBuf::new()
        }
    }

    pub fn _handle_command(&mut self, tokens: Vec<&str>, bot: &mut RustBot) -> String {
        let ans = match tokens[0] {
            "exit" | "q" => {
                format!("Exiting. Goodbye!")
            },
            "wexit" | "wq" | "qw" => {
                let path = if tokens.len()>1 {
                    self.save_dir.join(tokens[1..].join("_"))
                } else if !self.current.as_os_str().is_empty() {
                    self.current.clone()
                } else {
                    let topic = self.generate_topic(bot).unwrap_or(format!("unnamed.json"));
                    let filename = self.get_filename_from_topic(topic);
                    self.save_dir.join(filename)
                };
                match self.save_session_as_json(path.clone(), bot) {
                    Ok(_) => format!("Saved session to\n{}.\nGoodbye!", path.clone().as_os_str().display()),
                    Err(e) => format!("Failed to save session to\n{}.: {}\nGoodbye!", path.as_os_str().display(), e),
                }
            },
            "list" => match self.list_sessions() {
                Ok(sessions) => format!("Saves in {}\n\t{}", self.save_dir.display(), sessions[..].join("\n\t")),
                Err(e) => format!("Failed to load saved sessions: {}", e),
            },
            "save" | "w" => {
                let path = if tokens.len()>1 {
                    self.save_dir.join(tokens[1..].join("_"))
                } else if !self.current.as_os_str().is_empty() {
                    self.current.clone()
                } else {
                    let topic = self.generate_topic(bot).unwrap_or(format!("unnamed.json"));
                    let filename = self.get_filename_from_topic(topic);
                    self.save_dir.join(filename)
                };
                match self.save_session_as_json(path.clone(), bot) {
                    Ok(_) => format!("Saved session to\n{}", path.clone().as_os_str().display()),
                    Err(e) => format!("Failed to save session to\n{}: {}", path.as_os_str().display(), e),
                }
            },
            "load" => {
                if tokens.len()>1 {
                    let path = self.save_dir.join(tokens[1..].join("_"));
                    match self.load_session_from_json(path.clone(), bot) {
                        Ok(_) => format!("Loaded session from\n{}", path.clone().as_os_str().display()),
                        Err(e) => format!("Failed to load session from\n{}: {}", path.as_os_str().display(), e),
                    }
                } else {
                    format!("Must specify session file to load")
                }
            },
            _ => format!("Control shouldn't reach here."),
        };
        format!("\x1b[1;35m>>\x1b[0m {}", ans)
    }

    pub fn save_session(&mut self, bot: &RustBot, filename_arg: Option<String>) -> Result<(),Box<dyn std::error::Error>> {
        let path = match filename_arg {
            Some(filename) => self.save_dir.join(filename),
            None => {
                if !self.current.as_os_str().is_empty() {
                    self.current.clone()
                } else {
                    let topic = self.generate_topic(bot).unwrap_or(format!("unnamed.json"));
                    let filename = self.get_filename_from_topic(topic);
                    self.save_dir.join(filename)
                }
            }
        };
        self.save_session_as_json(path.clone(), bot)
    }

    pub fn load_session(&mut self, bot: &mut RustBot, filename: String) -> Result<(),Box<dyn std::error::Error>> {
        let path = self.save_dir.join(filename);
        self.load_session_from_json(path.clone(), bot)
    }

    pub fn list_sessions(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let saved_sessions = fs::read_dir(&self.save_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.file_name().to_str().map(|s| s.to_string())
                })
            })
            .collect::<Vec<String>>();
        Ok(saved_sessions)
    }

    fn get_filename_from_topic(&self, topic: String) -> String {
        let file_stem = topic.trim().replace(" ","_").to_ascii_lowercase();
        format!("{file_stem}.json")
    }

    fn generate_topic(&self, bot: &RustBot) -> Result<String, Box<dyn std::error::Error>> {
        let mut convo_copy = bot.get_messages();
        let topic_prompt = MessageBuilder::default()
            .role(Role::User)
            .content(vec![ContentBlock::Text { 
                text: format!("What is the topic of this conversation, in five words or less? Write your response with as few words as possible. Response with these five or less words only.")
            }])
            .build()?;
        convo_copy.push(topic_prompt);
        
        let anthroclient = bot.get_anthropic_client();
        let topic_response = anthroclient.send_message(&convo_copy, "", 0.0)?;

        match topic_response.content.first() {
            Some(ContentBlock::Text { text }) => Ok(text.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect()),
            Some(ContentBlock::Image { source: _, media_type: _, data: _ }) => Err("Unexpected content block type".into()),
            None => Err("Response content is empty".into())
        }
    }

    pub fn save_motd(&self, bot: &RustBot) -> Result<(),Box<dyn std::error::Error>> {
        if !self.data_dir.exists() { fs::create_dir_all(&self.data_dir)?; }

        let json = serde_json::to_string_pretty(&bot.get_motd())?;
        Ok(fs::write(&self.data_dir.join(MOTD_FILENAME), json)?)
    }

    pub fn load_motd(&self, bot: &mut RustBot) -> Result<(),Box<dyn std::error::Error>> {
        let json = fs::read_to_string(&self.data_dir.join(MOTD_FILENAME))?;
        let motd: (chrono::NaiveDate, Option<String>) = serde_json::from_str(&json)?;
        Ok(bot.set_motd(motd))
    }

    fn save_session_as_json(&mut self, path: PathBuf, bot: &RustBot) -> Result<(),Box<dyn std::error::Error>> {        
        if !self.save_dir.exists() { fs::create_dir_all(&self.save_dir)?; }

        let session = Session { 
            topic: bot.get_topic(),
            messages: bot.get_messages(),
        };

        // set this as the current session file
        self.current = path;

        let json = serde_json::to_string_pretty(&session)?;
        fs::write(&self.current, json)?;
        
        Ok(())
    }

    fn load_session_from_json(&mut self, path: PathBuf, bot: &mut RustBot) -> Result<(),Box<dyn std::error::Error>> {
        let json = fs::read_to_string(&path)?;
        let session: Session = serde_json::from_str(&json)?;

        self.current = path;

        bot.set_topic(session.topic);
        bot.set_messages(session.messages);

        self.load_motd(bot)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_existing() {
        let _sm = SessionManager::new();
    }
}