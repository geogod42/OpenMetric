pub mod events;
pub mod retention;
pub mod calculators;

pub use events::load_events;
pub use retention::load_retention;
pub use calculators::{collect_monthly_metrics};


use std::fs;

pub fn get_data_files() -> Vec<(String, String)> {
    let mut evnt_files = vec![];
    let mut ret_files = vec![];

    if let Ok(entries) = fs::read_dir("data/") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "evnt" {
                    evnt_files.push(path.file_stem().unwrap().to_string_lossy().to_string());
                } else if ext == "ret" {
                    ret_files.push(path.file_stem().unwrap().to_string_lossy().to_string());
                }
            }
        }
    }

    evnt_files
        .iter()
        .filter_map(|evnt| {
            ret_files
                .iter()
                .find(|ret| *ret == evnt)
                .map(|ret| (format!("data/{}.evnt", evnt), format!("data/{}.ret", ret)))
        })
        .collect()
}

