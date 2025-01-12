use actix::prelude::*; // Use prelude to import Actor and StreamHandler
use actix_web::{web, HttpRequest, Responder};
use actix_web_actors::ws::{self, WebsocketContext};
use serde_json::json;
use crate::metrics::{get_data_files, load_events, load_retention, collect_monthly_metrics};
use crate::routes::index::filter_events_by_months;

pub struct MetricsWebSocket;

impl Actor for MetricsWebSocket {
    type Context = WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MetricsWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                let request: serde_json::Value = serde_json::from_str(&text).unwrap_or(json!({}));
                if let Some(time_range) = request.get("value").and_then(|v| v.as_u64()) {
                    let data_files = get_data_files();
                    if data_files.is_empty() {
                        ctx.text(json!({ "error": "No data files found" }).to_string());
                        return;
                    }

                    let (events_file, retention_file) = &data_files[0];
                    let events = load_events(events_file).unwrap();
                    let retention_map = load_retention(retention_file).unwrap();
                    let filtered_events = filter_events_by_months(&events, time_range as i64);
                    let metrics = collect_monthly_metrics(&filtered_events, &retention_map);

                    let response = json!({
                        "months": metrics.months,
                        "revenue": metrics.revenue,
                        "burn_rate": metrics.burn_rate,
                        "runway": metrics.runway,
                        "retention": metrics.retention,
                        "net_dollar_retention": metrics.net_dollar_retention,
                        "gross_margin": metrics.gross_margin,
                    });

                    ctx.text(response.to_string());
                } else {
                    ctx.text(json!({ "error": "Invalid time range" }).to_string());
                }
            }
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            _ => (),
        }
    }
}

pub async fn metrics_ws(req: HttpRequest, stream: web::Payload) -> impl Responder {
    ws::start(MetricsWebSocket {}, &req, stream)
}

