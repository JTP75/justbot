use std::{fs, path::Path};

use crate::{common::{config, config_const::{json::{ANTHROPIC_CONFIG, EMBEDDING_CONFIG, VECTORDB_CONFIG}, keys::{DEFAULT_SEARCH_LIMIT, ENABLE_ANTHROPIC, ENABLE_LOCAL, ENABLE_QDRANT, ENABLE_VOYAGE}}}, connection::{EmbeddingClient, anthropic_client::AnthropicClient, local_embedding_client::LocalClient, qdrant_client::QdrantClient, voyage_client::VoyageClient}};

#[derive(Debug)]
pub struct ConnectionManager {
    pub chat_client: Option<AnthropicClient>,
    pub vdb_client: Option<QdrantClient>,
    pub embedding_client: Option<Box<dyn EmbeddingClient>>,
}

impl ConnectionManager {

    pub fn new() -> Self {
        let use_anthropic: bool = config
            ::get_config(ANTHROPIC_CONFIG, ENABLE_ANTHROPIC)
            .unwrap_or(false);
        let use_voyage: bool = config
            ::get_config(EMBEDDING_CONFIG, ENABLE_VOYAGE)
            .unwrap_or(false);
        let use_local: bool = config
            ::get_config(EMBEDDING_CONFIG, ENABLE_LOCAL)
            .unwrap_or(false);
        let use_qdrant: bool = config
            ::get_config(VECTORDB_CONFIG, ENABLE_QDRANT)
            .unwrap_or(false);

        let chat_client = if use_anthropic {
            Some(AnthropicClient::new().unwrap())
        } else {
            None
        };
        if (use_voyage || use_local) && use_qdrant {
            if use_voyage {
                log::info!("Using qdrant database and voyage embedding");
            } else if use_local {
                log::info!("Using qdrant database and local embedding");
            }

            let embedding_client: Option<Box<dyn EmbeddingClient>> = if use_voyage {
                Some(Box::new(VoyageClient::new().unwrap()))
            } else if use_local {
                Some(Box::new(LocalClient::new().unwrap()))
            } else {
                log::error!("Invalid embedding selection (unreachable)");
                unreachable!()
            };
            let vdb_client = Some(QdrantClient::new().unwrap());

            Self { chat_client, vdb_client, embedding_client }
        } else {
            if use_qdrant {
                log::warn!("No embedding is enabled; skipping vector database");
            } else if use_voyage || use_local {
                log::warn!("No vector database is enabled; skipping embedding");
            } else {
                log::warn!("Vector database and embedding are disabled");
            }

            let embedding_client = None;
            let vdb_client = None;

            Self { chat_client, vdb_client, embedding_client }
        }
    }

    // routines
    
    /// Gets the embedding for a file and stores to the Vector DB
    /// 
    /// - assumes `file_path` is valid
    /// - attempts to convert pdfs to text
    pub async fn embed_file(&self, collection_name: &str, file_path: &Path) 
    -> Result<(), Box<dyn std::error::Error>> {
        if self.embedding_client.is_some() && self.vdb_client.is_some() {
            let path_str = match file_path.to_str() {
                Some(s) => s,
                None => { return Err(format!("Error converting path <{}> to &str", file_path.display()).into()) }
            };
            let content = match file_path.extension().and_then(|ext| ext.to_str())  {
                Some("pdf") => crate::common::pdf
                    ::extract_pdf_text(&file_path)?,
                _ => fs::read_to_string(&file_path)
                    .map_err(|_| format!("Failed to read file {}", file_path.display()))?
            };

            // embed content and path
            //      fixme theres a better way to group embeddings...
            let text_data = format!("{{\"file_path\": \"{}\", \"content\": \"{}\"}}", path_str, content);
            let embedding = self.embedding_client.as_ref().unwrap().get_embedding(&text_data, "document").await?;

            // store content to vdb
            // (make a new collection if it doesnt exist)
            if !self.vdb_client.as_ref().unwrap().list_collections().await?.contains(&collection_name.to_string()) {
                self.vdb_client.as_ref().unwrap().add_collection(collection_name).await?;
            }
            self.vdb_client.as_ref().unwrap().insert_to_collection(collection_name, embedding, path_str, &content).await?;

            Ok(())
        } else {
            log::error!("Cannot embed file; vdb and/or embedding is disabled");
            Err("Cannot embed file; vdb and/or embedding is disabled".into())
        }
    }

    /// Gets the embedding for multiple files and stores to the Vector DB
    /// 
    /// - assumes each path in `file_path` is valid
    /// - attempts to convert pdfs to text
    pub async fn embed_files(&self, collection_name: &str, file_paths: Vec<&Path>) 
    -> Result<(), Box<dyn std::error::Error>> {
        if self.embedding_client.is_some() && self.vdb_client.is_some() {

            let mut texts = Vec::new();
            let mut contents = Vec::new();

            for file_path in file_paths.iter() {
                let path_str = match file_path.to_str() {
                    Some(s) => s,
                    None => { return Err(format!("Error converting path <{}> to &str", file_path.display()).into()) }
                };
                let content = match file_path.extension().and_then(|ext| ext.to_str())  {
                    Some("pdf") => crate::common::pdf
                        ::extract_pdf_text(&file_path)?,
                    _ => fs::read_to_string(&file_path)
                        .map_err(|_| format!("Failed to read file {}", file_path.display()))?
                };
                let text_data = format!("{{\"file_path\": \"{}\", \"content\": \"{}\"}}", path_str, content);

                contents.push(content.clone());
                texts.push(text_data);
            }

            let texts: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
            let contents: Vec<&str> = contents.iter().map(|s| s.as_str()).collect();
            let file_paths: Vec<&str> = file_paths.iter()
                .map(|p| p.to_str().unwrap_or(""))
                .collect();

            let embeddings = self.embedding_client.as_ref().unwrap().get_embeddings(texts, "document").await?;

            // (make a new collection if it doesnt exist)
            if !self.vdb_client.as_ref().unwrap().list_collections().await?.contains(&collection_name.to_string()) {
                self.vdb_client.as_ref().unwrap().add_collection(collection_name).await?;
            }
            self.vdb_client.as_ref().unwrap().insert_multiple_to_collection(
                collection_name, 
                embeddings, 
                file_paths, 
                contents
            ).await?;

            Ok(())
        } else {
            log::error!("Cannot embed files; vdb and/or embedding is disabled");
            Err("Cannot embed files; vdb and/or embedding is disabled".into())
        }
    }
    
    /// Searches the Vector DB with the user's query
    /// 
    /// - returns json result containing a list of the top `n` relevant files
    ///     - where `n` is the search_limit set in vectordb_config.json
    /// - each entry contains the file path, file contents, and search score value
    pub async fn query_vdb(&self, collection_name: &str, query: &str, search_limit: Option<u64>) 
    -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        if self.embedding_client.is_some() && self.vdb_client.is_some() {

            // vectorize query
            let qvec = self.embedding_client.as_ref().unwrap().get_embedding(query, "query").await?;

            let config_limit = crate::common::config
                ::get_config(VECTORDB_CONFIG, DEFAULT_SEARCH_LIMIT)?;
            let limit = if let Some(limit) = search_limit && limit < config_limit {
                limit
            } else {
                config_limit
            };

            // search db
            let search_result = self.vdb_client.as_ref().unwrap().search_collection(collection_name, qvec, limit).await?;

            // build context (json)
            Ok(serde_json::json!(
                search_result.iter().map(|sp| {
                    serde_json::json!({
                        "file_path": sp.payload
                            .get("file_path"),
                        "content": sp.payload
                            .get("content"),
                        "score": sp.score,
                    })
                }).collect::<Vec<_>>()
            ))
        } else {
            log::error!("Cannot query vdb; vdb and/or embedding is disabled");
            Err("Cannot query vdb; vdb and/or embedding is disabled".into())
        }
    }
}