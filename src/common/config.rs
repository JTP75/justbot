use std::{collections::HashMap, fs};

use directories::ProjectDirs;
use once_cell::sync::Lazy;

pub const QUALIFIER: &str = "com";
pub const ORGANIZATION: &str = "The justbot Company";
pub const APPLICATION: &str = "rustbot";
pub static PROJECT_DIRS: Lazy<ProjectDirs> = 
    Lazy::new(|| ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap());

/// Retrieves config value given a filename and key
/// 
/// - assumes that all config files are stored in `ProjectDirs::config_dir()`
/// - works for any integer type, boolean, or string
pub fn get_config<T>(filename: &str, key: &str) -> Result<T,Box<dyn std::error::Error>> 
    where 
        T: std::str::FromStr, 
        T::Err: std::error::Error + 'static 
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