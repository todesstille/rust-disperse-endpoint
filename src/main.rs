use actix_web::{web, App, HttpServer};
mod connectors;
use connectors::disperse_connector::make_disperse;
use connectors::collect_connector::make_collect;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .route("/disperse", web::post().to(make_disperse))
            .route("/collect", web::post().to(make_collect))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}