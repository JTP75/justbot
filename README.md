# Puetce

A modern and extensible AI agent implemented in Rust with a command line interface, RAG (Retrieval-Augmented Generation) capabilities, MCP (Model Context Protocol) integration.

## Features

- **🤖 LLM Integration** - Powered by Anthropic's Claude models for intelligent conversations
- **🛜 Configurable MCP Servers** - Configurable MCP servers are registered and launched at runtime
- **🔍 RAG Pipeline** - Semantic search using Qdrant vector database and Voyage AI embeddings
- **📅 MOTD Support** - Daily message of the day display for emotional support (and fun)
- **📂 Session Management** - Persistent conversation history with save/load capabilities
- **🗨️ Extensible Commands** - Modular command system with automatic registration
- **🛠️ Extensible Custom Tools** - Modular custom tool system with automatic registration
- **⚙️ Configurable** - JSON-based configuration for models, tokens, and behavior

## Modules

Project modules are documented in their respective README files

| Module | Docs | Brief |
|:---|:---|:---|
| `common` | [src/common/README.md](src/common/README.md) | Common functionality for all modules
| `commands` | [src/commands/README.md](src/commands/README.md) | Extensible command modules
| `tools` | [src/tools/README.md](src/tools/README.md) | Extensible custom tool modules
| `connection` | [src/connection/README.md](src/connection/README.md) | Clients for 3rd party services
| `mcp` | [src/mcp/README.md](src/mcp/README.md) | MCP clients and server hosting
| `app` (Core) | [src/app/README.md](src/app/README.md) | Core functionality and integration

## Installation

**(EVERYTHING IN THIS SECTION IS NOT UP TO DATE)**

### Prerequisites

#### Required
- Current version is only tested on WSL2 Ubuntu
- Rust 1.70+ (2024 edition)
- Docker and docker-compose (for Qdrant vector database service)
  - [Docker Desktop](https://www.docker.com/products/docker-desktop) (includes docker-compose)
- API keys for:
  - [Anthropic Claude API Key](https://console.anthropic.com/)
  - [Voyage AI API Key](https://www.voyageai.com/)

#### Optional
- Node.js (for running MCP servers)
  - [Node.js](https://nodejs.org)
- `poppler-utils` (for embedding PDFs)

### Setup

### 1. Clone the Repository

```
git clone https://github.com/JTP75/puetce.git
cd puetce
```

### 2. Run Setup Script

Puetce includes an automated setup script that creates necessary directories and configuration files:

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

### Starting Puetce

From installation dir:
```bash
cargo run --release
```

Or use the compiled binary:
```bash
./target/release/puetce
```

If installed by cargo, run anywhere:
```bash
puetce
```

When launched, Puetce will:
1. Start the Qdrant vector database service via docker-compose
2. Initialize the bot and session manager
3. Display the message of the day
4. Begin the CLI

### Accessing the Qdrant VectorDB

Once the bot is running you can open the [Qdrant Dashboard](http://localhost:6333/dashboard) in your browser on localhost. This allows access to existing collections, tutorials, and visualization tools.

### Basic Interaction

```
>> Hi, I'm puetce! Type 'help' to see what I can do.
>> Today is Monday, January 1, 2024.
>> Welcome to Puetce!
<< hello
>> Hello there! My name is puetce.
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
        message-tools           aliases=(msgt)
        ...
<< help load
>> load

DESC
Load a session file over the current session

HELP
Usage: load <filename>
Filename must be specified

<< whats on my agenda for today?
>> 25 hours of coding
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

The `commands` and `tools` modules were designed with extensibility in mind.
- **Adding new commands**: See [Creating a New Command](src/commands/README.md#creating-a-new-command)
- **Adding new tools**: See [Creating a New Tool](src/tools/README.md#creating-a-new-tool)

## Dependencies

- `tokio` - Asynchronous runtime with full feature set
- `serde`/`serde_json` - Serialization and JSON support
- `reqwest` - HTTP client for API calls
- `qdrant-client` - Vector database operations
- `yup-oauth2` - OAuth2 authentication for Google APIs
- `chrono` - Date and time handling
- `directories` - Platform-specific configuration paths
- `dotenvy` - Environment variable loading from `.env` files
- `rustyline` - Interactive CLI with history and line editing
- `log`/`env_logger` - Logging framework and implementation
- `figlet-rs` - ASCII art text generation
- `rand` - Random number generation
- `once_cell` - Lazy static initialization
- `derive_builder` - Builder pattern macros
- `ctor` - Constructor functions

For versions and features, see [Cargo.toml](Cargo.toml)

## Message Indicators

Puetce uses `Result<T, Box<dyn std::error::Error>>` throughout for comprehensive error handling. Errors are displayed with color-coded output:
- 🟡 Yellow - User prompts
- 🔵 Blue - Startup/shutdown messages
- 🟢 Green - Successful responses
- 🔴 Red - Error messages

## License

[MIT License](LICENSE)