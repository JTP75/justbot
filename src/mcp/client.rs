use std::{
    collections::HashMap, io::{BufRead, BufReader, BufWriter, Write}, process::{Child, ChildStdin, ChildStdout, Command, Stdio}, time::Duration
};

use serde_json::Value;

use crate::mcp::{McpTool, McpToolResult};

#[derive(Debug)]
pub struct McpClient {
    process: Child,
    reader: BufReader<ChildStdout>,
    writer: BufWriter<ChildStdin>,
    id: u64,
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.process.kill();
    }
}

impl McpClient {
    pub fn new(command: &str, args: &[&str], env: Option<HashMap<String, String>>) 
    -> Result<Self, Box<dyn std::error::Error>> {
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

        let mut client = Self { process, reader, writer, id };
        client.init()?;

        Ok(client)
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
                    "name": "rustbot",
                    "version": "0.1.0"
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
        
        let rt = tokio::runtime::Runtime::new()?;
        let mut line = String::new();

        rt.block_on(async {
            let timeout_duration = Duration::from_secs(10);
            
            match tokio::time::timeout(timeout_duration, async {
                self.reader.read_line(&mut line)
            }).await {
                Ok(Ok(_)) => Ok(()),
                Ok(Err(e)) => Err(Box::new(e) as Box<dyn std::error::Error>),
                Err(_) => Err("Read timeout after 10 seconds".into()),
            }
        })?; // this should make the stdin reader time out, but it hangs...

        match serde_json::from_str(&line) {
            Ok(response) => Ok(response),
            Err(e) => {
                log::warn!("json convert failed: {e}\nresponse: {}", line);
                Err(e.into())
            }
        }
    }
}