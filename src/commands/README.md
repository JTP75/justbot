# Commands Directory

This directory contains the command system implementation for RustBot. Commands are modular, self-registering units that extend the bot's functionality.

## Overview

The command system uses a trait-based architecture with automatic registration via the `#[ctor::ctor]` attribute. Each command is a separate module that implements the `Command` trait and registers itself with the global `REGISTRY`.

## Architecture

### Command Trait

All commands must implement the `Command` trait with the following methods:

- `name(&self) -> &str` - The primary command name
- `aliases(&self) -> Vec<&str>` - Alternative names for the command
- `desc(&self) -> &str` - Short description of what the command does
- `help(&self) -> &str` - Detailed usage instructions
- `exec(&self, sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>>` - Command execution logic

### Command Registry

The global `REGISTRY` manages all available commands. Commands are automatically registered at program startup using constructor functions marked with `#[ctor::ctor]`.

## Example Commands

### `help`
- **Description**: Get help on command usage
- **Usage**: `help [<command>]`
- **Details**: Shows help and description for a specified command. If no command is specified, lists all available commands with their aliases.

### `hello`
- **Description**: Print a friendly greeting
- **Usage**: `hello`
- **Details**: Takes no arguments. Returns a greeting with the bot's name.

### `whereami`
- **Description**: Prints the current working directory of rustbot
- **Usage**: `whereami`
- **Details**: Takes no arguments. Returns the absolute path of the bot's current working directory.

### `list`
- **Description**: List all saved session files
- **Usage**: `list`
- **Details**: Takes no arguments. Shows all saved session files in the rustbot data directory.

## Creating a New Command

To create a new command:

1. Create a new module file in `src/commands/`
2. Define a struct for your command
3. Implement the `Command` trait
4. Add a registration function with `#[ctor::ctor]`

### Example Template

```rust
use crate::rustbot::{bot::RustBot, session::SessionManager};
use super::{Command, REGISTRY};

pub struct MyCommand;

impl Command for MyCommand {
    fn name(&self) -> &str { "mycommand" }
    fn aliases(&self) -> Vec<&str> { vec!["mc", "mycmd"] }
    fn desc(&self) -> &str { "Brief description" }
    fn help(&self) -> &str { "Usage: mycommand [args]\nDetailed help text" }
    fn exec(&self, sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) 
        -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Implementation
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

## Notes

- Commands return `Result<Option<String>, Box<dyn std::error::Error>>`
- Return `Ok(Some(String))` for output to display
- Return `Ok(None)` for silent success
- Return `Err(...)` for error conditions
- Commands have access to `SessionManager` for session state and `RustBot` for bot configuration