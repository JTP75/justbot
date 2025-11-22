use std::{fs, sync::Arc};

use puetce::{
    app::{
        connection_manager::ConnectionManager, http::HttpServer, tool_manager::ToolManager
    }, 
    common::config, 
    mcp::McpConfig
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let _ = env_logger::builder().try_init();

    // STARTUP PROCEDURE
    // ============================================================================
    log::info!("Entering startup...");
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
    log::info!("Shutdown complete");

    Ok(())
}