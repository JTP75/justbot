# Puetce

A modern and extensible AI agent implemented in Rust with a command line interface, RAG (Retrieval-Augmented Generation) capabilities, MCP (Model Context Protocol) integration.

## Features

- **🤖 LLM Integration** - Powered by Anthropic's Claude models for intelligent conversations
- **🛜 Configurable MCP Servers** - Configurable MCP servers are registered and launched at runtime
- **🔍 RAG Pipeline** - Semantic search using Qdrant vector database and Voyage AI embeddings
- **📅 MOTD Support** - Daily message of the day display for emotional support (and fun)
- **📦 Docker Backend** - Backend server hosted inside docker container for compatibility and easy setup
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

### Prerequisites

#### Required
- Current version is only tested on WSL2 Ubuntu
- Rust 1.70+ (2024 edition)
- [Docker Desktop](https://www.docker.com/products/docker-desktop)
- [Anthropic API Key](https://console.anthropic.com/)

#### Optional
- For RAG Capabilities:
  - Needs either Voyage or local embedding model deployment via HTTP server
    - [Voyage AI API Key](https://www.voyageai.com/)
    - **TODO** link for local text embedding deployment
  - Qdrant vector database
    - Locally installed \& deployed using Docker (handled automatically)
  
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
  - **Linux**: `$HOME/.config/rustbot/`
  - **macOS**: `$HOME/Library/Application Support/com.puetceco.rustbot/`
  - **Windows**: `%APPDATA%\puetceco\rustbot\config\`
- Create data directory at:
  - **Linux**: `$HOME/.local/share/rustbot/`
  - **macOS**: `$HOME/Library/Application Support/com.puetceco.rustbot/`
  - **Windows**: `%APPDATA%\puetceco\rustbot\data\`
- Create cache directory at:
  - **Linux**: `$HOME/.cache/rustbot`
  - **macOS**: `$HOME/Library/Caches/com.puetceco.rustbot/`
  - **Windows**: `%APPDATA%\puetceco\rustbot\cache\`
- Create necessary storage subdirectories
- Copy default configuration and environment files to correct locations:
  - [bot_config.json](config/bot_config.json)
  - [anthropic_config.json](config/anthropic_config.json)
  - [vectordb_config.json](config/vectordb_config.json)
  - [embedding_config.json](config/embedding_config.json)
  - [mcp_servers.json](config/mcp_servers.json)
  - [dotenv_template](config/dotenv_template)

**NOTE**: The docker-compose.yml is configured for Linux by default. If running on a different platform, you will need to change the backend volume mappings appropriately.

For macOS:

```yml
services:
  backend:
    # ...
    volumes:
      - ${HOME}/Library/Application Support/com.puetceco.rustbot:/root/.config/rustbot
      - ${HOME}/Library/Application Support/com.puetceco.rustbot:/root/.local/share/rustbot
      - ${HOME}/Library/Caches/com.puetceco.rustbot:/root/.cache/rustbot
    # ...
```

### 3. Configure API Key(s)

After running setup, edit the `.env` file created in your config directory (according to the [template](config/dotenv_template)). The Anthropic API key is always mandatory and the Voyage key is only needed you want to set up the RAG pipeline *without* a local embedding model.

```env
ANTHROPIC_API_KEY=INSERT_REQUIRED_ANTHROPIC_KEY_HERE
VOYAGE_API_KEY=OPTIONAL_VOYAGE_KEY
```

### 4. Edit Configurations 

By default, only minimal features are enabled, including:

- Anthropic API Integration
- Custom tools (get/set MOTD, greeting)

#### RAG Pipeline Setup

1. Set `use_voyage_embedding` to `true` in [embedding_config.json](config/embedding_config.json)
    - Alternatively, set up local embedding model (TODO)
2. Set `enable_qdrant` to `true` in [vectordb_config.json](config/vectordb_config.json)
3. Copy new config files to config directory (overwrite)

**NOTE**: You will need to run docker compose with the "rag" profile to launch qdrant. Additionally, if you setup local embedding, you will need to run with "localrag" profile:

```bash
docker compose --profile rag up -d
```

```bash
docker compose --profile localrag up -d
```

#### MCP Servers

1. Set `enable_mcp_servers` to `true` in [bot_config.json](config/bot_config.json)
2. Add MCP servers to [mcp_servers.json](config/bot_config.json)
    - [MCP Configuration Instructions](src/mcp/README.md#configuration)
3. Copy new config files to config directory (overwrite)

### 5. Build/Install

Frontend client
```bash
cargo install --path . --bin puetce-client
```

Backend server
```bash
docker-compose build
```

## Usage

### Starting Puetce

Start backend server:
```bash
docker-compose up -d                      # no rag pipeline
docker-compose --profile rag up -d        # rag pipeline with voyage embedding
docker-compose --profile localrag up -d   # rag pipeline with local embedding model (requires separate installation)
```

This handles:
1. Starting each configured MCP server
2. Hosting the Qdrant vector database service
3. Hosting the backend server

Start frontend client instance:
```bash
puetce-client
```

This handles:
1. Initializing the app and session manager
2. Starting the CLI

### Accessing the Qdrant VectorDB

If the `rag` profile is enabled, you can open the [Qdrant Dashboard](http://localhost:6333/dashboard) in your browser on localhost. This allows access to existing collections, tutorials, and visualization tools.

### Basic Interaction

```
Today is Monday, January 1, 2024.
Here is todays MOTD!
> hello
Hello there! My name is puetce.
> help
Available commands are:
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
> help load
load

DESC
Load a session file over the current session

HELP
Usage: load <filename>
Filename must be specified

> msgt whats on my agenda for today?
25 hours of coding
>
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
- `axum` - HTTP server for backend
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

## License

[MIT License](LICENSE)