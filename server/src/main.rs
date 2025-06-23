mod api;
mod options;

use std::io::Result;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use clap::Parser;

use crate::api::AppData;
use crate::options::Args;

#[actix_web::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    HttpServer::new(|| {
        App::new()
            .app_data(web::Data::new(AppData {}))
            .route("/healthz", web::get().to(healthz))
    })
    .bind((args.listen_address, args.port))?
    .run()
    .await
}

async fn healthz(_: String) -> impl Responder {
    HttpResponse::Ok().body("healthy")
}
