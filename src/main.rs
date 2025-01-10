mod metrics;
mod charts;
mod routes;

use actix_web::{App, HttpServer};
use routes::index::index;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    println!("Starting server at http://127.0.0.1:8080/");

    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(actix_files::Files::new("/static", "./src/"))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

