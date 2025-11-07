use std::{ffi::OsStr, fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::{common::{config::{APPLICATION, ORGANIZATION, QUALIFIER}, types::{ContentBlock, Message, Role}}, rustbot::bot::RustBot};

/// Serializable struct that can be saved to a json file
/// 
/// - contains topic string and messages data
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub topic: String,
    pub messages: Vec<Message>,
}

pub struct SessionManager {
    pub data_dir: PathBuf,
    pub save_dir: PathBuf,
    pub current: PathBuf,
}

impl SessionManager {

    // public

    /// Create a new SessionManager
    /// 
    /// - technically might panic, but almost certainly not
    pub fn new() -> Self {
        let project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap();
        SessionManager { 
            data_dir: project_dirs.data_dir().join(""),
            save_dir: project_dirs.data_dir().join("sessions/"),
            current: PathBuf::new()
        }
    }

    /// Save conversation and topic string to a file
    pub fn save_session(&mut self, bot: &mut RustBot, filename_arg: Option<String>) -> Result<(),Box<dyn std::error::Error>> {
        let path = if let Some(filename) = filename_arg {
            let path = PathBuf::from(filename.clone());
            let stem = path.file_stem()
                .unwrap_or(OsStr::new(&filename));
            if let Some(topic) = stem.to_str() {
                bot.set_topic(topic.replace("_"," "));
            }
            self.save_dir.join(filename)
        } else if !self.current.as_os_str().is_empty() {
            self.current.clone()
        } else {
            let topic = self.generate_topic(bot)?;
            bot.set_topic(&topic);
            let filename = self.get_filename_from_topic(topic);
            self.save_dir.join(filename)
        };
        self.save_session_as_json(path.clone(), bot)
    }

    /// Load conversation and topic string from a file
    pub fn load_session(&mut self, bot: &mut RustBot, filename: String) -> Result<(),Box<dyn std::error::Error>> {
        let path = self.save_dir.join(filename);
        self.load_session_from_json(path.clone(), bot)
    }

    /// Get a list of session files in the save_dir
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

    // private

    fn get_filename_from_topic(&self, topic: String) -> String {
        let file_stem = topic.trim().replace(" ","_").to_ascii_lowercase();
        format!("{file_stem}.json")
    }

    fn generate_topic(&self, bot: &RustBot) -> Result<String, Box<dyn std::error::Error>> {
        let mut convo_copy = bot.get_messages();
        let topic_prompt = Message {
            role: Role::User,
            content: vec![ContentBlock::Text { 
                text: r#"What is the topic of this conversation? This should be extremely short; try to keep
                the character count less than 10. If that is too short, the hard maximum is 30 characters."#.into()
            }]
        };
        convo_copy.push(topic_prompt);
        
        let sys_prompt: String = crate::common::config
            ::get_config("bot_config.json","base_sys_prompt")?;
        let topic_response = bot.query_llm(&convo_copy, &sys_prompt, 0.0)?;

        match topic_response.content.first() {
            Some(ContentBlock::Text { text }) => Ok(text.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect()),
            Some(_) => Err("Unexpected content block type".into()),
            None => Err("Response content is empty".into())
        }
    }

    pub fn save_motd(&self, bot: &RustBot) -> Result<(),Box<dyn std::error::Error>> {
        if !self.data_dir.exists() { fs::create_dir_all(&self.data_dir)?; }

        let json = serde_json::to_string_pretty(&bot.get_motd())?;
        let filename: String = crate::common::config
            ::get_config("bot_config.json","motd_filename")?;
        Ok(fs::write(&self.data_dir.join(filename), json)?)
    }

    pub fn load_motd(&self, bot: &mut RustBot) -> Result<(),Box<dyn std::error::Error>> {
        let filename: String = crate::common::config
            ::get_config("bot_config.json","motd_filename")?;
        let json = fs::read_to_string(&self.data_dir.join(filename))?;
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
    fn test_show_directories() {
        let sm = SessionManager::new();

        println!("Data dir:   {}", sm.data_dir.display());
        println!("Save dir:   {}", sm.save_dir.display());
    }
}