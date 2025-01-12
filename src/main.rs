mod metrics;
mod charts;
mod routes;

use actix_web::{App, HttpServer, web}; // Add `web` here
use routes::index::index;
use routes::websocket::metrics_ws;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    println!("Starting server at http://127.0.0.1:8080/");

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(
                web::resource("/metrics_ws").route(web::get().to(metrics_ws)) // Uses `web` here
            )
            .service(actix_files::Files::new("/static", "./src/"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

