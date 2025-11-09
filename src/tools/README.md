# Tools Module

The `tools` module provides a framework for integrating custom tools into rustbot. These tools allow the AI agent to perform tasks according to the tool specifications. This directory contains tool definitions, implementations, and management utilities.

## Modules

There is a separate module for each implemented command. The currently supported commands are
- calendar [deprecated]
- get_greeting
- motd
- rag

## Configuration

- Confiugrations for the `motd` and `rag` tools are in `bot_config.json`

## Integration

The tools module integrates via the `REGISTRY` in src/tools/mod.rs. This registry is used directly by `RustBot`.

## Creating a New Tool

### General

#### Tool Trait Implementation

All tools must implement the `Tool` trait with the following methods:

```rust
pub trait Tool {
    fn name(&self) -> &str;                    // Tool identifier
    fn description(&self) -> &str;             // AI-readable description
    fn input_schema(&self) -> ToolInputSchema; // Input parameter schema
    fn exec(&self, bot: &mut RustBot, args: &serde_json::Value)
        -> Result<Option<String>, Box<dyn std::error::Error>>; // Execution logic
}
```

#### Tool Definition

When defining a tool, all the descriptions (for the tool and for each property) should be **as thorough as possible**. More thorough descriptions make it easier for the AI to understand how to use your tool. This is especially important for 

- **`name`**: Short and simple
    - All tools implemented here should be prefixed with `custom-...` to indicate to the AI agent that this is a custom tool, not an MCP tool.
    - Only allows alphanumeric, '-', and '_' characters
- **`description`**: Long and thorough
    - Describe the behavior of the tool
- **`input_schema`**: As necessary
    - Consists of *properties*
    - Each property needs:
        - `type`: the json data type of the property (i.e. "number", "string")
        - `title`: short name for the property
        - `description`: thorough description of the property, including limit (i.e. "this number should be a positive integer less than 5")
    - See the steps for implementation details

#### Tool Execution Flow

1. **Tool Definition**: Tools are defined with name, description, and input schema
2. **Tool Discovery**: ToolManager collects all available tools from registry and MCP servers
3. **Tool Selection**: LLM selects appropriate tool based on use case
4. **Execution**: ToolManager routes execution to either integrated or MCP tool
5. **Response**: Tool returns result as `Result<Option<String>>`

### Steps

To create a new tool:
1. Create a new file in `src/tools/` directory
2. Define a struct for your tool
3. Implement the `Tool` trait
4. Add a registration function with `#[ctor::ctor]`
5. Add public module to `src/tools/mod.rs`

#### Example Tool Template

```rust
// in src/tools/my_tool.rs

use crate::tools::{Tool, ToolInputSchema, ToolInputSchemaBuilder};
use crate::rustbot::bot::RustBot;

pub struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &str {
        "custom-my_tool"
    }

    fn description(&self) -> &str {
        "A thorough description of what this tool does"
    }

    fn input_schema(&self) -> ToolInputSchema {
        ToolInputSchemaBuilder::default()
            .property("param1", "string", "Description of param1")          // required property
            .optional_property("param2", "number", "Description of param2") // optional property
            .build()
            .expect("Failed to build schema")
    }

    fn exec(&self, bot: &mut RustBot, args: &serde_json::Value)
        -> Result<Option<String>, Box<dyn std::error::Error>>
    {
        // Extract parameters from args
        let param1 = args.get("param1")
            .and_then(|v| v.as_str())
            .ok_or("Missing param1")?;

        // Implement tool logic
        let result = format!("Processed: {}", param1);

        Ok(Some(result))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "custom-my_tool".into(), 
        || Box::new(MyTool),
    );
}
```

```rust
// in src/tools/mod.rs

// add this line
pub mod my_tool;
```

### Notes

- Tool executions return `Result<Option<String>, Box<dyn std::error::Error>>`
- Return `Ok(Some(String))` for response to agent
- Return `Ok(None)` for silent success
- Return `Err(...)` for error conditions
- Tools have mutable access to `RustBot` for bot functionality