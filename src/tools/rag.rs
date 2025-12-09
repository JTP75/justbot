use crate::{app::{connection_manager::ConnectionManager, http::APP_STATE}, connection::anthropic_client::{Source, ToolResultContentBlock}, tools::{ToolInputSchema, ToolInputSchemaBuilder}};

use super::{Tool, REGISTRY};

pub struct RagTool;

impl Tool for RagTool {
    fn name(&self) -> &str { "custom-rag-text" }
    fn description(&self) -> &str { "This is a RAG tool that retrieves text documents from a vector database using semantic search. It returns text data and a score for each retrieved document." }
    fn input_schema(&self) -> ToolInputSchema { 
        ToolInputSchemaBuilder::default()
            .property("query", "string", "The query for searching the vector database")
            .property("number_of_documents", "number", "This is the positive integer max number of documents to retrieve. The value should be at least 1 and at most 10.")
            .build()
            .expect("Tool schema builder failed")
    }
    fn exec(&self, cm: &ConnectionManager, args: &serde_json::Value) -> Result<Vec<ToolResultContentBlock>, Box<dyn std::error::Error>> {
        let query = args.get("query").ok_or("Arguments are missing a paramater 'query'")?
            .as_str().ok_or("Unexpected argument type for parameter 'query'")?;
        let count = args.get("number_of_documents").ok_or("Arguments are missing a paramater 'number_of_documents'")?
            .as_u64().ok_or("Unexpected argument type for parameter 'number_of_documents'")?;

        let default_collection = crate::common::config
            ::get_config::<String>("bot_config.json", "default_collection")?;
        let as_cname = &APP_STATE.lock().unwrap().collection_name;
        let collection_name = if let Some(collection_name) = as_cname { 
            collection_name.as_str()
        } else { 
            default_collection.as_str()
        };

        let json_results = tokio::task::block_in_place(|| {
            tokio::runtime::Runtime::new()?
                .block_on(cm.query_vdb(collection_name, query, Some(count)))
        })?;

        let content: Vec<ToolResultContentBlock> = json_results
            .as_array().ok_or("Unexpected JSON type from query_vdb")?
            .iter().map(|document| ToolResultContentBlock::Document { 
                source: Source::Text { 
                    media_type: "text/plain".to_string(), 
                    data: serde_json::to_string(&document.get("content"))
                        .unwrap_or("The content of this document is blank".to_string())
                }, 
                title: serde_json::to_string(
                    &serde_json::json!(document.get("file_path"))).ok(), 
                context: serde_json::to_string(
                    &serde_json::json!({
                        "file_path": document.get("file_path"),
                        "score": document.get("score"),
                    })).ok()
            }).collect();

        Ok(content)
    }
}

// class DocumentBlockParam(TypedDict, total=False):
//     source: Required[Source]

//     type: Required[Literal["document"]]

//     cache_control: Optional[CacheControlEphemeralParam]
//     """Create a cache control breakpoint at this content block."""

//     citations: Optional[CitationsConfigParam]

//     context: Optional[str]

//     title: Optional[str]

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "custom-rag-text".into(), 
        || Box::new(RagTool),
    );
}