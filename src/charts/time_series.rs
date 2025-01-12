use plotters::prelude::*;
use plotters_svg::SVGBackend;

/// Formats numeric value into $/k/m format.
fn format_revenue(value: f64) -> String {
    if value.abs() < 1_000.0 {
        format!("${:.0}", value)
    } else if value.abs() < 1_000_000.0 {
        format!("${:.1}k", value / 1_000.0)
    } else {
        format!("${:.2}m", value / 1_000_000.0)
    }
}

pub fn generate_time_series_chart(
    x_labels: &[String],
    y_values: &[f64],
    chart_title: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_area = SVGBackend::new(output_path, (640, 480)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let max_val = y_values.iter().cloned().fold(0.0_f64, f64::max).max(10.0);
    let x_range = 0..x_labels.len();
    let y_range = 0.0..(max_val * 1.1);

    let mut chart = ChartBuilder::on(&root_area)
        .caption(chart_title, ("Arial", 22).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_range, y_range)?;

    chart.configure_mesh()
        .x_labels(x_labels.len())
        .x_label_formatter(&|idx| x_labels.get(*idx).cloned().unwrap_or_default())
        .y_label_formatter(&|val| format_revenue(*val))
        .draw()?;

    chart.draw_series(LineSeries::new(
        (0..).zip(y_values.iter()).map(|(i, &val)| (i, val)),
        &BLACK,
    ))?;

    Ok(())
}
