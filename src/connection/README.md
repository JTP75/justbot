# Connection Directory

This directory contains client implementations for external API services used by RustBot. These clients handle authentication, request/response processing, and communication with third-party services.

## Overview

The connection module provides abstracted interfaces to external services including:
- **Anthropic API** - Large Language Model (LLM) interactions
- **Qdrant** - Vector database for RAG (Retrieval-Augmented Generation)
- **Voyage AI** - Text embedding generation

## Modules

### `anthropic_client.rs`

Client for interacting with Anthropic's Claude API.

#### Features
- Configurable model selection and token limits
- System prompt support
- Temperature-based randomness control
- Async message processing

#### Configuration
Requires the following:
- `ANTHROPIC_API_KEY` environment variable (loaded from `.env` in config directory)
- `anthropic_config.json` with:
  - `default_model` - Model identifier (e.g., "claude-3-opus-20240229")
  - `max_tokens` - Maximum tokens per response

#### Usage Example
```rust
let client = AnthropicClient::new()?;
let response = client.call_model(
    &messages,
    "You are a helpful assistant",
    1.0  // temperature
).await?;
```

#### Key Methods
- `new()` - Initialize client with API key and configuration
- `call_model(messages, sys_prompt, randomness)` - Send messages to LLM and receive response

### `qdrant_client.rs`

Client for interacting with Qdrant vector database.

#### Features
- Vector storage and retrieval
- Similarity search for RAG pipeline
- Collection management
- Point insertion and querying

#### Purpose
Enables semantic search capabilities by storing and retrieving document embeddings, allowing the bot to find relevant context from its knowledge base.

### `voyage_client.rs`

Client for Voyage AI's embedding API.

#### Features
- Text-to-vector embedding generation
- Batch processing support
- Configurable embedding models

#### Purpose
Converts text documents and queries into high-dimensional vectors for storage in Qdrant and similarity comparison in the RAG pipeline.

## Dependencies

As defined in `Cargo.toml`:
- `anthropic = "0.0.8"` - Official Anthropic SDK
- `qdrant-client = "1.15.0"` - Qdrant vector database client
- `reqwest = "0.12.24"` - HTTP client for API requests
- `tokio = "1.47.1"` - Async runtime
- `dotenvy = "0.15.7"` - Environment variable loading
- `directories = "6.0.0"` - Platform-specific directory paths

## Configuration

### Environment Variables
Store API keys in `.env` file located in the config directory:
```
ANTHROPIC_API_KEY=your_key_here
VOYAGE_API_KEY=your_key_here
```

### Config Files
JSON configuration files should be placed in the config directory:
- `anthropic_config.json` - LLM settings
- `vectordb_config.json` - Vector database settings

## Error Handling

All clients return `Result<T, Box<dyn std::error::Error>>` for consistent error handling. Common error scenarios:
- Missing or invalid API keys
- Network connectivity issues
- Rate limiting
- Invalid request parameters
- Service unavailability

## Architecture Notes

- All API clients are async-first, leveraging Tokio runtime
- Configuration is loaded from project-specific directories using the `directories` crate
- Clients are designed to be instantiated once and reused throughout the application lifecycle
- API keys are never hardcoded and must be provided via environment variables

## Adding New Clients

To add a new external service client:

1. Create a new module file (e.g., `new_service_client.rs`)
2. Implement a struct with configuration fields
3. Add `new()` constructor that loads credentials and config
4. Implement async methods for API interactions
5. Export the module in `mod.rs`
6. Add required dependencies to `Cargo.toml`
7. Document configuration requirements