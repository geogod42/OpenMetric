use plotters::prelude::*;
use plotters_svg::SVGBackend; // <-- Import the SVG backend

pub fn generate_time_series_chart(
    x_labels: &[String],
    y_values: &[f64],
    chart_title: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use SVGBackend here instead of BitMapBackend:
    let root_area = SVGBackend::new(output_path, (640, 480)).into_drawing_area();
    
    // Fill background with white
    root_area.fill(&WHITE)?;
    
    // Compute max for the y-axis
    let max_val = y_values
        .iter()
        .cloned()
        .fold(0.0_f64, f64::max)
        .max(10.0);

    let x_range = 0..x_labels.len();
    let y_range = 0.0..(max_val * 1.1);

    // Build a chart with black text and lines
    let mut chart = ChartBuilder::on(&root_area)
        .caption(chart_title, ("Arial", 22).into_font().color(&BLACK))
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(50)
        .build_cartesian_2d(x_range, y_range)?;

    chart
        .configure_mesh()
        .x_labels(x_labels.len())
        .x_label_formatter(&|idx| {
            x_labels.get(*idx).unwrap_or(&"".to_string()).clone()
        })
        .y_desc(chart_title)
        .y_label_style(("Arial", 14).into_font().color(&BLACK))
        .x_label_style(("Arial", 14).into_font().color(&BLACK))
        .axis_style(&BLACK) // make axis lines black
        .draw()?;

    // Draw the actual line series in black
    chart.draw_series(LineSeries::new(
        (0..).zip(y_values.iter()).map(|(i, &val)| (i, val)),
        &BLACK, // black line
    ))?
    .label(chart_title)
    .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 15, y)], &BLACK));

    // Draw legend (optional)
    chart.configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .label_font(("Arial", 14).into_font().color(&BLACK))
        .draw()?;

    Ok(())
}

