//#![allow(unused)]

/// config json files
pub mod json {
    pub const BOT_CONFIG: &str = "bot_config.json";

    pub const ANTHROPIC_CONFIG: &str = "anthropic_config.json";
    pub const VECTORDB_CONFIG: &str = "vectordb_config.json";
    pub const EMBEDDING_CONFIG: &str = "embedding_config.json";

    pub const MCP_SERVERS_CONFIG: &str = "mcp_servers.json";
    
    pub const MOTD_FILENAME: &str = "motd.json";
}

/// config keys
pub mod keys {
    pub const HOST: &str = "host";
    pub const PORT: &str = "port";

    pub const START_DIR: &str = "start_dir";
    pub const DEFAULT_NAME: &str = "default_name";
    pub const CLIENT_HOST: &str = "client_host";
    pub const ENABLE_MCP: &str = "enable_mcp_servers";
    pub const ENABLE_CUSTOM_TOOLS: &str = "enable_custom_tools";

    pub const ENABLE_ANTHROPIC: &str = "enable_anthropic";
    pub const BASE_URL: &str = "base_url";
    pub const ANTHROPIC_VERSION: &str = "anthropic_version";
    pub const MAX_TOKENS: &str = "max_tokens";
    pub const DEFAULT_MODEL: &str = "default_model";
    pub const MAX_INPUT_TPM: &str = "max_input_tpm";
    pub const MAX_OUTPUT_TPM: &str = "max_output_tpm";

    pub const ENABLE_QDRANT: &str = "enable_qdrant";
    pub const REST_PORT: &str = "rest_port";
    pub const GRPC_PORT: &str = "grpc_port";
    pub const DEFAULT_COLLECTION: &str = "default_collection";
    pub const DIM: &str = "dimensionality";
    pub const DEFAULT_SEARCH_LIMIT: &str = "default_search_limit";

    pub const ENABLE_PDF_EMBEDDING: &str = "allow_pdfs";
    pub const ENABLE_VOYAGE: &str = "use_voyage_embedding";
    pub const ENABLE_LOCAL: &str = "use_local_embedding";
    pub const VOYAGE_URL: &str = "embedding_url";
    pub const VOYAGE_MODEL: &str = "embedding_model";
}

/// prompts
pub mod prompts {
    pub const PROMPTS_DIR: &str = "prompts";

    pub const SYSTEM_BASE: &str = "base_sys_prompt.txt";
    pub const SYSTEM_RAG: &str = "rag_sys_prompt.txt";
    pub const SYSTEM_TOOL: &str = "tool_sys_prompt.txt";
}