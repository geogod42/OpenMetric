use actix_files as afs; // Rename actix_files to afs to avoid conflict
use actix_web::{get, App, HttpServer, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use chrono::{DateTime, Utc, Datelike}; // For date parsing
use plotters::prelude::*;

/// Represents an event in the system, e.g., payment, expense, cancellation, etc.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Event {
    event_type: String,
    customer_id: Option<u32>,
    amount: Option<f64>,
    description: Option<String>,
    timestamp: String,
}

/// Retention data for a given time period (e.g., a month or quarter).
#[derive(Serialize, Deserialize, Debug)]
struct RetentionData {
    acquired: u32,      // Number of newly acquired users in this period
    active: Vec<u32>,   // Number of active users in subsequent weeks/months
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://127.0.0.1:8080/");
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(afs::Files::new("/static", "./src/")) // Serve static files like charts
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

/// Main dashboard route
#[get("/")]
async fn index() -> impl Responder {
    // 1. Identify data files
    let data_files = get_data_files();
    if data_files.is_empty() {
        return HttpResponse::Ok()
            .content_type("text/html")
            .body("<h1>No valid .evnt and .ret file pairs found in data/ folder</h1>");
    }

    // 2. Load events & retention data from first pair
    let (events_file, retention_file) = &data_files[0];
    let events = match load_events(events_file) {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load events: {}", e));
        }
    };

    let retention_map = match load_retention(retention_file) {
        Ok(data) => data,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to load retention data: {}", e));
        }
    };

    // 3. Collect monthly metrics
    let monthly_metrics = collect_monthly_metrics(&events, &retention_map);

    // 4. Generate charts
    if let Err(e) = generate_all_charts(&monthly_metrics) {
        return HttpResponse::InternalServerError()
            .body(format!("Failed to generate charts: {}", e));
    }

    // 5. Build final HTML response
    let html_response = build_html_response(&monthly_metrics);

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html_response)
}

/// A struct to hold all monthly series for each metric
struct MonthlyMetrics {
    months: Vec<String>,
    revenue: Vec<f64>,
    burn_rate: Vec<f64>,
    runway: Vec<f64>,
    retention: Vec<f64>,
    net_dollar_retention: Vec<f64>,
    gross_margin: Vec<f64>,
}

/// Collect monthly metrics for each metric
fn collect_monthly_metrics(
    events: &[Event],
    retention_map: &HashMap<String, RetentionData>,
) -> MonthlyMetrics {
    // Group events by month
    let grouped_events = group_events_by_month(events);

    // Sort month keys
    let mut month_keys: Vec<String> = grouped_events.keys().cloned().collect();
    month_keys.sort();

    let mut revenue = Vec::new();
    let mut burn_rate = Vec::new();
    let mut runway = Vec::new();
    let mut retention = Vec::new();
    let mut ndr = Vec::new();
    let mut gross_margin = Vec::new();

    for m in &month_keys {
        let events = grouped_events.get(m).unwrap();
        revenue.push(events.iter().filter(|e| e.event_type == "payment").map(|e| e.amount.unwrap_or(0.0)).sum());
        let expenses: f64 = events.iter().filter(|e| e.event_type == "expense").map(|e| e.amount.unwrap_or(0.0)).sum();
        burn_rate.push(expenses - revenue.last().unwrap_or(&0.0));
        runway.push(if burn_rate.last().unwrap_or(&0.0) > &0.0 { 10000.0 / burn_rate.last().unwrap_or(&0.0) } else { f64::INFINITY });
        retention.push(retention_map.get(m).map(|r| r.active.iter().sum::<u32>() as f64 / r.acquired as f64 * 100.0).unwrap_or(0.0));
        ndr.push(calculate_net_dollar_retention(events));
        gross_margin.push(calculate_gross_margin(events));
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

/// Generate all charts
fn generate_all_charts(metrics: &MonthlyMetrics) -> Result<(), Box<dyn std::error::Error>> {
    generate_time_series_chart(&metrics.months, &metrics.revenue, "Revenue", "src/templates/charts/chart_revenue.png")?;
    generate_time_series_chart(&metrics.months, &metrics.burn_rate, "Burn Rate", "src/templates/charts/chart_burn_rate.png")?;
    generate_time_series_chart(&metrics.months, &metrics.runway, "Runway (Months)", "src/templates/charts/chart_runway.png")?;
    generate_time_series_chart(&metrics.months, &metrics.retention, "Retention (%)", "src/templates/charts/chart_retention.png")?;
    generate_time_series_chart(&metrics.months, &metrics.net_dollar_retention, "Net Dollar Retention (%)", "src/templates/charts/chart_ndr.png")?;
    generate_time_series_chart(&metrics.months, &metrics.gross_margin, "Gross Margin (%)", "src/templates/charts/chart_gross_margin.png")?;
    Ok(())
}

/// Build HTML response
fn build_html_response(metrics: &MonthlyMetrics) -> String {
    let mut html_table = String::new();
    html_table.push_str("<table border='1' style='border-collapse: collapse;'>\n");
    html_table.push_str("<tr><th>Month</th><th>Revenue</th><th>Burn Rate</th><th>Runway</th><th>Retention</th><th>NDR</th><th>Gross Margin</th></tr>\n");

    for i in 0..metrics.months.len() {
        html_table.push_str(&format!(
            "<tr><td>{}</td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td><td>{:.2}</td></tr>\n",
            metrics.months[i],
            metrics.revenue[i],
            metrics.burn_rate[i],
            metrics.runway[i],
            metrics.retention[i],
            metrics.net_dollar_retention[i],
            metrics.gross_margin[i]
        ));
    }
    html_table.push_str("</table>\n");

    format!(
        r#"<html>
            <head><title>Business Metrics Dashboard</title></head>
            <body>
                <h1>Business Metrics</h1>
                <p>{}</p>
                <img src="/static/templates/charts/chart_revenue.png" />
                <img src="/static/templates/charts/chart_burn_rate.png" />
                <img src="/static/templates/charts/chart_runway.png" />
                <img src="/static/templates/charts/chart_retention.png" />
                <img src="/static/templates/charts/chart_ndr.png" />
                <img src="/static/templates/charts/chart_gross_margin.png" />
            </body>
        </html>"#,
        html_table
    )
}

/// Group events by month (YYYY-MM)
fn group_events_by_month(events: &[Event]) -> HashMap<String, Vec<Event>> {
    let mut map: HashMap<String, Vec<Event>> = HashMap::new();
    for e in events {
        let dt = DateTime::parse_from_rfc3339(&e.timestamp).unwrap().with_timezone(&Utc);
        let month_key = format!("{}-{:02}", dt.year(), dt.month());
        map.entry(month_key).or_default().push(e.clone());
    }
    map
}

/// Load events from a JSON file
fn load_events(file_path: &str) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let events: Vec<Event> = serde_json::from_str(&data)?;
    Ok(events)
}

/// Load retention data from a JSON file
fn load_retention(file_path: &str) -> Result<HashMap<String, RetentionData>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;
    let retention: HashMap<String, RetentionData> = serde_json::from_str(&data)?;
    Ok(retention)
}

/// Generate a simple time-series chart (PNG)
fn generate_time_series_chart(
    x_labels: &[String],
    y_values: &[f64],
    chart_title: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_area = BitMapBackend::new(output_path, (640, 480)).into_drawing_area();
    root_area.fill(&BLACK)?;

    let max_val = y_values.iter().cloned().fold(0.0_f64, f64::max).max(10.0);
    let x_range = 0..x_labels.len();
    let y_range = 0.0..(max_val * 1.1);

    let mut chart = ChartBuilder::on(&root_area)
        .caption(chart_title, ("Arial", 25).into_font().color(&WHITE))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_range, y_range)?;

    chart.configure_mesh()
        .x_labels(x_labels.len())
        .x_label_formatter(&|idx| {
            if let Some(label) = x_labels.get(*idx) {
                label.clone()
            } else {
                "".to_string()
            }
        })
        .y_desc(chart_title)
        .y_label_style(("Arial", 15).into_font().color(&WHITE))
        .x_label_style(("Arial", 15).into_font().color(&WHITE))
        .draw()?;

    chart.draw_series(LineSeries::new(
        (0..).zip(y_values.iter()).map(|(i, &val)| (i, val)),
        &GREEN,
    ))?;

    Ok(())
}

/// Calculate Net Dollar Retention (toy logic)
fn calculate_net_dollar_retention(events: &[Event]) -> f64 {
    let mut starting_mrr = 0.0;
    let mut ending_mrr = 0.0;
    for e in events {
        match e.event_type.as_str() {
            "payment" => {
                starting_mrr += e.amount.unwrap_or(0.0);
                ending_mrr += e.amount.unwrap_or(0.0);
            }
            "cancellation" => {
                ending_mrr -= 12000.0; // Example churn logic
            }
            _ => {}
        }
    }
    if starting_mrr.abs() < f64::EPSILON {
        return 100.0;
    }
    (ending_mrr / starting_mrr) * 100.0
}

/// Calculate Gross Margin (toy logic)
fn calculate_gross_margin(events: &[Event]) -> f64 {
    let revenue: f64 = events.iter().filter(|e| e.event_type == "payment").map(|e| e.amount.unwrap_or(0.0)).sum();
    if revenue <= 0.0 {
        return 0.0;
    }
    let expenses: f64 = events.iter().filter(|e| e.event_type == "expense").map(|e| e.amount.unwrap_or(0.0)).sum();
    let cogs = expenses * 0.5; // Cost of goods sold
    ((revenue - cogs) / revenue) * 100.0
}

/// Get paired .evnt and .ret files in data/ directory
fn get_data_files() -> Vec<(String, String)> {
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

