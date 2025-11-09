# Connection Module

The connection module provides abstracted interfaces to external API services used by RustBot. These clients handle authentication, request/response processing, and communication with third-party services.

The key features are
- **Async-first design** - All clients use Tokio for async operations
- **Error handling** - Consistent `Result<T, Box<dyn std::error::Error>>` patterns
- **Configuration loading** - Platform-specific directory paths via `directories` crate
- **Reusable clients** - Designed for single instantiation and reuse throughout application lifecycle

## Modules

### `anthropic_client.rs`
Client for interacting with Anthropic's Claude API. Handles message sending, system prompts, and configurable model selection with token limits.

### `qdrant_client.rs`
Client for local Qdrant vector database. Enables semantic search by storing and retrieving document embeddings for the RAG pipeline.

### `voyage_client.rs`
Client for Voyage AI's embedding API. Converts text documents and queries into high-dimensional vectors for similarity search.

### DEPRECATED `google_client.rs`
Client for Google APIs including Calendar and Tasks integration.

## Configuration

API keys and settings are managed via:
- `.env` file for API keys
- JSON configuration files (`anthropic_config.json` and `vectordb_config.json`)

## Integration

The Connection Module Integrates with
- **RustBot** (via `ClientManager`): Sync callbacks and routines for RustBot to use each client