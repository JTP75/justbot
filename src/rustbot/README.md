# RustBot Core Directory

This directory contains the core implementation of RustBot, including the bot engine, session management, and conversation handling logic.

## Overview

The `rustbot` module provides the fundamental components that power the chatbot's functionality, including:
- Bot configuration and state management
- Conversation session handling
- Message processing and LLM interaction
- RAG (Retrieval-Augmented Generation) pipeline integration

## Modules

### `bot.rs`

The main bot implementation containing the `RustBot` struct.

#### Responsibilities
- Bot identity and configuration (name, working directory)
- Initialization and setup
- Coordination between sessions, commands, and external services
- State management across the application lifecycle

#### Key Features
- `get_name()` - Returns the bot's configured name
- `get_cwd()` - Returns the bot's current working directory
- Configuration loading from project directories
- Integration point for LLM client and RAG pipeline

### `session.rs`

Session management system for handling conversation state.

#### `SessionManager` Responsibilities
- Creating, loading, saving, and deleting conversation sessions
- Managing conversation history and context
- Persistence to disk for session continuity
- Session file organization in data directory

#### Key Features
- **Session Persistence** - Automatically saves conversation history
- **Multi-Session Support** - Multiple independent conversation threads
- **Session Listing** - Enumerate all saved sessions
- **Serialization** - JSON-based session storage with `serde`

#### Session Structure
Sessions typically include:
- Unique session identifier
- Message history (user and assistant messages)
- Timestamps using `chrono`
- Metadata (creation date, last modified, etc.)

### Integration Points

#### With Commands
Commands receive mutable references to both `SessionManager` and `RustBot`:
```rust
fn exec(&self, sm: &mut SessionManager, bot: &mut RustBot, args: &Vec<String>) 
    -> Result<Option<String>, Box<dyn std::error::Error>>
```

This allows commands to:
- Query and modify session state
- Access bot configuration
- Load/save sessions
- Interact with conversation history

#### With Connection Clients
The bot coordinates with external services:
- **AnthropicClient** - Sends conversation history to LLM
- **QdrantClient** - Retrieves relevant context from vector database
- **VoyageClient** - Generates embeddings for RAG queries

#### With RAG Pipeline
The bot integrates RAG functionality:
1. User query is embedded via Voyage AI
2. Qdrant performs similarity search for relevant context
3. Retrieved context is injected into system prompt
4. Enhanced prompt sent to Anthropic Claude
5. Response incorporates retrieved knowledge

## Data Flow

```
User Input
    ↓
Command Parser (if command detected)
    ↓
SessionManager (conversation context)
    ↓
RAG Pipeline (if enabled)
    ├─→ VoyageClient (embed query)
    ├─→ QdrantClient (similarity search)
    └─→ Context Retrieval
    ↓
RustBot (coordinates request)
    ↓
AnthropicClient (LLM inference)
    ↓
Response Processing
    ↓
SessionManager (update history)
    ↓
Output to User
```

## Configuration

The rustbot core uses configuration from:
- **Project Directories** - Platform-specific config/data paths via `directories` crate
- **Environment Variables** - API keys loaded via `dotenvy`
- **JSON Config Files** - Model settings, token limits, etc.
- **Runtime State** - Session data, conversation history

### Directory Structure
```
~/.config/rustbot/          # Configuration files
    .env                    # API keys
    anthropic_config.json   # LLM settings
    qdrant_config.json      # Vector DB settings
    voyage_config.json      # Embedding settings

~/.local/share/rustbot/     # Data files
    sessions/               # Saved conversation sessions
        session_*.json
```

## Error Handling

All core components use `Result<T, Box<dyn std::error::Error>>` for consistent error propagation:
- Session file I/O errors
- API communication failures
- Invalid configuration
- Serialization/deserialization errors

## Dependencies

Key dependencies used by rustbot core:
- `anthropic = "0.0.8"` - LLM client
- `qdrant-client = "1.15.0"` - Vector database
- `serde = "1.0.228"` - Serialization
- `serde_json = "1.0.145"` - JSON handling
- `chrono = "0.4.42"` - Timestamps
- `directories = "6.0.0"` - Platform paths
- `tokio = "1.47.1"` - Async runtime

## Usage Example

```rust
// Initialize bot
let mut bot = RustBot::new()?;

// Create session manager
let mut session_manager = SessionManager::new()?;

// Load or create session
session_manager.load_or_create_session("my_chat")?;

// Process user message
let user_message = "Hello, rustbot!";
session_manager.add_user_message(user_message);

// Get response from LLM
let response = bot.generate_response(&session_manager).await?;

// Save session
session_manager.save_current_session()?;
```

## Extension Points

To extend rustbot functionality:

1. **Add New Bot Capabilities** - Extend `RustBot` with new methods
2. **Custom Session Storage** - Implement alternative persistence backends
3. **Enhanced Context Management** - Add metadata, tags, or categorization to sessions
4. **RAG Pipeline Customization** - Modify retrieval strategies or ranking algorithms
5. **Multi-Model Support** - Add support for additional LLM providers

## Thread Safety

- `SessionManager` is designed for single-threaded use per session
- Bot configuration is typically read-only after initialization
- External API clients handle their own thread safety
- Consider using `Arc<Mutex<>>` for shared state in concurrent scenarios