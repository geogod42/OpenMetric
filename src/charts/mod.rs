pub mod time_series; // Declare the time_series module

use crate::metrics::calculators::MonthlyMetrics;
use std::error::Error;
use crate::charts::time_series::generate_time_series_chart; // Import generate_time_series_chart function

pub fn generate_all_charts(metrics: &MonthlyMetrics) -> Result<(), Box<dyn Error>> {
    let chart_paths = vec![
        ("Revenue",            "src/templates/charts/chart_revenue.svg",         &metrics.revenue),
        ("Burn Rate",          "src/templates/charts/chart_burn_rate.svg",       &metrics.burn_rate),
        ("Runway (Months)",    "src/templates/charts/chart_runway.svg",          &metrics.runway),
        ("Retention (%)",      "src/templates/charts/chart_retention.svg",       &metrics.retention),
        ("Net Dollar Ret. (%)","src/templates/charts/chart_ndr.svg",             &metrics.net_dollar_retention),
        ("Gross Margin (%)",   "src/templates/charts/chart_gross_margin.svg",    &metrics.gross_margin),
    ];

    for (title, path, data) in chart_paths {
        generate_time_series_chart(&metrics.months, data, title, path)?;
    }

    Ok(())
}

