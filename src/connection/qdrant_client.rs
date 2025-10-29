#![allow(unused)]

use std::collections::HashMap;

use qdrant_client::{
    Payload, Qdrant, qdrant::{
        CreateCollection, Distance, PointStruct, ScoredPoint, SearchPoints, UpsertPointsBuilder, Value, VectorParams, VectorsConfig, vectors_config
    }
};
use uuid::Uuid;

const QDRANT_HOST: &str = "127.0.0.1";
const _QDRANT_REST_PORT: u16 = 6333;
const QDRANT_GRPC_PORT: u16 = 6334;

pub struct QdrantClient {
    client: Qdrant,
    url: String,
}

impl QdrantClient {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let url = format!("http://{}:{}/collections", QDRANT_HOST, QDRANT_GRPC_PORT);
        Ok(Self {
            client: Qdrant::from_url(&url).build()?,
            url: url,
        })
    }

    pub async fn add_collection(&self, collection_name: &str, size: usize, distance: Distance) 
    -> Result<(), Box<dyn std::error::Error>> {
        let req = CreateCollection {
            collection_name: collection_name.into(),
            vectors_config: Some(VectorsConfig { 
                config: Some(vectors_config::Config::Params(VectorParams {
                    size: 1536,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })), 
            }),
            ..Default::default()
        };
        self.client.create_collection(req).await?;
        Ok(())
    }

    pub async fn list_collections(&self)
    -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let collections = self.client.list_collections().await?;
        let list = collections.collections.iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>();
        Ok(list)
    }

    pub async fn insert_to_collection(&self, collection_name: &str, 
        vector: Vec<f32>, file_path: &str, content: &str)
    -> Result<(), Box<dyn std::error::Error>> {
        let payload: Payload = serde_json::json!({
            "file_path": file_path,
            "content": content,
        }).try_into()?;

        let id = Uuid::new_v4().to_string();
        let point = PointStruct::new(id, vector, payload);
        let req = UpsertPointsBuilder::new(collection_name, vec![point]);
        
        self.client.upsert_points(req).await?; // todo the chunking method is better for large sets
        
        Ok(())
    }

    pub async fn search_collection(&self, collection_name: &str, 
        query_vec: Vec<f32>, limit: u64) 
    -> Result<Vec<ScoredPoint>, Box<dyn std::error::Error>> {
        let req = SearchPoints {
            collection_name: collection_name.into(),
            vector: query_vec,
            limit,
            with_payload: Some(true.into()),
            ..Default::default()
        };

        let search_result = self.client.search_points(req).await?;

        Ok(search_result.result)
    }
}
 
#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;

    const TEST_COLLECTION_NAME: &str = "this_is_a_test_delete_me";
    const DIM: usize = 1536;

    #[tokio::test]
    async fn test_client_new_is_ok() {
        let client = QdrantClient::new();
        assert!(client.is_ok())
    }

    #[tokio::test]
    async fn test_add_and_list_collections() {
        let client = QdrantClient::new().unwrap();
        let collection_name = TEST_COLLECTION_NAME;

        // add collection
        let result = client.add_collection(collection_name, DIM, Distance::Cosine).await;
        match result {
            Ok(_) => println!("Created collection: {}", collection_name),
            Err(e) => eprintln!("Collection already exists (or a different error) {:?}", e),
        }

        // get existing collections
        let result = client.list_collections().await;
        match &result {
            Ok(collections) => println!("Existing collections: {:?}", collections),
            Err(e) => eprintln!("Setup failed: {:?}", e),
        }
        assert!(result.is_ok());
        
        // make sure new collection exists in db
        let list = result.unwrap();
        assert!(list.contains(&collection_name.to_string()));
    }

    #[tokio::test]
    async fn test_insert_and_search_in_collection() {
        let client = QdrantClient::new().unwrap();
        let collection_name = TEST_COLLECTION_NAME;

        // insert to collection
        let result = client.insert_to_collection(collection_name, 
            vec![0.72; DIM], "project/README.md", "# README hello").await;
        match result {
            Ok(_) => println!("Inserted to collection: {}", collection_name),
            Err(e) => eprintln!("Failed to insert to collection: {:?}", e),
        }

        // search collection
        let result = client.search_collection(collection_name, vec![0.72; DIM], 10).await;
        match &result {
            Ok(v) => println!("Search results: {:?}", v),
            Err(e) => eprintln!("Search failed: {:?}", e),
        }
        assert!(result.is_ok());

        // make sure new vector is found
    }

    #[tokio::test]
    async fn test_insert_many() {
        let client = QdrantClient::new().unwrap();
        let collection_name = TEST_COLLECTION_NAME;

        let mut rng = rand::rng();

        for i in 0..500 {
            let vec: Vec<f32> = (0..DIM).map(|_| rng.random_range(-1.0..1.0)).collect();
            let result = client.insert_to_collection(collection_name, 
                vec, "project/README.md", "# README hello").await;
            assert!(result.is_ok());
        }
    }
}