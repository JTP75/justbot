# Common Module

The common module provides shared utilities and configurations used throughout RustBot.

## Modules

### `config.rs`
Configuration management and loading from JSON files and environment variables

### `pdf.rs` 
Utility module for converting PDFs to plain text

## Configuration

The common module handles centralized configuration loading:

- **Configuration Sources**: Loads from `.env` files, JSON configuration files, and environment variables
- **Platform-Specific Paths**: Uses the `directories` crate to locate config files in OS-appropriate locations

Configuration files are typically located in:
- Linux/macOS: `~/.config/rustbot/`
- Windows: `%APPDATA%\rustbot\`

## Integration

The common module is integrated throughout RustBot. Used in almost every module

## Note

The PDF module is still highly experimental. It requires the apt package `poppler-utils` to be installed.

**This feature will most likely not work outside linux**