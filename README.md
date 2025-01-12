
# OpenMetric

## Overview

**OpenMetric** is a business metrics dashboard designed to provide essential business metrics such as revenue, burn rate, runway, retention, net dollar retention (NDR), and gross margin, presented through a web-based dashboard.

## Features

- **Data-Driven Insights**: Parse event and retention data to generate key business metrics.
- **Interactive Charts**: Generate time-series charts for quick visual analysis.
- **WebSocket Support**: Real-time updates for selected time ranges.
- **Lightweight Design**: Optimized for simplicity and ease of use.

## Project Structure

```plaintext
business_metrics/
├── Cargo.toml         # Project dependencies and metadata
├── src/
│   ├── main.rs        # Entry point, initializes the web server
│   ├── metrics/       # Metric-related logic and data processing
│   │   ├── mod.rs     # Re-exports for metrics
│   │   ├── events.rs  # Event-related structs and logic
│   │   ├── retention.rs # Retention data handling
│   │   └── calculators.rs # Functions for metric calculations (NDR, gross margin, etc.)
│   ├── charts/        # Chart generation logic
│   │   ├── mod.rs     # Re-exports for charts
│   │   └── time_series.rs # Time-series chart generation
│   ├── routes/        # Web routes for the application
│   │   ├── mod.rs     # Re-exports for routes
│   │   └── index.rs   # Dashboard ("/") route
│   ├── templates/     # Web server HTML templates (for charts and data display)
│   │   ├── index.html
│   │   └── charts/
│   │       └── chart1.png
├── data/              # Test data files
│   ├── TEST_seriesD.evnt   # Event data for series D startup
│   ├── TEST_seriesD.ret    # Retention data for series D startup
│   └── other/         # Additional test files
```

## Dependencies

This project leverages the following Rust crates:

- `serde` and `serde_json` for JSON serialization and deserialization
- `actix-web` for building the web server
- `plotters` and `plotters-svg` for chart generation
- `chrono` for handling date and time
- `env_logger` and `log` for logging
- `serde_urlencoded` for handling URL query strings


## Getting Started

### Prerequisites

- Install [Rust](https://www.rust-lang.org/)
- Clone this repository
- Navigate to the project directory and run `cargo build`

### Run the Server

To start the web server:

```bash
cargo run
```

The server will be accessible at `http://127.0.0.1:8080/`.

### Test Data

Sample event and retention data can be found in the `data/` directory. Modify these files to experiment with different scenarios.

## Roadmap

- [ ] Implement real-time chart updates with WebSockets
- [ ] Add authentication and user management
- [ ] Extend metric calculations with additional business KPIs
- [ ] Optimize chart rendering for large datasets
- [ ] Dockerize


## Acknowledgments

Shout out the Rust community

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.