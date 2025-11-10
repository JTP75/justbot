use crate::{app::bot::PuetceApp, tools::{ToolInputSchema, ToolInputSchemaBuilder}};

use super::{Tool, REGISTRY};

pub struct GetGreetingTool;

impl Tool for GetGreetingTool {
    fn name(&self) -> &str { "custom-misc-get_greeting" }
    fn description(&self) -> &str { "Retrieves a friendly greeting" }
    fn input_schema(&self) -> ToolInputSchema { 
        ToolInputSchemaBuilder::default()
            .optional_property("name", "string", "Your name, if you have one")
            .build()
            .expect("Tool schema builder failed")
    }
    fn exec(&self, _bot: &mut PuetceApp, args: &serde_json::Value) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let Some(name) = args.as_object().unwrap_or(&serde_json::Map::new()).get("name") {
            Ok(Some(format!("Hello there! My name is {}.", name.as_str().unwrap())))
        } else {
            Ok(Some(format!("Hello there! This is a friendly greeting!")))
        }
    }
}

#[ctor::ctor]
fn register() {
    REGISTRY.lock().unwrap().register(
        "custom-misc-get_greeting".into(), 
        || Box::new(GetGreetingTool),
    );
}