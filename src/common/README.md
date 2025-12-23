# Common Module

The common module provides shared utilities and configurations used throughout Puetce.

## Modules

### `config.rs`
Configuration management and loading from JSON files and environment variables

### `config_const.rs`
Define public constants for config file names, keys, etc.

### `pdf.rs` 
Utility module for converting PDFs to plain text

### `chunking.rs`
Utility module for chunking large text documents (not currently in use)

## Configuration

The common module handles centralized configuration loading:

- **Configuration Sources**: Loads from `.env` files, JSON configuration files, and environment variables
- **Platform-Specific Paths**: Uses the `directories` crate to locate config files in OS-appropriate locations

Configuration files are typically located in:
- Linux/macOS: `~/.config/rustbot/`
- Windows: `%APPDATA%\rustbot\`

## Integration

The common module is integrated throughout Puetce