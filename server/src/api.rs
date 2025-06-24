use std::time::Duration;

use actix_web::{Result, error, get, post, web};
use serde::{Deserialize, Serialize};

use crate::data_store::DataStore;

struct AppData {
    data_store: Box<dyn DataStore>,
}

pub fn configure(cfg: &mut web::ServiceConfig, data_store: Box<dyn DataStore>) {
    let app_data = AppData {
        data_store: data_store,
    };

    cfg.service(get_secret)
        .service(post_secret)
        .app_data(web::Data::new(app_data));
}

#[derive(Deserialize)]
struct GetSecretRequest {
    id: uuid::Uuid,
}

#[derive(Deserialize)]
struct PostSecretRequest {
    data: String,
    expires_in: Duration,
}

#[derive(Deserialize, Serialize)]
struct PostSecretResponse {
    id: uuid::Uuid,
}

#[get("/secret/{id}")]
async fn get_secret(
    req: web::Path<GetSecretRequest>,
    app_data: web::Data<AppData>,
) -> Result<String> {
    let id = req.id;
    let data_store = app_data.data_store.as_ref();

    match data_store.get(id).await {
        Ok(data) => match data {
            Some(secret) => Ok(secret),
            None => Err(error::ErrorNotFound("Secret not found")),
        },
        Err(e) => Err(error::ErrorInternalServerError(e)),
    }
}

#[post("/secret")]
async fn post_secret(
    req: web::Json<PostSecretRequest>,
    app_data: web::Data<AppData>,
) -> Result<web::Json<PostSecretResponse>> {
    let id = uuid::Uuid::new_v4();

    let data_store = app_data.data_store.as_ref();
    data_store
        .put(id, req.data.clone(), req.expires_in)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(web::Json(PostSecretResponse { id }))
}
