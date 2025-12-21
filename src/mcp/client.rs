#![allow(unused)]

use std::{
    collections::HashMap, 
    io::{BufRead, BufReader, BufWriter, Write}, 
    process::{Child, ChildStdin, ChildStdout, Command, Stdio}, 
    time::Duration
};

use serde_json::Value;

use crate::{common::config_const::{json::BOT_CONFIG, keys::START_DIR}, mcp::{McpRoot, McpRootBuilder, McpTool, McpToolResult}};

#[derive(Debug)]
pub struct McpClient {
    process: Child,
    reader: BufReader<ChildStdout>,
    writer: BufWriter<ChildStdin>,
    id: u64,
    roots: Vec<McpRoot>,
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

impl McpClient {
    pub fn new(command: &str, args: &[&str], env: Option<HashMap<String, String>>) 
    -> Result<Self, Box<dyn std::error::Error>> {
        let sd: String = crate::common::config
            ::get_config(BOT_CONFIG, START_DIR)?;

        let mut command_obj = Command::new(command);
        command_obj
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit());
        for (key,value) in env.unwrap_or_default() {
            command_obj.env(key, value);
        }
        let mut process = command_obj.spawn()?;
        let reader = BufReader::new(process.stdout.take().unwrap());
        let writer = BufWriter::new(process.stdin.take().unwrap());
        let id = 0;
        let roots = vec![McpRootBuilder::default().uri(format!("file://{sd}")).build()?];

        let mut client = Self { process, reader, writer, id, roots };
        client.init()?;

        Ok(client)
    }

    pub fn kill(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(self.process.kill()?)
    }

    pub fn call_tool(&mut self, name: &str, args: &Value)
    -> Result<McpToolResult, Box<dyn std::error::Error>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "tools/call",
            "params": {
                "name": name,
                "arguments": args
            }
        });

        self.send_request(&request)?;
        let response = self.read_response()?;

        Ok(serde_json::from_value(response["result"].clone())?)
    }

    pub fn list_tools(&mut self) -> Result<Vec<McpTool>, Box<dyn std::error::Error>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "tools/list"
        });

        self.send_request(&request)?;
        let response = self.read_response()?;

        match serde_json::from_value::<Vec<McpTool>>(response["result"]["tools"].clone()) {
            Ok(tools) => Ok(tools),
            Err(e) => {
                log::warn!("json convert failed: {e}\nresponse: {}", response);
                Err(e.into())
            }
        }
    }

    // private

    fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "puetce",
                    "version": "0.2.0"
                }
            },
        });

        self.send_request(&request)?;
        let _response = self.read_response()?;

        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });
        self.send_request(&notification)?;

        Ok(())
    }

    fn handle_roots_request(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "result": {
                "roots": self.roots
            }
        });

        Ok(())
    }

    fn next_id(&mut self) -> u64 {
        self.id += 1;
        self.id
    }

    fn send_request(&mut self, request: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string(request)?;
        writeln!(self.writer, "{}", json)?;
        self.writer.flush()?;
        Ok(())
    }

    fn read_response(&mut self) -> Result<Value, Box<dyn std::error::Error>> {
        
        let mut line = String::new();

        // let rt = tokio::runtime::Runtime::new()?;
        // rt.block_on(async {
        //     let timeout_duration = Duration::from_secs(10);
            
        //     match tokio::time::timeout(timeout_duration, async {
        //         self.reader.read_line(&mut line)
        //     }).await {
        //         Ok(Ok(_)) => Ok(()),
        //         Ok(Err(e)) => Err(Box::new(e) as Box<dyn std::error::Error>),
        //         Err(_) => Err("Read timeout after 10 seconds".into()),
        //     }
        // })?; // this should make the stdin reader time out, but it hangs...

        self.reader.read_line(&mut line)?;

        match serde_json::from_str(&line) {
            Ok(response) => Ok(response),
            Err(e) => {
                log::warn!("json convert failed: {e}\nresponse: {}", line);
                Err(e.into())
            }
        }
    }
}

/// These tests use @modelcontextprotocol/server-filesystem
/// 
/// https://github.com/modelcontextprotocol/servers/tree/main/src/filesystem
/// 
/// There may some extern dependency problems
#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, sync::Mutex};
    use crate::mcp::McpContent;

    use super::*;

    const TEST_OUTPUT_DIR: &str = "/home/pacel/misc/rust/rustbot/.ignore/output/";
    static TEST_CLIENT: Mutex<Option<McpClient>> = Mutex::new(None);

    #[ctor::ctor]
    fn setup() {
        let _ = env_logger::builder().filter_level(log::LevelFilter::Debug).try_init();
        log::info!("Starting test server...");

        // setup

        log::info!("Test server started.");
    }

    #[ctor::dtor]
    fn cleanup() {
        log::info!("Stopping test server...");

        // cleanup

        log::info!("Test server stopped.");
    }

    #[test]
    fn test_get_tools() -> () {
        let mut mutex = TEST_CLIENT.lock().unwrap();
        let mut client = mutex.take().unwrap();

        let tools = client.list_tools();

        mutex.replace(client);
        drop(mutex);

        assert!(tools.is_ok(), "{}", tools.err().unwrap());
        let tools = tools.unwrap();

        log::info!("{}", tools.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join("\n"));
    }

    #[test]
    fn test_call_tool() -> () {
        let path_str = format!("{}/{}", TEST_OUTPUT_DIR, "test.txt");
        let content = "Hello, World!";
        fs::write(&path_str, content).unwrap();

        let mut mutex = TEST_CLIENT.lock().unwrap();
        let mut client = mutex.take().unwrap();

        let rslt = client.call_tool("read_text_file", &serde_json::json!({ "path": path_str }));

        mutex.replace(client);
        drop(mutex);

        assert!(rslt.is_ok());
        let rslt = rslt.unwrap();
        assert!(rslt.is_error.is_none() || !rslt.is_error.unwrap());

        log::info!("{:?}", rslt);

        let read_content = match &rslt.content[0] {
            McpContent::Text { text } => text,
        };
        assert_eq!(content,read_content);

        fs::remove_file(&path_str).unwrap();
    }
}