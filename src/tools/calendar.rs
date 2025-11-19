#![allow(unused)]

use chrono::{Duration, Local};

use crate::{app::puetce::PuetceApp, tools::{ToolInputSchema, ToolInputSchemaBuilder}};

use super::{Tool, REGISTRY};

// pub struct CalendarDefaultEventTool;
// pub struct GetEventsTool;

// impl Tool for GetEventsTool {
//     fn name(&self) -> &str { "custom-calendar-get_events" }
//     fn description(&self) -> &str { "Retrieves upcoming events from the users calendar. This will retrive all events between (now) and (now + ndays)." }
//     fn input_schema(&self) -> ToolInputSchema { 
//         ToolInputSchemaBuilder::default()
//             .property("ndays", "number", "The number of days from now to retrieve events for. This is a positive integer greater than zero.")
//             .build()
//             .expect("Tool schema builder failed")
//     }
//     fn exec(&self, bot: &mut RustBot, args: &serde_json::Value) -> Result<Option<String>, Box<dyn std::error::Error>> {
//         let count = args.get("ndays").ok_or("Arguments are missing a paramater 'ndays'")?
//             .as_i64().ok_or("Unexpected argument type for parameter 'ndays'")?;

//         let start = Local::now();
//         let until = start + Duration::days(count);

//         let events = bot.get_calendar_events_range("primary", start, until)?;
//         let json_str = serde_json::to_string_pretty(&events)?;

//         Ok(Some(json_str))
//     }
// }

// impl Tool for CalendarDefaultEventTool {
//     fn name(&self) -> &str { "custom-calendar-default_event" }
//     fn description(&self) -> &str { "Gets instructions, info, and default behaviors for creating calendar events. These are instructions for what to put in event fields if the user does not specify." }
//     fn input_schema(&self) -> ToolInputSchema { 
//         ToolInputSchemaBuilder::default()
//             .build()
//             .unwrap()
//     }
//     fn exec(&self, _bot: &mut RustBot, _args: &serde_json::Value) -> Result<Option<String>, Box<dyn std::error::Error>> {
        
//         Ok(Some(
//             serde_json::to_string_pretty(&serde_json::json!({
//                 "general instructions": "These are default instructions for creating calendar events if the user does not specify certain fields. The goal is to make it so the user does not need to be specific when scheduling events. There are a few fields that are mandatory.",
//                 "mandatory fields": ["title", "date"],
//                 "useful info": [
//                     format!("The current datetime (rfc3339) is {}", Local::now().to_rfc3339()),
//                     format!("The user's current time zone is {}", Local::now().offset()),
//                 ],
//                 "default behaviors": {
//                     "start_time": "If the user does not specify a start_time, assume it is an all-day event.",
//                     "duration or end_time": "If the user does not specify a duration or end_time, default to 30 minutes after start_time.",
//                     "participants": "If there isnt enough information to determine participants, leave the field empty.",
//                     "location": "If the user does not specify a location, leave the field empty.",
//                     "reminders": "If the user does not specify reminders, set a default reminder for 10 minutes before the event.",
//                 }
//             }))?
//         ))
//     }
// }

#[ctor::ctor]
fn register() {
    let mut r = REGISTRY.lock().unwrap();
    // r.register(
    //     "custom-calendar-default_event".into(), 
    //     || Box::new(CalendarDefaultEventTool),
    // );
    // r.register(
    //     "custom-calendar-get_events".into(), 
    //     || Box::new(GetEventsTool),
    // );
}