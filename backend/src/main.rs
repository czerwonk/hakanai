use std::io::Result;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};

#[actix_web::main]
async fn main() -> Result<()> {
    HttpServer::new(|| App::new().route("/healthz", web::get().to(healthz)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

async fn healthz(_: String) -> impl Responder {
    HttpResponse::Ok().body("healthy")
}
