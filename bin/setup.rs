use std::{fs, path::Path};

use puetce::common::{config::PROJECT_DIRS, config_const::{json::MCP_SERVERS_CONFIG, prompts::{PROMPTS_DIR, SYSTEM_BASE, SYSTEM_TOOL}}};

fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or("info")
    ).init();


    match verify_all() {
        Ok(_) => {
            log::info!("All config files already exist. Skipping setup");
            return Ok(());
        },
        Err(_) => {
            log::error!("Config file(s) missing. Entering setup");
        }
    }
    
    log::info!("Beginning setup...");

    // create base project dirs
    //
    if !PROJECT_DIRS.config_dir().exists() {
        fs::create_dir_all(PROJECT_DIRS.config_dir())?;
    }
    if !PROJECT_DIRS.data_dir().exists() {
        fs::create_dir_all(PROJECT_DIRS.data_dir())?;
    }
    if !PROJECT_DIRS.cache_dir().exists() {
        fs::create_dir_all(PROJECT_DIRS.cache_dir())?;
    }

    // create project subdirs
    //
    if !PROJECT_DIRS.config_dir().join("auth").exists() {
        fs::create_dir_all(PROJECT_DIRS.config_dir().join("auth"))?;
    }
    if !PROJECT_DIRS.data_dir().join("sessions").exists() {
        fs::create_dir_all(PROJECT_DIRS.data_dir().join("sessions"))?;
    }

    // copy config
    //
    copy_dir_recursive(Path::new("config"), PROJECT_DIRS.config_dir())?;

    // copy dotenv template to .env
    //
    if !PROJECT_DIRS.config_dir().join(".env").exists() {
        log::info!(".env file does not exist. Creating from template file");
        log::warn!(".env file does not contain the necessary API key(s)");
        fs::copy(
            PROJECT_DIRS.config_dir().join("dotenv_template"),
            PROJECT_DIRS.config_dir().join(".env"),
        )?;
    }

    log::info!("Setup finished. Verifying...");

    match verify_all() {
        Ok(_) => {
            log::info!("All files and directories created. Setup complete");
            Ok(())
        },
        Err(e) => {
            log::error!("Verification failed: {e}");
            Err(std::io::Error::new(std::io::ErrorKind::Other, format!("{e}")))
        }
    }    
}

/// recursively copy a directory from `src` to `dst` without replacing existing files
/// 
/// 
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dst.join(&file_name);

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            if !dest_path.exists() {
                fs::copy(&path, &dest_path)?;
            }
        }
    }

    Ok(())
}

/// verify existence all config/cache/data files and directories
/// 
/// 
fn verify_all() -> Result<(), Box<dyn std::error::Error>> {
    use puetce::common::config_const::json::{
        BOT_CONFIG, ANTHROPIC_CONFIG, VECTORDB_CONFIG, EMBEDDING_CONFIG
    };

    fn exists_err(dir: &Path, msg: &str) -> Result<(), Box<dyn std::error::Error>> {
        dir.exists().then_some(0).ok_or(msg)?;
        Ok(())
    }

    // config dir
    exists_err(PROJECT_DIRS.config_dir(), 
        format!("Config dir {:?} doesn't exist",
        PROJECT_DIRS.config_dir()).as_str())?;

    // config dir config files
    exists_err(&PROJECT_DIRS.config_dir().join(BOT_CONFIG), 
        format!("{BOT_CONFIG} file doesn't exist").as_str())?;
    exists_err(&PROJECT_DIRS.config_dir().join(ANTHROPIC_CONFIG), 
        format!("{ANTHROPIC_CONFIG} file doesn't exist").as_str())?;
    exists_err(&PROJECT_DIRS.config_dir().join(VECTORDB_CONFIG), 
        format!("{VECTORDB_CONFIG} file doesn't exist").as_str())?;
    exists_err(&PROJECT_DIRS.config_dir().join(EMBEDDING_CONFIG), 
        format!("{EMBEDDING_CONFIG} file doesn't exist").as_str())?;
    exists_err(&PROJECT_DIRS.config_dir().join(MCP_SERVERS_CONFIG), 
        format!("{MCP_SERVERS_CONFIG} file doesn't exist").as_str())?;
    
    // config dir .env file
    exists_err(&PROJECT_DIRS.config_dir().join(".env"), 
        format!(".env file doesn't exist").as_str())?;    

    // config dir prompts dir
    exists_err(&PROJECT_DIRS.config_dir().join(PROMPTS_DIR), 
        format!("{PROMPTS_DIR} dir doesn't exist").as_str())?;

    // config dir prompts dir prompt txt files
    exists_err(&PROJECT_DIRS.config_dir().join(PROMPTS_DIR).join(SYSTEM_BASE), 
        format!("{SYSTEM_BASE} file doesn't exist").as_str())?;
    exists_err(&PROJECT_DIRS.config_dir().join(PROMPTS_DIR).join(SYSTEM_TOOL), 
        format!("{SYSTEM_TOOL} file doesn't exist").as_str())?;

    // cache dir
    exists_err(PROJECT_DIRS.cache_dir(), 
        format!("Cache dir {:?} doesn't exist",
        PROJECT_DIRS.cache_dir()).as_str())?;

    // data dir
    exists_err(PROJECT_DIRS.data_dir(), 
        format!("Data dir {:?} doesn't exist",
        PROJECT_DIRS.data_dir()).as_str())?;
    
    // data dir motd file
    exists_err(&PROJECT_DIRS.config_dir().join(BOT_CONFIG), 
        format!("{BOT_CONFIG} file doesn't exist").as_str())?;

    // data dir sessions dir
    exists_err(&PROJECT_DIRS.data_dir().join("sessions"), 
        format!("sessions dir doesn't exist").as_str())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::verify_all;

    #[test]
    fn test_verify_config_files() {
        let result = verify_all();
        assert!(result.is_ok(), "Verification failed: {}", result.err().unwrap());
    }
}