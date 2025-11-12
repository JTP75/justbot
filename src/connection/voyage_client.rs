// #![allow(unused)]

use reqwest::{Client, ClientBuilder};

use crate::{common::config, connection::EmbeddingClient};

#[derive(Debug)]
pub struct VoyageClient {
    client: Client,
    model: String,
    url: String,
    api_key: String,
}

impl VoyageClient {
    /// Create a new instance of `VoyageClient`
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let env_path = config::PROJECT_DIRS.config_dir().join(".env");
        dotenvy::from_path(env_path).ok();

        let api_key = std::env::var("VOYAGE_API_KEY")?;
        Ok(Self {
            client: ClientBuilder::default().build()?,
            model: crate::common::config
                ::get_config("vectordb_config.json", "embedding_model")?,
            url: crate::common::config
                ::get_config("vectordb_config.json", "embedding_url")?,
            api_key: api_key,
        })
    }
}

#[async_trait::async_trait]
impl EmbeddingClient for VoyageClient {
    async fn get_embeddings(&self, texts: Vec<&str>, input_type: &str) 
    -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "input": texts,
                "model": self.model,
                "input_type": input_type,
            }))
            .send().await?
            .json::<serde_json::Value>().await?;

        let data = response["data"].as_array()
            .ok_or("Null response from voyage api")?;

        let mut embeddings: Vec<Vec<f32>> = Vec::new();
        for value in data.iter() {
            let embedding = value["embedding"].as_array()
                .ok_or("Missing embedding in response")?
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                .collect();
            embeddings.push(embedding)
        }

        if texts.len()==embeddings.len() {
            Ok(embeddings)
        } else {
            Err("Embeddings missing from response".into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// docker container with embedding service must be running
    #[tokio::test]
    async fn test_embed_some_text() {
        let client: Box<dyn EmbeddingClient> = Box::new(VoyageClient::new().unwrap());

        let embedding = client.get_embeddings(vec!["Hello this is some text"], "document").await;
        assert!(embedding.is_ok());
        assert_eq!(embedding.unwrap().len(), 768);
    }

    /// docker container with embedding service must be running
    #[tokio::test]
    async fn test_embed_some_texts() {
        let client: Box<dyn EmbeddingClient> = Box::new(VoyageClient::new().unwrap());

        let embeddings = client.get_embeddings(vec!["Hello this is some text"; 1024], "document").await;
        assert!(embeddings.is_ok());
    }
}