# RustBot

A modern, extensible CLI chatbot implemented in Rust with LLM integration and RAG (Retrieval-Augmented Generation) capabilities.

## Overview

RustBot is an interactive command-line chatbot powered by Anthropic's Claude API, featuring a modular command system, persistent conversation sessions, and semantic search through vector database integration. The bot combines natural language processing with a flexible architecture that allows easy extension and customization.

## Features

- **🤖 LLM Integration** - Powered by Anthropic's Claude models for intelligent conversations
- **🔍 RAG Pipeline** - Semantic search using Qdrant vector database and Voyage AI embeddings
- **💬 Session Management** - Persistent conversation history with save/load capabilities
- **🛠️ Extensible Commands** - Modular command system with automatic registration
- **🐳 Docker Integration** - Automatic Qdrant service management via docker-compose
- **⚙️ Configurable** - JSON-based configuration for models, tokens, and behavior
- **📅 MOTD Support** - Daily message of the day display for emotional support

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

- Current version is only tested on WSL2 Ubuntu
- Rust 1.70+ (2024 edition)
- Docker and docker-compose (for Qdrant vector database service)
  - [Docker Desktop](https://www.docker.com/products/docker-desktop) (includes docker-compose)
- API keys for:
  - [Anthropic Claude API Key](https://console.anthropic.com/)
  - [Voyage AI API Key](https://www.voyageai.com/)

### Setup

### 1. Clone the Repository

```
git clone https://github.com/JTP75/justbot.git
cd rustbot
```

### 2. Run Setup Script

RustBot includes an automated setup script that creates necessary directories and configuration files:

```bash
cargo run --bin setup
```

This setup script will:
- Create configuration directory at:
  - **Linux/macOS**: `~/.config/rustbot/`
  - **Windows**: `%APPDATA%\The justbot Company\rustbot\config\`
- Create data directory at:
  - **Linux/macOS**: `~/.local/share/rustbot/`
  - **Windows**: `%APPDATA%\The justbot Company\rustbot\data\`
- Create sessions subdirectory for conversation persistence
- Copy default configuration files:
  - `bot_config.json`
  - `vectordb_config.json` (Qdrant settings)
  - `anthropic_config.json`
  - `prompts.json`
  - `docker-compose.yml`
- Generate a `.env` template file

### 3. Configure API Keys

After running setup, edit the `.env` file created in your config directory:

Add your API keys:
```env
ANTHROPIC_API_KEY=your_anthropic_api_key_here
VOYAGE_API_KEY=your_voyage_api_key_here
```

**Important**: Do not commit the `.env` file to version control!

### 4. Configure Bot Settings (Optional)

You can customize the bot's behavior by editing the JSON configuration files in your config directory:

**`bot_config.json`** - General bot settings:
```json
{
  "default_name": "rustbot",
  "default_collection": "rustbot_memory",
  "motd_filename": "motd.json"
}
```

**`anthropic_config.json`** - Claude model and Anthropic settings:
```json
{
  "default_model": "claude-3-opus-20240229",
  "max_tokens": 4096
}
```

**`vectordb_config.json`** - Qdrant database and Voyage AI settings:
```json
{
  "dimensionality": 512,
  "default_search_limit": 5,

  "embedding_url": "https://api.voyageai.com/v1/embeddings",
  "embedding_model": "voyage-3-lite"
}
```

**`prompts.json`** - Prompts used for generation tasks:
```json
{
  "generate_topic": "What is the topic of this conversation, in five words or less? Write your response with as few words as possible. Response with these five or less words only.",
  "motd": "Write a creative, one-sentence MOTD. It can be related to news/events, or just a friendly message. Respond with only the message."
}
```

### 5. Install PDF-to-Text Util (Optional)

**This part highly unstable and incomplete.**

The current implementation relies on the pdftotext command and will certainly not work outside Linux.

```bash
sudo apt install poppler-utils
```

This allows the `store` command to work for PDF files.

### 6. Build/Install the Project

Build the project
```bash
cargo build --release
```

Install the project
```bash
cargo install --path .
```

## Usage

### Starting RustBot

From installation dir:
```bash
cargo run --release
```

Or use the compiled binary:
```bash
./target/release/rustbot
```

If installed by cargo, run anywhere:
```bash
rustbot
```

When launched, RustBot will:
1. Start the Qdrant vector database service via docker-compose
2. Initialize the bot and session manager
3. Display the message of the day
4. Begin the CLI

### Accessing the Qdrant VectorDB

Once the bot is running you can open the [Qdrant Dashboard](http://localhost:6333/dashboard) in your browser on localhost. This allows access to existing collections, tutorials, and visualization tools.

### Basic Interaction

```
>> Hi, I'm rustbot! Type 'help' to see what I can do.
>> Today is Monday, January 1, 2024.
>> Welcome to RustBot!
<< hello
>> Hello there! My name is rustbot.
<< help
>> Available commands are:
        load                    aliases=()
        wexit                   aliases=(wq)
        hello                   aliases=()
        get-collection          aliases=(getc)
        whereami                aliases=()
        exit                    aliases=(q | quit)
        store                   aliases=()
        message                 aliases=(msg)
        set-collection          aliases=(setc)
        save                    aliases=(w)
        message-rag             aliases=(msg-rag | rag)
        list                    aliases=()
        motd                    aliases=()
        help                    aliases=()
        ...
<< help load
>> load

DESC
Load a session file over the current session

HELP
Usage: load <filename>
Filename must be specified

<< 
```

### Conversation Sessions

Conversation sessions can be saved and loaded using the `save` and `load` commands. Sessions are managed and persisted to:
```
~/.local/share/rustbot/sessions/
```

Command history is saved to:
```
~/.local/share/rustbot/rustyline_history.txt
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

## Error Handling

RustBot uses `Result<T, Box<dyn std::error::Error>>` throughout for comprehensive error handling. Errors are displayed with color-coded output:
- 🟢 Green - Successful responses
- 🔴 Red - Error messages
- 🟡 Yellow - User prompts

## License

[Specify your license here]
