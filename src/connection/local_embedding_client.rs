use reqwest::{Client, ClientBuilder};

use crate::connection::EmbeddingClient;

#[derive(Debug)]
pub struct LocalClient {
    client: Client,
    url: String,
}

impl LocalClient {
    /// Create a new instance of `LocalClient`
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host: String = crate::common::config
            ::get_config("vectordb_config.json", "host")?;
        let port: u64 = crate::common::config
            ::get_config("vectordb_config.json", "embedding_port")?;
        let url = format!("http://{host}:{port}/embed");
        Ok(Self { client: ClientBuilder::default().build()?, url })
    }
}

#[async_trait::async_trait]
impl EmbeddingClient for LocalClient {
    async fn get_embeddings(&self, texts: Vec<&str>, input_type: &str) 
    -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&self.url)
            .json(&serde_json::json!({
                "texts": texts,
                "type": input_type
            }))
            .send().await?
            .json::<serde_json::Value>().await?;

        let result: Vec<Vec<f32>> = response["embeddings"]
            .as_array().ok_or("embeddings not found in response")?
            .iter().map(|val| val.as_array().expect("unexpected embedding type")
                .iter().map(|val| val.as_f64().unwrap_or(0.0) as f32)
                .collect::<Vec<f32>>()
            ).collect();
        
        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// docker container with embedding service must be running
    #[tokio::test]
    async fn test_embed_some_text() {
        let client: Box<dyn EmbeddingClient> = Box::new(LocalClient::new().unwrap());

        let embedding = client.get_embeddings(vec!["Hello this is some text"], "document").await;
        assert!(embedding.is_ok());
        assert_eq!(embedding.unwrap().len(), 768);
    }

    /// docker container with embedding service must be running
    #[tokio::test]
    async fn test_embed_some_texts() {
        let client: Box<dyn EmbeddingClient> = Box::new(LocalClient::new().unwrap());

        let embeddings = client.get_embeddings(vec!["Hello this is some text"; 1024], "document").await;
        assert!(embeddings.is_ok());
    }
}