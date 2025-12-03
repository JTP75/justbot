use std::{fs, sync::Arc};
use fd_lock::{RwLock, RwLockWriteGuard};

use puetce::{
    app::{
        connection_manager::ConnectionManager, http::HttpServer, tool_manager::ToolManager
    }, 
    common::config, 
    mcp::McpConfig
};

// rw lock

fn get_fd_lock() -> Result<RwLockWriteGuard<'static, fs::File>, Box<dyn std::error::Error>> {
    let lock_path = config::PROJECT_DIRS.cache_dir().join(".backend.lock");

    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(&lock_path)?;
    // allocate the lock on the heap and leak it so the returned guard can live for 'static
    let lock_box: &'static mut RwLock<fs::File> = Box::leak(Box::new(RwLock::new(file)));

    match lock_box.try_write() {
        Ok(guard) => Ok(guard),
        Err(_) => Err("Lock already held".into())
    }
}

// main

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let _ = env_logger::builder().filter_level(log::LevelFilter::Info).try_init();

    // STARTUP PROCEDURE
    // ============================================================================
    log::info!("Entering startup...");
    log::info!("Acquiring lock...");

    let guard = match get_fd_lock() {
        Ok(guard) => guard,
        Err(e) => {
            log::error!("{}", e);
            return Err(e);
        }
    };

    log::info!("Lock acquired");
    log::info!("Initializing managers...");

    let conn_mgr = ConnectionManager::new();
    let mut tool_mgr = ToolManager::new();

    log::info!("Managers created");
    log::info!("Starting docker-compose services... ");
    
    let compose_file = config::PROJECT_DIRS.config_dir().join("docker-compose.yml");
    let output = std::process::Command::new("docker-compose")
        .arg("-f")
        .arg(&compose_file)
        .arg("up")
        .arg("-d")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "docker-compose up service(s) failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    log::info!("Docker-compose services started");
    log::info!("Registering and Starting MCP servers... ");

    let server_file = config::PROJECT_DIRS.config_dir().join("mcp_servers.json");
    let json = fs::read_to_string(server_file).unwrap();
    let mcp_config: McpConfig = serde_json::from_str(&json)?;
    for mcp_server in mcp_config.mcp_servers {
        let args: Vec<&str> = mcp_server.args.iter().map(|s| s.as_str()).collect();
        if let Err(e) = tool_mgr.register_mcp_server(
            mcp_server.name.clone(), 
            &mcp_server.command, 
            &args,
            mcp_server.env
        ) {
            log::error!("Failed to register server '{}':\n{}", mcp_server.name, e);
        } else {
            log::info!("Successfully registered server '{}'", mcp_server.name);
        }
    }

    log::info!("MCP servers started");
    log::info!("Startup complete");

    // HTTP SERVER
    // ============================================================================
    log::info!("Starting HTTP server");

    let mut server = HttpServer::new();
    server.init_router(Arc::new(conn_mgr), Arc::new(tool_mgr))?;
    server.init_tcp_listener().await?;

    log::info!("Hosting backend on {}", server.addr());
    server.serve().await?;

    // SHUTDOWN PROCEDURE
    // ============================================================================
    log::info!("Entering shutdown...");

    log::info!("Kill MCP server processes... ");

    // tools are killed when tool mgr is dropped
    drop(server);
    
    log::info!("MCP server processes stopped.");
    log::info!("Shutting down docker-compose services... ");

    let compose_file = config::PROJECT_DIRS.config_dir().join("docker-compose.yml");
    let output = std::process::Command::new("docker-compose")
        .arg("-f")
        .arg(&compose_file)
        .arg("down")
        .output()?;
    if !output.status.success() {
        return Err(format!(
            "docker-compose down service(s) failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }
    
    log::info!("Docker-compose services stopped.");
    log::info!("Releasing lock...");

    drop(guard);

    log::info!("Lock released");
    log::info!("Shutdown complete");

    Ok(())
}