use std::{fs, path::Path};

use crate::{common::config, connection::{EmbeddingClient, anthropic_client::{AnthropicClient}, local_embedding_client::LocalClient, qdrant_client::QdrantClient, voyage_client::VoyageClient}};

#[derive(Debug)]
pub struct ConnectionManager {
    pub chat_client: AnthropicClient,
    pub vdb_client: QdrantClient,
    pub embedding_client: Box<dyn EmbeddingClient>,
}

impl ConnectionManager {

    pub fn new() -> Self {
        let use_voyage: bool = config
            ::get_config("vectordb_config.json", "use_voyage_embedding")
            .expect("get config failed");
        let use_local: bool = config
            ::get_config("vectordb_config.json", "use_local_embedding")
            .expect("get config failed");
        
        let embedding_client: Box<dyn EmbeddingClient> = if use_voyage {
            Box::new(VoyageClient::new().unwrap())
        } else if use_local {
            Box::new(LocalClient::new().unwrap())
        } else {
            log::error!("Must select an embedding method");
            unreachable!()
        };

        Self {
            chat_client: AnthropicClient::new().unwrap(),
            vdb_client: QdrantClient::new().unwrap(),
            embedding_client
        }
    }

    // routines
    
    /// Gets the embedding for a file and stores to the Vector DB
    /// 
    /// - assumes `file_path` is valid
    /// - attempts to convert pdfs to text
    pub async fn embed_file(&self, collection_name: &str, file_path: &Path) 
    -> Result<(), Box<dyn std::error::Error>> {
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
        let embedding = self.embedding_client.get_embedding(&text_data, "document").await?;

        // store content to vdb
        // (make a new collection if it doesnt exist)
        if !self.vdb_client.list_collections().await?.contains(&collection_name.to_string()) {
            self.vdb_client.add_collection(collection_name).await?;
        }
        self.vdb_client.insert_to_collection(collection_name, embedding, path_str, &content).await?;

        Ok(())
    }

    /// Gets the embedding for multiple files and stores to the Vector DB
    /// 
    /// - assumes each path in `file_path` is valid
    /// - attempts to convert pdfs to text
    pub async fn embed_files(&self, collection_name: &str, file_paths: Vec<&Path>) 
    -> Result<(), Box<dyn std::error::Error>> {

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

        let embeddings = self.embedding_client.get_embeddings(texts, "document").await?;

        // (make a new collection if it doesnt exist)
        if !self.vdb_client.list_collections().await?.contains(&collection_name.to_string()) {
            self.vdb_client.add_collection(collection_name).await?;
        }
        self.vdb_client.insert_multiple_to_collection(
            collection_name, 
            embeddings, 
            file_paths, 
            contents
        ).await?;

        Ok(())
    }
    
    /// Searches the Vector DB with the user's query
    /// 
    /// - returns json result containing a list of the top `n` relevant files
    ///     - where `n` is the search_limit set in vectordb_config.json
    /// - each entry contains the file path, file contents, and search score value
    pub async fn query_vdb(&self, collection_name: &str, query: &str, search_limit: Option<u64>) 
    -> Result<serde_json::Value, Box<dyn std::error::Error>> {

        // vectorize query
        let qvec = self.embedding_client.get_embedding(query, "query").await?;

        let config_limit = crate::common::config
            ::get_config("vectordb_config.json", "default_search_limit")?;
        let limit = if let Some(limit) = search_limit && limit < config_limit {
            limit
        } else {
            config_limit
        };

        // search db
        let search_result = self.vdb_client.search_collection(collection_name, qvec, limit).await?;

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
    }
}