// #![allow(unused)]

use std::hash::{DefaultHasher, Hasher};

use qdrant_client::{
    Payload, Qdrant, qdrant::{
        CreateCollection, Distance, PointStruct, ScoredPoint, SearchPoints, UpsertPointsBuilder, VectorParams, VectorsConfig, vectors_config
    }
};

pub struct QdrantClient {
    client: Qdrant,
    _url: String,
}

impl std::fmt::Debug for QdrantClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QdrantClient")
            .field("client", &"<Qdrant client>")
            .field("_url", &self._url)
            .finish()
    }
}

impl QdrantClient {

    /// Create a new gRPC `QdrantClient` instance 
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host: String = crate::common::config
            ::get_config("vectordb_config.json", "host")?;
        let port: u16 = crate::common::config
            ::get_config("vectordb_config.json", "grpc_port")?;

        let url = format!("http://{}:{}/collections", host, port);
        Ok(Self {
            client: Qdrant::from_url(&url).skip_compatibility_check().build()?,
            _url: url,
        })
    }

    /// Create a new collection in the VectorDB
    pub async fn add_collection(&self, collection_name: &str) 
    -> Result<(), Box<dyn std::error::Error>> {
        let req = CreateCollection {
            collection_name: collection_name.into(),
            vectors_config: Some(VectorsConfig { 
                config: Some(vectors_config::Config::Params(VectorParams {
                    size: crate::common::config
                        ::get_config::<u64>("vectordb_config.json", "dimensionality")?,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })), 
            }),
            ..Default::default()
        };

        // 5 retries
        for attempt in 1..6 {
            let response = self.client.create_collection(req.clone()).await;
            if let Ok(_response) = response {
                return Ok(());
            } else {
                log::warn!(
                    "Qdrant request failed (attempt {}/5): {}", 
                    attempt, 
                    response.unwrap_err()
                );
            }
        }
        Err("All create collection attempts failed.".into())
    }

    /// List all available collections in the Vector DB
    pub async fn list_collections(&self)
    -> Result<Vec<String>, Box<dyn std::error::Error>> {
        
        // 5 retries
        for attempt in 1..6 {
            let response = self.client.list_collections().await;
            if let Ok(response) = response {
                let list = response.collections.iter()
                    .map(|c| c.name.clone())
                    .collect::<Vec<_>>();
                return Ok(list);
            } else {
                log::warn!(
                    "Qdrant request failed (attempt {}/5): {}", 
                    attempt, 
                    response.unwrap_err()
                );
            }
        }
        Err("All list collections attempts failed.".into())
    }

    pub async fn insert_to_collection(&self, collection_name: &str, 
        vectors: Vec<Vec<f32>>, file_paths: Vec<&str>, contents: Vec<&str>)
    -> Result<Vec<u64>, Box<dyn std::error::Error>> {
        
        let mut points = Vec::new();
        let mut ids = Vec::new();

        for (i,vector) in vectors.into_iter().enumerate() {
            let payload: Payload = serde_json::json!({
                "file_path": file_paths[i],
                "content": contents[i],
            }).try_into()?;

            let mut hasher = DefaultHasher::new();
            hasher.write(file_paths[i].as_bytes());
            let id = hasher.finish();

            points.push(PointStruct::new(id.clone(), vector, payload));
            ids.push(id);
        }

        let req = UpsertPointsBuilder::new(collection_name, points);
        self.client.upsert_points(req).await?;
        
        Ok(ids)
    }

    // todo add a separate insert fn for large vector sets

    /// Search a collection in the Vector DB using a query vector
    /// 
    /// - `limit` is the max number of results to return
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
        let result = client.add_collection(collection_name).await;
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
            vec![vec![0.72; DIM]], vec!["project/README.md"], vec!["# README hello"]).await;
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
}