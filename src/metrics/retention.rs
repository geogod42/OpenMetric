use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
pub struct RetentionData {
    pub acquired: u32,
    pub active: Vec<u32>,
}

pub fn load_retention(file_path: &str) -> Result<HashMap<String, RetentionData>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let retention: HashMap<String, RetentionData> = serde_json::from_str(&data)?;

    println!("Loaded retention data: {:?}", retention); // Debug log
    Ok(retention)
}

