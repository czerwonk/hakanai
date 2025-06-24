mod api;
mod data_store;
mod options;

use std::io::Result;

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use clap::Parser;

use crate::data_store::RedisDataStore;
use crate::options::Args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let data_store: RedisDataStore = match RedisDataStore::new(&args.redis_dsn).await {
        Ok(store) => store,
        Err(e) => {
            eprintln!("Failed to create Redis data store: {}", e);
            return Err(std::io::Error::other(e));
        }
    };

    HttpServer::new(move || {
        App::new()
            .route("/healthz", web::get().to(healthz))
            .service(
                web::scope("/api")
                    .configure(|cfg| api::configure(cfg, Box::new(data_store.clone()))),
            )
    })
    .bind((args.listen_address, args.port))?
    .run()
    .await
}

async fn healthz(_: String) -> impl Responder {
    HttpResponse::Ok().body("healthy")
}
