//! This is for non-mcp tools

use std::collections::HashMap;
use derive_builder::Builder;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{connection::anthropic_client::ToolDefinition, rustbot::bot::RustBot};

pub trait Tool {
    // templates

    /// Returns the formal name of the tool
    fn name(&self) -> &str;

    /// Returns a short (and AI readable) description for the tool
    fn description(&self) -> &str;

    /// Returns a tool input schema the input schema for the tool
    fn input_schema(&self) -> ToolInputSchema;

    /// Executes the tool
    /// 
    /// Returns response wrapped in a Result and Option
    fn exec(&self, bot: &mut RustBot, args: &serde_json::Value) -> Result<Option<String>, Box<dyn std::error::Error>>;

    // impls

    /// Converts a struct implementing Tool into a ToolDefinition struct
    fn as_tooldef(&self) -> ToolDefinition {
        ToolDefinition { 
            name: self.name().into(), 
            description: self.description().into(), 
            input_schema: serde_json::json!(self.input_schema())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Builder)]
#[serde(rename_all = "snake_case")]
#[builder(build_fn(validate = "Self::validate_required"))]
pub struct ToolInputSchema {
    #[builder(default = "object".into())]
    r#type: String,
    #[builder(default = HashMap::new(), setter(custom))]
    properties: HashMap<String, ToolInputProperty>,
    #[builder(default = Vec::new())]
    required: Vec<String>,
    #[builder(default = false)]
    additional_properties: bool,
}

#[allow(unused)]
impl ToolInputSchemaBuilder {
    fn validate_required(&self) -> Result<(), String> {
        if let Some(req) = &self.required {
            for key in req.iter() {
                if !self.properties.as_ref()
                    .ok_or("Properties empty")?
                    .contains_key(key) {
                        return Err(format!("Properties has no key '{}'", key));
                    }
            }
        }
        Ok(())
    }
    pub fn property(&mut self, title: &str, r#type: &str, desc: &str) 
    -> &mut Self {
        self.properties
            .get_or_insert_with(HashMap::new)
            .insert(title.into(), ToolInputProperty {
                r#type: r#type.into(),
                title: title.into(),
                description: desc.into(),
                r#enum: Vec::new(),
            });
        self.required
            .get_or_insert_with(Vec::new)
            .push(title.into());
        self
    }
    pub fn optional_property(&mut self, title: &str, r#type: &str, desc: &str) 
    -> &mut Self {
        self.properties
            .get_or_insert_with(HashMap::new)
            .insert(title.into(), ToolInputProperty {
                r#type: r#type.into(),
                title: title.into(),
                description: desc.into(),
                r#enum: Vec::new(),
            });
        self
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Builder)]
#[serde(rename_all = "snake_case")]
pub struct ToolInputProperty {
    r#type: String,
    title: String,
    description: String,
    #[builder(default = Vec::new())]
    r#enum: Vec<String>
}

pub type ToolFactory = fn() -> Box<dyn Tool>;

pub struct ToolRegistry {
    tools: HashMap<String, ToolFactory>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }
    
    pub fn register(&mut self, name: String, factory: ToolFactory) {
        self.tools.insert(name.clone(), factory);
    }

    pub fn _get(&self, name: &str) -> Option<Box<dyn Tool>> {
        self.tools.get(name).map(|factory| factory())
    }

    pub fn tools(&self) -> Vec<Box<dyn Tool>> {
        self.tools.values().map(|factory| factory()).collect()
    }
}

pub static REGISTRY: Lazy<std::sync::Mutex<ToolRegistry>> = 
    Lazy::new(|| std::sync::Mutex::new(ToolRegistry::new()));

// register tools

pub mod get_greeting;