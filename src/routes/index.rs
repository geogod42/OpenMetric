// index.rs
use actix_web::{get, HttpRequest, HttpResponse, Responder};
use crate::metrics::calculators::MonthlyMetrics;
use crate::metrics::events::Event;
use crate::metrics::{get_data_files, load_events, load_retention, collect_monthly_metrics};
use crate::charts::generate_all_charts;
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};

#[get("/")]
pub async fn index(req: HttpRequest) -> impl Responder {
    let query: HashMap<String, String> = serde_urlencoded::from_str(req.query_string()).unwrap_or_default();
    let time_window = query.get("time_window").cloned().unwrap_or_else(|| "all".to_string());
    let page: usize = query.get("page").and_then(|v| v.parse().ok()).unwrap_or(0);

    let data_files = get_data_files();
    if data_files.is_empty() {
        return HttpResponse::Ok().body("<h1>No valid .evnt and .ret file pairs found in data/ folder</h1>");
    }

    let (events_file, retention_file) = &data_files[0];
    let events = load_events(events_file).unwrap_or_else(|e| panic!("Failed to load events: {}", e));
    let retention_map = load_retention(retention_file).unwrap_or_else(|e| panic!("Failed to load retention data: {}", e));

    let filtered_events = match time_window.as_str() {
        "3" => filter_events_by_months(&events, 3),
        "6" => filter_events_by_months(&events, 6),
        "12" => filter_events_by_months(&events, 12),
        _ => events,
    };

    let monthly_metrics = collect_monthly_metrics(&filtered_events, &retention_map);
    generate_all_charts(&monthly_metrics).expect("Failed to generate charts");

    let html = build_html_response(&monthly_metrics, events_file, page);
    HttpResponse::Ok().content_type("text/html").body(html)
}

pub fn filter_events_by_months(events: &[Event], months: i64) -> Vec<Event> {
    let cutoff_date = Utc::now() - Duration::days(months * 30);
    events
        .iter()
        .filter(|e| {
            let dt = DateTime::parse_from_rfc3339(&e.timestamp).unwrap().with_timezone(&Utc);
            dt > cutoff_date
        })
        .cloned()
        .collect()
}

fn build_html_response(metrics: &MonthlyMetrics, events_file: &str, page: usize) -> String {
    let last_idx = if metrics.months.is_empty() {
        0
    } else {
        metrics.months.len() - 1
    };

    let latest_revenue = metrics.revenue.get(last_idx).unwrap_or(&0.0);
    let latest_burn = metrics.burn_rate.get(last_idx).unwrap_or(&0.0);
    let latest_runway = metrics.runway.get(last_idx).unwrap_or(&0.0);
    let latest_retention = metrics.retention.get(last_idx).unwrap_or(&0.0);
    let latest_ndr = metrics.net_dollar_retention.get(last_idx).unwrap_or(&0.0);
    let latest_margin = metrics.gross_margin.get(last_idx).unwrap_or(&0.0);

    let html_template = format!(
        "<html><head><title>Dashboard</title></head><body>\
        <h1>Metrics Dashboard</h1>\
        <p>Revenue: ${:.2}, Burn Rate: ${:.2}, Runway: {:.2} months</p>\
        <p>Retention: {:.2}%, NDR: {:.2}%, Gross Margin: {:.2}%</p>\
        <p>Page: {}</p>\
        </body></html>",
        latest_revenue, latest_burn, latest_runway, latest_retention, latest_ndr, latest_margin, page
    );

    html_template
}

