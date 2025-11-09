# Commands Module

This directory contains the command system implementation for RustBot. Commands are modular, self-registering units that extend the bot's functionality.

The command system uses a trait-based architecture with automatic registration via the `#[ctor::ctor]` attribute. Each command is a separate module that implements the `Command` trait and registers itself with the global `REGISTRY`.

This design makes it relatively easy for contributors to add new commands (see bottom).

## Modules

There is a separate module for each implemented command. The currently supported commands are
- hello
- help
- message
- message_rag
- message_tool
- message_knowledge
- schedule
- news
- motd
- store
- get_collection
- set_collection
- get_tools
- whereami
- ls
- cd
- new
- save
- load
- list
- exit
- wexit

## Configuration

- **Prompts**: found in `prompts.json`
- **System Prompts**: found in `bot_config.json`

## Integration

The commands module integrates via the `REGISTRY` in src/commands/mod.rs. This registry is used directly by `RustBot`.

## Creating a New Command

To create a new command:

1. Create a new module file in `src/commands/`
2. Define a struct for your command
3. Implement the `Command` trait
4. Add a registration function with `#[ctor::ctor]`
5. Add public module to `src/commands/mod.rs`

### Example Command Template

```rust
// in src/commands/mycommand.rs
use crate::rustbot::{bot::RustBot, session::SessionManager};
use super::{Command, REGISTRY};

pub struct MyCommand;

impl Command for MyCommand {
    fn name(&self) -> &str { "mycommand" }
    fn aliases(&self) -> Vec<&str> { vec!["mc", "mycmd"] }
    fn desc(&self) -> &str { "Brief description" }
    fn help(&self) -> &str { "Usage: mycommand [args]\nDetailed help text" }
    fn exec(&self, _sm: &mut SessionManager, _bot: &mut RustBot, args: &Vec<String>) 
        -> Result<Option<String>, Box<dyn std::error::Error>> {
        // implementation
        Ok(Some("Command output".to_string()))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "mycommand".into(), 
        || Box::new(MyCommand),
    );
}
```

```rust
// in src/commands/mod.rs

// add this line:
pub mod mycommand;
```

### Notes

- Commands return `Result<Option<String>, Box<dyn std::error::Error>>`
- Return `Ok(Some(String))` for output to display
- Return `Ok(None)` for silent success
- Return `Err(...)` for error conditions
- Commands have access to `SessionManager` for session state and `RustBot` for bot configuration