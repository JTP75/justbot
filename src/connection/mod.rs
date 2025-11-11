use std::fmt::Debug;

#[async_trait::async_trait]
pub trait EmbeddingClient: Debug {
    /// Get the embedding for input text
    /// 
    /// - `input_text` can be either "document" or "query"
    ///     - use "document" to embed the contents of a file
    ///     - use "query" to get the query vector for a search query
    async fn get_embedding(&self, text: &str, input_type: &str) 
    -> Result<Vec<f32>, Box<dyn std::error::Error>>;

    /// Get the embeddings for multiple input texts
    /// 
    /// - `input_text` can be either "document" or "query"
    ///     - use "document" to embed the contents of a file
    ///     - use "query" to get the query vector for a search query
    async fn get_embeddings(&self, texts: Vec<&str>, input_type: &str) 
    -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>>;
}

pub mod anthropic_client;
pub mod qdrant_client;
pub mod voyage_client;
pub mod local_embedding_client;
pub mod google_client;