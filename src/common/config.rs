use std::{collections::HashMap, fs};

use directories::ProjectDirs;

pub fn get_config<T>(filename: &str, key: &str) -> Result<T,Box<dyn std::error::Error>> 
    where 
        T: std::str::FromStr, 
        T::Err: std::error::Error + 'static 
{
    let project_dirs = ProjectDirs::from("com", "Justin Inc.", "rustbot").unwrap();
    let path = project_dirs.config_dir().join(filename);
    let json = fs::read_to_string(&path)?;
    let map: HashMap<String, serde_json::Value> = serde_json::from_str(&json)?;
    match map.get(key) {
        Some(val) => {
            // Try to get as string first, otherwise convert to string
            let string_val = if let Some(s) = val.as_str() {
                s.to_string()
            } else {
                // For numbers, booleans, etc., convert without quotes
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