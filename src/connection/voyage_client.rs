// #![allow(unused)]

use directories::ProjectDirs;
use reqwest::{Client, ClientBuilder};

#[derive(Debug)]
pub struct VoyageClient {
    client: Client,
    model: String,
    url: String,
    api_key: String,
}

impl VoyageClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let project_dirs = ProjectDirs::from("com", "Justin Inc.", "rustbot").unwrap();
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

        // fixme this will panic
        let embedding = response["data"][0]["embedding"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();

        Ok(embedding)
    }
}

// curl https://api.voyageai.com/v1/embeddings \
//   -H "Content-Type: application/json" \
//   -H "Authorization: Bearer $VOYAGE_API_KEY" \
//   -d '{
//     "input": "Sample text",
//     "model": "voyage-3.5",
//     "input_type": "document"
//   }'

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