use std::{fs, path::PathBuf};

use anthropic::types::Message;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::rustbot::bot::RustBot;

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub topic: String,
    pub messages: Vec<Message>,
}

pub struct SessionManager {
    pub save_dir: PathBuf,
    pub current: PathBuf,
}

impl SessionManager {
    pub fn new() -> Self {
        SessionManager { 
            save_dir: ProjectDirs::from("com", "Justin Inc.", "rustbot").unwrap()
                .data_dir()
                .join(""),
            current: PathBuf::new()
        }
    }

    pub fn handle_command(&mut self, tokens: Vec<&str>, bot: &mut RustBot) -> String {
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
                    // TODO call agent to generate save name
                    self.save_dir.join("unnamed.json")
                };
                match self.save_session_as_json(path.clone(), bot) {
                    Ok(_) => format!("Saved session to\n{}.\nGoodbye!", path.clone().as_os_str().display()),
                    Err(e) => format!("Failed to save session to\n{}.: {}\nGoodbye!", path.as_os_str().display(), e),
                }
            },
            "list" => match self.list_saved_sessions() {
                Ok(sessions) => format!("Saves in {}\n\t{}", self.save_dir.display(), sessions[..].join("\n\t")),
                Err(e) => format!("Failed to load saved sessions: {}", e),
            },
            "save" | "w" => {
                let path = if tokens.len()>1 {
                    self.save_dir.join(tokens[1..].join("_"))
                } else if !self.current.as_os_str().is_empty() {
                    self.current.clone()
                } else {
                    // TODO call agent to generate save name
                    self.save_dir.join("unnamed.json")
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
        let json = fs::read_to_string(path)?;
        let session: Session = serde_json::from_str(&json)?;

        bot.set_topic(session.topic);
        bot.set_messages(session.messages);

        Ok(())
    }

    fn list_saved_sessions(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let saved_sessions = fs::read_dir(&self.save_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    e.file_name().to_str().map(|s| s.to_string())
                })
            })
            .collect::<Vec<String>>();
        Ok(saved_sessions)
    }
}