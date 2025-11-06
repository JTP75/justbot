use std::fs;

use chrono::Local;
use directories::ProjectDirs;

use crate::{common::config::{APPLICATION, ORGANIZATION, QUALIFIER}, rustbot::bot::RustBot, tools::{ToolInputSchema, ToolInputSchemaBuilder}};

use super::{Tool, REGISTRY};

pub struct GetMotdTool;

pub struct SetMotdTool;

impl Tool for GetMotdTool {
    fn name(&self) -> &str { "custom-motd-get_motd" }
    fn description(&self) -> &str { "Retrieves today's message of the day from storage. If there is no stored MOTD or the MOTD is expired, this tool won't return anything." }
    fn input_schema(&self) -> ToolInputSchema { 
        ToolInputSchemaBuilder::default()
            .build()
            .unwrap()
    }
    fn exec(&self, bot: &mut RustBot, _args: &serde_json::Value) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if let (date, Some(message)) = bot.get_motd() {
            if date == Local::now().date_naive() {
                Ok(Some(message))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}

impl Tool for SetMotdTool {
    fn name(&self) -> &str { "custom-motd-set_motd" }
    fn description(&self) -> &str { "Sets and stores a new message of the day." }
    fn input_schema(&self) -> ToolInputSchema { 
        ToolInputSchemaBuilder::default()
            .property("new_motd", "string", "The new message of the day to be stored. This should always be one sentence. It should be creative, funny, interesting, or some combination of those things.")
            .build()
            .unwrap()
    }
    fn exec(&self, bot: &mut RustBot, args: &serde_json::Value) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let new_motd = args.get("new_motd").ok_or("Arguments are missing a paramater 'new_motd'")?
            .as_str().ok_or("Unexpected argument type for parameter 'new_motd'")?;

        bot.set_motd((Local::now().date_naive(), Some(new_motd.into())));

        let json = serde_json::to_string_pretty(&bot.get_motd())?;
        let filename: String = crate::common::config
            ::get_config("bot_config.json","motd_filename")?;
        let pd=ProjectDirs::from(QUALIFIER,ORGANIZATION,APPLICATION).unwrap();let data_dir = pd.data_dir();
        fs::write(data_dir.join(filename), json)?;

        Ok(None)
    }
}

#[ctor::ctor]
fn register() {
    let mut r = REGISTRY.lock().unwrap();
    r.register(
        "custom-misc-get_motd".into(), 
        || Box::new(GetMotdTool),
    );
    r.register(
        "custom-misc-set_motd".into(), 
        || Box::new(SetMotdTool),
    );
}