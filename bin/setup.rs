use std::fs;
use directories::ProjectDirs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(
        env_logger::Env::default()
            .default_filter_or("info")
    ).init();
    log::info!("Beginning setup...");

    let p_dirs = ProjectDirs::from("com", "The justbot Company", "rustbot")
        .ok_or("Could not determine project directories")?;

    // check and create each project directory

    // create config directory if it doesn't exist
    let config_dir = p_dirs.config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)?;
        log::info!("Created config directory at {:?}", config_dir);
    } else {
        log::debug!("Config directory already exists at {:?}", config_dir);
    }

    // create data directory if it doesn't exist
    let data_dir = p_dirs.data_dir();
    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
        log::info!("Created data directory at {:?}", data_dir);
    } else {
        log::debug!("Data directory already exists at {:?}", data_dir);
    }

    // create saves directory if it doesn't exist
    let save_dir_owned = p_dirs.data_dir().join("sessions");
    let save_dir = save_dir_owned.as_path();
    if !save_dir.exists() {
        fs::create_dir_all(save_dir)?;
        log::info!("Created save directory at {:?}", save_dir);
    } else {
        log::debug!("Save directory already exists at {:?}", save_dir);
    }

    // copy each config file to config_dir if they don't exist

    // copy bot_config.json
    if !config_dir.join("bot_config.json").exists() {
        fs::copy("config_files/bot_config.json", config_dir.join("bot_config.json"))?;
        log::info!("Copied bot_config.json to {:?}", config_dir);
    } else {
        log::debug!("bot_config.json already exists, skipping copy.");
    }

    // copy vectordb_config.json
    if !config_dir.join("vectordb_config.json").exists() {
        fs::copy("config_files/vectordb_config.json", config_dir.join("vectordb_config.json"))?;
        log::info!("Copied vectordb_config.json to {:?}", config_dir);
    } else {
        log::debug!("vectordb_config.json already exists, skipping copy.");
    }

    // copy anthropic_config.json
    if !config_dir.join("anthropic_config.json").exists() {
        fs::copy("config_files/anthropic_config.json", config_dir.join("anthropic_config.json"))?;
        log::info!("Copied anthropic_config.json to {:?}", config_dir);
    } else {
        log::debug!("anthropic_config.json already exists, skipping copy.");
    }

    // copy prompts.json
    if !config_dir.join("prompts.json").exists() {
        fs::copy("config_files/prompts.json", config_dir.join("prompts.json"))?;
        log::info!("Copied prompts.json to {:?}", config_dir);
    } else {
        log::debug!("prompts.json already exists, skipping copy.");
    }

    // copy mcp_servers.json
    if !config_dir.join("mcp_servers.json").exists() {
        fs::copy("config_files/mcp_servers.json", config_dir.join("mcp_servers.json"))?;
        log::info!("Copied mcp_servers.json to {:?}", config_dir);
    } else {
        log::debug!("mcp_servers.json already exists, skipping copy.");
    }

    // copy docker-compose.yml
    if !config_dir.join("docker-compose.yml").exists() {
        fs::copy("config_files/docker-compose.yml", config_dir.join("docker-compose.yml"))?;
        log::info!("Copied docker-compose.yml to {:?}", config_dir);
    } else {
        log::debug!("docker-compose.yml already exists, skipping copy.");
    }

    // generate a .env template if it doesn't exist
    let env_path = config_dir.join(".env");
    if !env_path.exists() {
        fs::write(
            &env_path,
            "ANTHROPIC_API_KEY=
VOYAGE_API_KEY="
        )?;
        log::info!("Created .env template at {:?}", env_path);
        log::warn!("Make sure to add your API keys to the .env file: {:?} before running the bot.", env_path);
    } else {
        log::debug!(".env file already exists, skipping creation.");
    }

    log::info!("Setup finished successfully.");
    Ok(())
}