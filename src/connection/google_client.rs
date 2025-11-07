#![allow(unused)]

use std::path::PathBuf;

use chrono::DateTime;
use reqwest::Client;
use yup_oauth2::{AccessToken, InstalledFlowAuthenticator, InstalledFlowReturnMethod};
use serde::{Deserialize, Serialize};
use derive_builder::Builder;

use crate::common::config;

// structs

#[derive(Debug, Deserialize, Serialize, Clone, Builder)]
struct EventDateTime {
    #[serde(rename = "dateTime")]
    date_time: Option<String>,
    date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Builder)]
pub struct CalendarEvent {
    id: String,
    #[builder(default = None)]
    summary: Option<String>,
    #[builder(default = None)]
    start: Option<EventDateTime>,
    #[builder(default = None)]
    end: Option<EventDateTime>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Builder)]
pub struct CalendarListResponse {
    #[builder(default = Vec::new())]
    items: Vec<CalendarEvent>,
}

#[derive(Debug, Clone)]
pub struct GoogleClient {
    client: Client,
    url: String,
    token: Option<AccessToken>,
    scopes: Vec<String>,
    credentials_path: PathBuf,
    token_path: PathBuf
}

impl EventDateTime {
    #[allow(unused)]
    fn as_ndt_opt(&self) -> Option<chrono::NaiveDateTime> {
        if let Some(date_time_str) = &self.date_time {
            match chrono::DateTime::parse_from_rfc3339(&date_time_str) {
                Ok(dt) => Some(dt.naive_local()),
                Err(_) => {
                    log::warn!("Failed to string '{}' to datetime", date_time_str);
                    None
                },
            }
        } else if let Some(date_str) = &self.date {
            match chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
                Ok(d) => d.and_hms_opt(0, 0, 0),
                Err(_) => {
                    log::warn!("Failed to string '{}' to datetime", date_str);
                    None
                },
            }
        } else {
            log::warn!("EventDateTime has no date or time");
            None
        }
    }
}

impl GoogleClient {
    pub fn new() -> Self {
        GoogleClient {
            client: Client::new(),
            url: "https://www.googleapis.com/calendar/v3/calendars".into(),
            token: None,
            scopes: vec!["https://www.googleapis.com/auth/calendar".into()],
            credentials_path: config::PROJECT_DIRS.config_dir().join("auth/gcp-oauth.keys.json"),
            token_path: config::PROJECT_DIRS.cache_dir().join("google_auth_token.json"),
        }
    }

    pub async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        log::info!(
            "Authenticating google client with paths:\n{:?}\n{:?}", 
            self.credentials_path, 
            self.token_path
        );

        if self.token.is_some() && !self.token.as_ref().unwrap().is_expired() {
            log::warn!("Already authenticated and token not expired");
            return Ok(());
        }

        let secret = yup_oauth2::read_application_secret(&self.credentials_path).await
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => format!(
                    "Credentials file not found at {:?}", 
                    self.credentials_path
                ), _ => format!("Failed to read credentials file: {}", e),
            })?;

        // prompt user for log in
        let auth = InstalledFlowAuthenticator::builder(secret, InstalledFlowReturnMethod::HTTPRedirect)
            .persist_tokens_to_disk(&self.token_path)
            .build().await?;
        self.token = Some(auth.token(&self.scopes).await.map_err(|e| 
            format!("Failed to log into google: {}", e)
        )?);
        
        Ok(())
    }
    
    pub async fn _retrieve_events(&mut self, calendar_id: impl Into<String>) -> Result<CalendarListResponse, Box<dyn std::error::Error>> {
        self.authenticate().await?;

        let url = format!(
            "{}/{}/events",
            self.url,
            calendar_id.into()
        );

        let token_str = self.token
            .as_ref().ok_or("No token")?
            .token().ok_or("No token")?;

        let response = self.client
            .get(&url)
            .bearer_auth(token_str)
            .query(&[
                ("orderBy", "startTime"),
                ("singleEvents", "true"),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            log::error!("Failed to retrieve events: {}", response.text().await.unwrap());
            return Err("Failed to retrieve events".into());
        }

        let calendar_list: CalendarListResponse = response.json().await.unwrap();

        Ok(calendar_list)
    }
    
    pub async fn retrieve_events_range<Tz>(
        &mut self, 
        calendar_id: impl Into<String>, 
        start: DateTime<Tz>,
        until: DateTime<Tz>
    ) 
    -> Result<Vec<CalendarEvent>, Box<dyn std::error::Error>> 
    where Tz: chrono::TimeZone + Send + Sync
    {
        self.authenticate().await?;

        let url = format!(
            "{}/{}/events",
            self.url,
            calendar_id.into()
        );

        let token_str = self.token
            .as_ref().ok_or("No token")?
            .token().ok_or("No token")?;

        let min_time = start.to_rfc3339();
        let max_time = until.to_rfc3339();
        let response = self.client
            .get(&url)
            .bearer_auth(token_str)
            .query(&[
                ("orderBy", "startTime"),
                ("singleEvents", "true"),
                ("timeMin", &min_time),
                ("timeMax", &max_time),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            log::error!("Failed to retrieve events: {}", response.text().await.unwrap());
            return Err("Failed to retrieve events".into());
        }

        let calendar_list: CalendarListResponse = response.json().await.unwrap();

        Ok(calendar_list.items)
    }
}

// Example EventDateTime JSON structures:

// "start": {
//     "date": "2025-10-10"
// },
// "end": {
//     "date": "2025-10-15"
// },

// "start": {
//     "dateTime": "2025-11-03T12:15:00-05:00",
//     "timeZone": "America/New_York"
// },
// "end": {
//     "dateTime": "2025-11-03T12:30:00-05:00",
//     "timeZone": "America/New_York"
// },

#[cfg(test)]
mod tests {
    use chrono::{Days, Local};

    use super::*;

    fn setup_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn test_retrieve_upcoming_events() {
        setup_logger();
        let mut client = GoogleClient::new();

        let events = client.retrieve_events_range(
            "primary", 
            Local::now(), 
            Local::now().checked_add_days(Days::new(7)).unwrap()
        ).await;

        assert!(events.is_ok(), "{}", events.err().unwrap());
        println!("{:#?}", events.unwrap());
    }

    #[tokio::test]
    async fn test_retrieve_events() {
        setup_logger();
        let mut client = GoogleClient::new();

        let events = client._retrieve_events("primary").await;

        assert!(events.is_ok(), "{}", events.err().unwrap());
        let events = events.unwrap();
        let event = events.items.iter().rev().collect::<Vec<_>>()[1];
        println!("2nd to last event: {:?}", event);
    }
}