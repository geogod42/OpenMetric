// calculators.rs
use crate::metrics::events::Event;
use crate::metrics::retention::RetentionData;
use chrono::{DateTime, Utc, Datelike};
use std::collections::HashMap;

/// Convert a month number (1-12) to an English month name used by the
/// retention JSON files.
fn month_number_to_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

pub struct MonthlyMetrics {
    pub months: Vec<String>,
    pub revenue: Vec<f64>,
    pub burn_rate: Vec<f64>,
    pub runway: Vec<f64>,
    pub retention: Vec<f64>,
    pub net_dollar_retention: Vec<f64>,
    pub gross_margin: Vec<f64>,
}

pub fn collect_monthly_metrics(
    events: &[Event],
    retention_map: &HashMap<String, RetentionData>,
) -> MonthlyMetrics {
    let grouped_events = group_events_by_month(events);

    let mut month_keys: Vec<String> = grouped_events.keys().cloned().collect();
    month_keys.sort();

    let mut revenue = Vec::new();
    let mut burn_rate = Vec::new();
    let mut runway = Vec::new();
    let mut retention = Vec::new();
    let mut ndr = Vec::new();
    let mut gross_margin = Vec::new();

    let static_empty_vec: Vec<Event> = Vec::new();

    for m in &month_keys {
        let events = grouped_events.get(m).unwrap_or(&static_empty_vec);
        let monthly_revenue: f64 = events.iter()
            .filter(|e| e.event_type == "payment")
            .map(|e| e.amount.unwrap_or(0.0))
            .sum();
        let monthly_expenses: f64 = events.iter()
            .filter(|e| e.event_type == "expense")
            .map(|e| e.amount.unwrap_or(0.0))
            .sum();

        revenue.push(monthly_revenue);
        burn_rate.push(monthly_expenses - monthly_revenue);
        runway.push(if monthly_expenses > 0.0 {
            monthly_revenue / monthly_expenses
        } else {
            f64::INFINITY
        });

        // retention data files use human-readable month names (e.g. "January")
        // while the events are grouped by "YYYY-MM". Convert the key so we can
        // look up the correct retention entry.
        let month_num: u32 = m.split('-').nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let month_name = month_number_to_name(month_num);

        if let Some(retention_data) = retention_map.get(month_name) {
            if retention_data.acquired > 0 {
                retention.push(
                    retention_data.active.iter().sum::<u32>() as f64 / retention_data.acquired as f64 * 100.0,
                );
            } else {
                retention.push(0.0);
            }
        } else {
            retention.push(0.0);
        }

        ndr.push(calculate_net_dollar_retention(events));
        gross_margin.push(calculate_gross_margin(monthly_revenue, monthly_expenses));
    }

    MonthlyMetrics {
        months: month_keys,
        revenue,
        burn_rate,
        runway,
        retention,
        net_dollar_retention: ndr,
        gross_margin,
    }
}

fn group_events_by_month(events: &[Event]) -> HashMap<String, Vec<Event>> {
    let mut grouped_events: HashMap<String, Vec<Event>> = HashMap::new();

    for event in events {
        let timestamp = DateTime::parse_from_rfc3339(&event.timestamp)
            .unwrap()
            .with_timezone(&Utc);
        let month_key = format!("{}-{:02}", timestamp.year(), timestamp.month());

        grouped_events.entry(month_key).or_default().push(event.clone());
    }

    grouped_events
}

pub fn calculate_net_dollar_retention(_events: &[Event]) -> f64 {
    100.0 // Placeholder logic
}

pub fn calculate_gross_margin(revenue: f64, expenses: f64) -> f64 {
    if revenue == 0.0 {
        0.0
    } else {
        ((revenue - expenses) / revenue) * 100.0
    }
}

pub fn format_revenue(value: f64) -> String {
    if value.abs() < 1_000.0 {
        format!("${:.0}", value)
    } else if value.abs() < 1_000_000.0 {
        format!("${:.1}k", value / 1_000.0)
    } else {
        format!("${:.2}m", value / 1_000_000.0)
    }
}
