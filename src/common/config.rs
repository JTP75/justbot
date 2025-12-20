use std::{collections::HashMap, fs};

use directories::ProjectDirs;
use once_cell::sync::Lazy;

pub const QUALIFIER: &str = "com";
pub const ORGANIZATION: &str = "puetceco";
pub const APPLICATION: &str = "rustbot";
pub const MODELS: &[&str] = &[
    "claude-haiku-4-5-20251001",
    "claude-sonnet-4-5-20250929",
    "claude-opus-4-5-20251101",
];

pub static PROJECT_DIRS: Lazy<ProjectDirs> = 
    Lazy::new(|| ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap());


/// Retrieves config value given a filename and key
/// 
/// - assumes that all config files are stored in `ProjectDirs::config_dir()`
/// - works for any integer type, boolean, or string
pub fn get_config<T>(filename: &str, key: &str) -> Result<T,Box<dyn std::error::Error>> 
    where T: std::str::FromStr, T::Err: std::error::Error + 'static 
{
    let path = PROJECT_DIRS.config_dir().join(filename);
    let json = fs::read_to_string(&path)?;
    let map: HashMap<String, serde_json::Value> = serde_json::from_str(&json)?;
    match map.get(key) {
        Some(val) => {
            let string_val = if let Some(s) = val.as_str() {
                s.to_string()
            } else {
                match val {
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => val.to_string()
                }
            };
            Ok(string_val.parse::<T>()?)
        },
        None => Err(format!("Could not find key '{}' in '{}'", key, filename).into())
    }
}

/// Retrieves a prewritten prompt from the prompts directory
/// 
/// - assumes the `prompts` directory is in `ProjectDirs::config_dir()`
pub fn get_prompt(filename: &str) -> Result<String,Box<dyn std::error::Error>> {
    let path = PROJECT_DIRS.config_dir()
        .join("prompts")
        .join(filename);
    let content = fs::read_to_string(&path)?;
    Ok(content)
}

#[cfg(test)]
mod tests { 
    use crate::common::config_const::{
        json::{ANTHROPIC_CONFIG, BOT_CONFIG, EMBEDDING_CONFIG, VECTORDB_CONFIG}, 
        keys::{
            ANTHROPIC_VERSION, BASE_URL, CLIENT_HOST, DEFAULT_COLLECTION, DEFAULT_MODEL, DEFAULT_NAME, DEFAULT_SEARCH_LIMIT, DIM, ENABLE_ANTHROPIC, ENABLE_CUSTOM_TOOLS, ENABLE_LOCAL, ENABLE_MCP, ENABLE_PDF_EMBEDDING, ENABLE_QDRANT, ENABLE_VOYAGE, GRPC_PORT, HOST, MAX_INPUT_TPM, MAX_OUTPUT_TPM, MAX_TOKENS, MOTD_FILENAME, PORT, REST_PORT, START_DIR, VOYAGE_MODEL, VOYAGE_URL
        }, 
        prompts::SYSTEM_TOOL
    };

    use super::*;

    fn test_config_exists(filename: &str, key: &str) -> () {
        let _: String = get_config(BOT_CONFIG, START_DIR)
            .expect(format!("failed to retrieve {key} from {filename}").as_str());
    }

    #[test]
    fn test_get_config() {
        let use_anthropic: bool = get_config(ANTHROPIC_CONFIG, ENABLE_ANTHROPIC)
            .expect("get config failed");
        assert!(use_anthropic);
    }

    #[test]
    fn test_get_prompt() {
        let _sp = get_prompt(SYSTEM_TOOL)
            .expect("get prompt failed");
    }

    #[test]
    fn test_all_bot_configs() {
        test_config_exists(BOT_CONFIG, START_DIR);
        test_config_exists(BOT_CONFIG, DEFAULT_NAME);
        test_config_exists(BOT_CONFIG, MOTD_FILENAME);
        test_config_exists(BOT_CONFIG, HOST);
        test_config_exists(BOT_CONFIG, PORT);
        test_config_exists(BOT_CONFIG, CLIENT_HOST);
        test_config_exists(BOT_CONFIG, ENABLE_MCP);
        test_config_exists(BOT_CONFIG, ENABLE_CUSTOM_TOOLS);
    }

    #[test]
    fn test_all_anthropic_configs() {
        test_config_exists(ANTHROPIC_CONFIG, ENABLE_ANTHROPIC);
        test_config_exists(ANTHROPIC_CONFIG, BASE_URL);
        test_config_exists(ANTHROPIC_CONFIG, ANTHROPIC_VERSION);
        test_config_exists(ANTHROPIC_CONFIG, MAX_TOKENS);
        test_config_exists(ANTHROPIC_CONFIG, DEFAULT_MODEL);
        test_config_exists(ANTHROPIC_CONFIG, MAX_INPUT_TPM);
        test_config_exists(ANTHROPIC_CONFIG, MAX_OUTPUT_TPM);
    }

    #[test]
    fn test_all_vdb_configs() {
        test_config_exists(VECTORDB_CONFIG, ENABLE_QDRANT);
        test_config_exists(VECTORDB_CONFIG, HOST);
        test_config_exists(VECTORDB_CONFIG, REST_PORT);
        test_config_exists(VECTORDB_CONFIG, GRPC_PORT);
        test_config_exists(VECTORDB_CONFIG, DEFAULT_COLLECTION);
        test_config_exists(VECTORDB_CONFIG, DIM);
        test_config_exists(VECTORDB_CONFIG, DEFAULT_SEARCH_LIMIT);
    }

    #[test]
    fn test_all_mbed_configs() {
        test_config_exists(EMBEDDING_CONFIG, ENABLE_PDF_EMBEDDING);
        test_config_exists(EMBEDDING_CONFIG, ENABLE_VOYAGE);
        test_config_exists(EMBEDDING_CONFIG, VOYAGE_URL);
        test_config_exists(EMBEDDING_CONFIG, VOYAGE_MODEL);
        test_config_exists(EMBEDDING_CONFIG, ENABLE_LOCAL);
        test_config_exists(EMBEDDING_CONFIG, HOST);
        test_config_exists(EMBEDDING_CONFIG, PORT);
    }
}