use std::{fs, path::PathBuf};

use anthropic::types::Message;
use directories::ProjectDirs;
use std::panic;

pub fn get_save_dir() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Justin Inc.", "rustbot") {
        proj_dirs.data_dir().join("")
    } else {
        panic!("get save dir failed");
    }
}

// save style topic_number_date.json
pub fn save_messages_as_json(filename: String, messages: Vec<Message>) -> Result<(),Box<dyn std::error::Error>> {
    let save_dir = get_save_dir();
    if !save_dir.exists() {
        fs::create_dir_all(&save_dir)?;
    }

    let json = serde_json::to_string_pretty(&messages)?;
    fs::write(save_dir.join(filename), json)?;
    
    Ok(())
}

pub fn load_messages_from_json(filename: String) -> Result<Vec<Message>,Box<dyn std::error::Error>> {
    let save_dir = get_save_dir();

    let json = fs::read_to_string(save_dir.join(filename))?;
    let messages: Vec<Message> = serde_json::from_str(&json)?;

    Ok(messages)
}

#[cfg(test)]
mod tests {
    use anthropic::types::{ContentBlock, Role};

    use super::*;

    #[test]
    fn get_save_dir_does_not_panic() {
        // Ensure the function can be called without panicking.
        assert!(panic::catch_unwind(|| get_save_dir()).is_ok());
    }

    #[test]
    fn save_json_creates_new_file() {
        let save_dir = get_save_dir();
        let fname = "test_01.json".to_owned();
        let convo = vec![
            Message {
                role: Role::User,
                content: vec![ContentBlock::Text {
                    text: "i like dih".to_owned()
                }]
            }
        ];
        
        assert!(save_messages_as_json(fname.clone(), convo.clone()).is_ok());

        let file_path = save_dir.join(&fname);
        assert!(file_path.exists());

        // cleanup
        fs::remove_file(save_dir.join(fname)).unwrap();
    }

    #[test]
    fn save_json_reloads() {
        let save_dir = get_save_dir();
        let fname = "test_02.json".to_owned();
        let convo = vec![
            Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Text {
                    text: "i hate dih".to_owned()
                }]
            }
        ];
        
        assert!(save_messages_as_json(fname.clone(), convo.clone()).is_ok());
        
        let saved_convo = load_messages_from_json(fname.clone()).unwrap();
        assert_eq!(convo,saved_convo);

        // cleanup
        fs::remove_file(save_dir.join(fname)).unwrap();
    }


}
