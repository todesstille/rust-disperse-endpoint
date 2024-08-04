use actix_web::{web, App, HttpServer};
mod connectors;
use connectors::disperse_connector::make_disperse;
use connectors::collect_connector::{make_collect, make_collect_percent};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .route("/disperse/amounts", web::post().to(make_disperse))
            .route("/collect/amounts", web::post().to(make_collect))
            .route("/collect/percents", web::post().to(make_collect_percent))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}