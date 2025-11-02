# RustBot

A modern, extensible CLI chatbot implemented in Rust with LLM integration and RAG (Retrieval-Augmented Generation) capabilities.

## Overview

RustBot is an interactive command-line chatbot powered by Anthropic's Claude API, featuring a modular command system, persistent conversation sessions, and semantic search through vector database integration. The bot combines natural language processing with a flexible architecture that allows easy extension and customization.

## Features

- **🤖 LLM Integration** - Powered by Anthropic's Claude models for intelligent conversations
- **🔍 RAG Pipeline** - Semantic search using Qdrant vector database and Voyage AI embeddings
- **💬 Session Management** - Persistent conversation history with save/load capabilities
- **🛠️ Extensible Commands** - Modular command system with automatic registration
- **📝 Interactive CLI** - Rich readline interface with history and auto-completion
- **🐳 Docker Integration** - Automatic Qdrant service management via docker-compose
- **⚙️ Configurable** - JSON-based configuration for models, tokens, and behavior
- **📅 MOTD Support** - Daily message of the day display

## Architecture

RustBot is organized into four main modules:

### `src/rustbot/`
Core bot implementation including:
- **Bot Engine** - Main `RustBot` struct managing bot state and configuration
- **Session Manager** - Conversation persistence and history management
- **Message Processing** - Coordination between LLM, RAG pipeline, and user input

### `src/commands/`
Extensible command system with:
- **Command Trait** - Interface for all bot commands
- **Auto-registration** - Commands register themselves at startup
- **Built-in Commands** - help, hello, whereami, list, motd, and more

### `src/connection/`
External service clients:
- **AnthropicClient** - Claude API integration
- **QdrantClient** - Vector database operations
- **VoyageClient** - Text embedding generation

### `src/common/`
Shared utilities and configuration management

## Installation

### Prerequisites

- Rust 1.70+ (2024 edition)
- Docker and docker-compose (for Qdrant vector database)
- API keys for:
  - Anthropic Claude
  - Qdrant Cloud (or local instance)
  - Voyage AI

### Setup

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd rustbot
   ```

2. **Install dependencies**
   ```bash
   cargo build --release
   ```

3. **Configure environment variables**
   
   Create `.env` file in the config directory (`~/.config/rustbot/.env`):
   ```env
   ANTHROPIC_API_KEY=your_anthropic_key
   QDRANT_API_KEY=your_qdrant_key
   VOYAGE_API_KEY=your_voyage_key
   ```

4. **Set up configuration files**
   
   Place JSON config files in `~/.config/rustbot/`:
   - `bot_config.json` - Bot name and general settings
   - `anthropic_config.json` - Model selection and token limits
   - `qdrant_config.json` - Vector database settings
   - `voyage_config.json` - Embedding model configuration
   - `docker-compose.yml` - Qdrant service definition

5. **Run the bot**
   ```bash
   cargo run --release
   ```

## Usage

### Starting RustBot

When launched, RustBot will:
1. Start the Qdrant vector database service via docker-compose
2. Initialize the bot and session manager
3. Display the message of the day
4. Present an interactive prompt

### Basic Interaction

```
>> Hi, I'm rustbot! Type 'help' to see what I can do.
>> Today is Monday, January 1, 2024.
>> Welcome to RustBot!

<< hello
>> Hello there! My name is rustbot.

<< help
>> Available commands are:
    help                aliases=()
    hello               aliases=()
    whereami            aliases=()
    list                aliases=()
    ...
```

### Commands

- `help [command]` - Show available commands or detailed help for a specific command
- `hello` - Get a friendly greeting
- `whereami` - Display current working directory
- `list` - Show all saved conversation sessions
- `exit`, `q`, `wq` - Exit the bot (with optional session save)

### Conversation Sessions

Sessions are automatically managed and persisted to:
```
~/.local/share/rustbot/sessions/
```

Command history is saved to:
```
~/.local/share/rustbot/rustyline_history.txt
```

## Configuration

### Directory Structure

RustBot uses platform-specific directories:

**Linux/macOS:**
- Config: `~/.config/rustbot/`
- Data: `~/.local/share/rustbot/`

**Windows:**
- Config: `%APPDATA%\The justbot Company\rustbot\config\`
- Data: `%APPDATA%\The justbot Company\rustbot\data\`

### Model Configuration

Edit `anthropic_config.json`:
```json
{
  "default_model": "claude-3-opus-20240229",
  "max_tokens": 4096
}
```

## Development

### Adding New Commands

1. Create a new file in `src/commands/`
2. Implement the `Command` trait
3. Add auto-registration with `#[ctor::ctor]`
4. Add new public module to `src/commands/mod.rs`


Example: `mycommand.rs`
```rust
use crate::rustbot::{bot::RustBot, session::SessionManager};
use super::{Command, REGISTRY};

pub struct MyCommand;

impl Command for MyCommand {
    fn name(&self) -> &str { "mycommand" }
    fn aliases(&self) -> Vec<&str> { vec![] }
    fn desc(&self) -> &str { "My custom command" }
    fn help(&self) -> &str { "Usage: mycommand" }
    fn exec(&self, sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) 
        -> Result<Option<String>, Box<dyn std::error::Error>> {
        Ok(Some("Command executed!".to_string()))
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

### Project Structure

```
rustbot/
├── src/
│   ├── main.rs           # Entry point and CLI loop
│   ├── rustbot/          # Core bot implementation
│   ├── commands/         # Command system
│   ├── connection/       # API clients
│   └── common/           # Shared utilities
├── Cargo.toml
└── README.md
```

## Dependencies

Key dependencies include:
- `anthropic` - Claude API client
- `qdrant-client` - Vector database operations
- `reqwest` - HTTP client for API calls
- `tokio` - Async runtime
- `rustyline` - Interactive CLI with history
- `serde`/`serde_json` - Serialization
- `chrono` - Date/time handling
- `directories` - Platform-specific paths

## Lifecycle

**Startup:**
1. Initialize logger
2. Start docker-compose services (Qdrant)
3. Load bot configuration
4. Initialize session manager
5. Load MOTD and display greeting
6. Enter interactive loop

**Shutdown:**
1. Save readline history
2. Stop docker-compose services
3. Clean up resources

## Error Handling

RustBot uses `Result<T, Box<dyn std::error::Error>>` throughout for comprehensive error handling. Errors are displayed with color-coded output:
- 🟢 Green - Successful responses
- 🔴 Red - Error messages
- 🟡 Yellow - User prompts

## License

[Specify your license here]