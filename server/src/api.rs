use std::time::Duration;

use actix_web::{Result, error, get, post, web};
use serde::{Deserialize, Serialize};

struct AppData {}

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_secret)
        .service(post_secret)
        .app_data(web::Data::new(AppData {}));
}

#[derive(Deserialize)]
struct GetSecretRequest {
    id: uuid::Uuid,
}

#[derive(Deserialize)]
struct PostSecretRequest {
    data: String,
    expires_in: Option<Duration>,
}

#[derive(Deserialize, Serialize)]
struct PostSecretResponse {
    id: uuid::Uuid,
}

#[get("/secret/{id}")]
async fn get_secret(req: web::Path<GetSecretRequest>) -> Result<String> {
    Err(error::ErrorInternalServerError("Not implemented yet"))
}

#[post("/secret")]
async fn post_secret(form: web::Json<PostSecretRequest>) -> Result<web::Json<PostSecretResponse>> {
    Err(error::ErrorInternalServerError("Not implemented yet"))
}
