use chrono::{Duration, Local};

use crate::{rustbot::bot::RustBot, tools::{ToolInputSchema, ToolInputSchemaBuilder}};

use super::{Tool, REGISTRY};

pub struct GetEventsTool;


impl Tool for GetEventsTool {
    fn name(&self) -> &str { "custom-calendar-get_events" }
    fn description(&self) -> &str { "Retrieves upcoming events from the users calendar. This will retrive all events between (now) and (now + ndays)." }
    fn input_schema(&self) -> ToolInputSchema { 
        ToolInputSchemaBuilder::default()
            .property("ndays", "number", "The number of days from now to retrieve events for. This is a positive integer greater than zero.")
            .build()
            .unwrap()
    }
    fn exec(&self, bot: &mut RustBot, args: &serde_json::Value) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let count = args.get("ndays").ok_or("Arguments are missing a paramater 'ndays'")?
            .as_i64().ok_or("Unexpected argument type for parameter 'ndays'")?;

        let start = Local::now();
        let until = start + Duration::days(count);

        let events = bot.get_calendar_events_range("primary", start, until)?;
        let json_str = serde_json::to_string_pretty(&events)?;

        Ok(Some(json_str))
    }
}

#[ctor::ctor]
fn register() {
    let mut r = REGISTRY.lock().unwrap();
    r.register(
        "custom-calendar-get_events".into(), 
        || Box::new(GetEventsTool),
    );
}