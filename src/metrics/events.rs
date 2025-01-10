use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    pub event_type: String,
    pub customer_id: Option<u32>,
    pub amount: Option<f64>,
    pub description: Option<String>,
    pub timestamp: String,
}

pub fn load_events(file_path: &str) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let events: Vec<Event> = serde_json::from_str(&data)?;

    println!("Loaded events: {:?}", events); // Debug log
    Ok(events)
}

