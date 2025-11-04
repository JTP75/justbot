// #![allow(unused)]

use directories::ProjectDirs;
use reqwest::{Client, ClientBuilder};

use crate::common::config::{APPLICATION, ORGANIZATION, QUALIFIER};

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
        let project_dirs = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).unwrap();
        let env_path = project_dirs.config_dir().join(".env");
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

    /// Get the embedding for input text
    /// 
    /// - `input_text` can be either "document" or "query"
    ///     - use "document" to embed the contents of a file
    ///     - use "query" to get the query vector for a search query
    pub async fn get_embedding(&self, text: &str, input_type: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
        let response = self.client
            .post(&self.url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&serde_json::json!({
                "input": [text],
                "model": self.model,
                "input_type": input_type,
            }))
            .send().await?
            .json::<serde_json::Value>().await?;

        let result = response["data"][0]["embedding"].as_array();
        match result {
            Some(arr) => Ok(arr.iter().map(|v| v.as_f64().unwrap_or(0.0) as f32).collect()),
            None => {
                log::error!("Voyage API failed: {}", response);
                Err("Embedding is null, voyage api request probably failed".into())
            }
        }
    }

    pub async fn get_embeddings(&self, texts: Vec<&str>, input_type: &str) -> Result<Vec<Vec<f32>>, Box<dyn std::error::Error>> {
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
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embed_a_string() {
        let client = VoyageClient::new().unwrap();

        let result = client.get_embedding("We want to embed this text!", "document").await;
        assert!(result.is_ok());

        println!("{:?}", result.unwrap())
    }
}