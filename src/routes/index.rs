use actix_web::{get, HttpResponse, Responder};
use crate::metrics::{get_data_files, load_events, load_retention, collect_monthly_metrics};
use crate::charts::generate_all_charts;

#[get("/")]
pub async fn index() -> impl Responder {
    // Identify data files
    let data_files = get_data_files();
    if data_files.is_empty() {
        return HttpResponse::Ok().body("<h1>No valid .evnt and .ret file pairs found in data/ folder</h1>");
    }

    // Load events and retention data
    let (events_file, retention_file) = &data_files[0];
    let events = load_events(events_file).unwrap_or_else(|e| panic!("Failed to load events: {}", e));
    let retention_map = load_retention(retention_file).unwrap_or_else(|e| panic!("Failed to load retention data: {}", e));

    // Calculate monthly metrics
    let monthly_metrics = collect_monthly_metrics(&events, &retention_map);

    // Generate charts (now .svg files)
    generate_all_charts(&monthly_metrics).expect("Failed to generate charts");

    // Build HTML response
    let html = build_html_response(&monthly_metrics);
    HttpResponse::Ok().content_type("text/html").body(html)
}

fn build_html_response(metrics: &crate::metrics::calculators::MonthlyMetrics) -> String {
    let last_idx = if metrics.months.is_empty() { 0 } else { metrics.months.len() - 1 };

    let spec_boxes = format!(
        r#"
        <div class="metric-box">
            <div class="metric-info">
                <h2>Revenue</h2>
                <p class="value">${:.2}</p>
            </div>
            <div class="chart-container">
                <object data="/static/templates/charts/chart_revenue.svg" type="image/svg+xml"></object>
            </div>
        </div>
        <div class="metric-box">
            <div class="metric-info">
                <h2>Burn Rate</h2>
                <p class="value">${:.2}</p>
            </div>
            <div class="chart-container">
                <object data="/static/templates/charts/chart_burn_rate.svg" type="image/svg+xml"></object>
            </div>
        </div>
        <div class="metric-box">
            <div class="metric-info">
                <h2>Runway (Months)</h2>
                <p class="value">{:.2}</p>
            </div>
            <div class="chart-container">
                <object data="/static/templates/charts/chart_runway.svg" type="image/svg+xml"></object>
            </div>
        </div>
        <div class="metric-box">
            <div class="metric-info">
                <h2>Retention (%)</h2>
                <p class="value">{:.2}%</p>
            </div>
            <div class="chart-container">
                <object data="/static/templates/charts/chart_retention.svg" type="image/svg+xml"></object>
            </div>
        </div>
        <div class="metric-box">
            <div class="metric-info">
                <h2>Net Dollar Ret. (%)</h2>
                <p class="value">{:.2}%</p>
            </div>
            <div class="chart-container">
                <object data="/static/templates/charts/chart_ndr.svg" type="image/svg+xml"></object>
            </div>
        </div>
        <div class="metric-box">
            <div class="metric-info">
                <h2>Gross Margin (%)</h2>
                <p class="value">{:.2}%</p>
            </div>
            <div class="chart-container">
                <object data="/static/templates/charts/chart_gross_margin.svg" type="image/svg+xml"></object>
            </div>
        </div>
        "#,
        metrics.revenue.get(last_idx).unwrap_or(&0.0),
        metrics.burn_rate.get(last_idx).unwrap_or(&0.0),
        metrics.runway.get(last_idx).unwrap_or(&0.0),
        metrics.retention.get(last_idx).unwrap_or(&0.0),
        metrics.net_dollar_retention.get(last_idx).unwrap_or(&0.0),
        metrics.gross_margin.get(last_idx).unwrap_or(&0.0),
    );

    format!(
        r#"
        <html>
        <head>
            <title>Business Metrics Dashboard</title>
            <style>
                body {{
                    font-family: Arial, sans-serif;
                    background-color: #fff; 
                    color: #000;
                }}
                h1 {{
                    text-align: center;
                    margin-top: 20px;
                }}
                h2 {{
                    margin: 0;
                    font-size: 1.5em;
                }}
                .file-name {{
                    text-align: center;
                    font-size: 1.2em;
                    margin-bottom: 20px;
                    color: #555;
                }}
                .grid-container {{
                    display: grid;
                    grid-template-columns: repeat(auto-fit, minmax(600px, 1fr));
                    gap: 20px;
                    padding: 20px;
                }}
                .metric-box {{
                    border: 3px solid #000;
                    border-radius: 15px; /* Rounded borders */
                    display: flex;
                    padding: 20px;
                    background-color: #fff;
                }}
                .metric-info {{
                    flex: 1;
                    margin-right: 20px;
                }}
                .metric-info .value {{
                    font-size: 1.5em;
                    margin-top: 10px;
                }}
                .chart-container {{
                    flex: 2;
                }}
                .chart-container object {{
                    width: 100%;
                    height: 300px;
                    border: 1px solid #ccc;
                }}
            </style>
        </head>
        <body>
            <h1>Business Metrics Dashboard</h1>
            <div class="file-name">Loaded File: data/your_file_name.evnt</div>
            <div class="grid-container">
                {spec_boxes}
            </div>
        </body>
        </html>
        "#,
        spec_boxes = spec_boxes
    )
}


