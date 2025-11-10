# App Module (Core)

This directory contains the core implementation of Puetce, including the bot engine, session management, and conversation handling logic.

The `app` module provides the fundamental components that power the chatbot's functionality, including:
- Bot configuration and state management
- Conversation session handling
- Message processing and LLM interaction
- RAG (Retrieval-Augmented Generation) pipeline integration
- Startup/shutdown routines
- Integration with tools

## Modules

### `bot.rs`

The main bot implementation containing the `PuetceApp` struct.

#### `PuetceApp` Responsibilities
- Initialization and setup
- Command routing
- State management
- Integration between project modules
- Sync callbacks to async `ClientManager` methods

#### `ClientManager` Responsibilities
- Async callbacks to client modules
- Async routines involving two or more connection clients
    - e.g. searching the vector database requires calls to the voyage and qdrant clients

### `session.rs`

Session management system for handling conversation state.

#### `SessionManager` Responsibilities
- Creating, loading, saving, and deleting conversation sessions
- Persistence to disk for session continuity

#### Session Structure
Saved session files include:
- Topic of the conversation
- Message history (user and assistant messages)

### `tool_manager.rs`

#### `ToolManager` Responsibilities
- Register MCP and integrated tools
- Execute MCP and integrated tools
- Convert MCP and integrated tools to anthropic-compatible tool definitions (`crate::connection::anthropic_client::ToolDefinition`)
- Maintain MCP server processes
- Kill MCP server process

## Integration

### With Commands
Commands receive mutable references to both `SessionManager` and `PuetceApp`:
```rust
fn exec(&self, sm: &mut SessionManager, bot: &mut PuetceApp, args: &Vec<String>) 
    -> Result<Option<String>, Box<dyn std::error::Error>>
```

This allows commands to:
- Query and modify session state
- Access bot configuration
- Load/save sessions
- Interact with conversation history
- Any other task involving a mutable `PuetceApp` ref

### With Connection Clients
The bot coordinates with external services:
- **AnthropicClient** - Sends conversation history to LLM
- **QdrantClient** - Retrieves relevant context from vector database
- **VoyageClient** - Generates embeddings for RAG queries

### With RAG Pipeline
The bot integrates RAG functionality:
1. User query is embedded via Voyage AI
2. Qdrant performs similarity search for relevant context
3. Retrieved context is injected into system prompt
4. Enhanced prompt sent to Anthropic Claude
5. Response incorporates retrieved knowledge

## Configuration

The Puetce core uses configuration from:
- **Project Directories** - JSON config files in platform-specific config/data paths via `directories` crate
- **Environment Variables** - API keys loaded via `dotenvy`

## Data Flow

Example data flow for tool use call involving RAG:

```
0. User Input
    ↓
1. Command Parser (if command detected)
    ↓
2. Tool Use Pipeline (get list of available tools)
    ↓
3. PuetceApp (coordinate first request)
    ↓
4. AnthropicClient (send query + tools to anthropic)
    ├─→ Process query and tools
    └─→ Respond with tool use request
    ↓
5. ToolManager (route tool use request)
    └─→ Execute RAG tool
    ↓
6. RAG Pipeline
    ├─→ VoyageClient (embed query)
    ├─→ QdrantClient (similarity search)
    └─→ Context Retrieval
    ↓
7. PuetceApp (coordinate second request)
    ↓
8. AnthropicClient (send query + tools to anthropic)
    ├─→ Process tool result
    └─→ Respond to user using tool results
    ↓
9. Response Processing
    ↓
10. Output to User
```

## Error Handling

All core components use `Result<T, Box<dyn std::error::Error>>` for consistent error propagation:
- Session file I/O errors
- API communication failures
- Invalid configuration
- Serialization/deserialization errors