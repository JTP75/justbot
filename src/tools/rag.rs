use crate::{app::puetce::PuetceApp, tools::{ToolInputSchema, ToolInputSchemaBuilder}};

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
    fn exec(&self, bot: &mut PuetceApp, args: &serde_json::Value) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let query = args.get("query").ok_or("Arguments are missing a paramater 'query'")?
            .as_str().ok_or("Unexpected argument type for parameter 'query'")?;
        let count = args.get("number_of_documents").ok_or("Arguments are missing a paramater 'number_of_documents'")?
            .as_u64().ok_or("Unexpected argument type for parameter 'number_of_documents'")?;

        let default_collection = crate::common::config
            ::get_config("bot_config.json", "default_collection")?;
        let results = bot.retrieve_from_vdb(&bot.get_current_collection().unwrap_or(default_collection), query, Some(count))?;

        Ok(Some(results))
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "custom-rag-text".into(), 
        || Box::new(RagTool),
    );
}