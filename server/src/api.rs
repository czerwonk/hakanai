use actix_web::{Result, error, get, post, web};
use uuid::Uuid;

use hakanai_lib::models::{PostSecretRequest, PostSecretResponse};

use crate::data_store::DataStore;

struct AppData {
    data_store: Box<dyn DataStore>,
}

/// Configures the Actix Web services for the application.
///
/// This function registers the API routes and sets up the application data,
/// including the data store that will be shared across all handlers.
pub fn configure(cfg: &mut web::ServiceConfig, data_store: Box<dyn DataStore>) {
    let app_data = AppData {
        data_store: data_store,
    };

    cfg.service(get_secret)
        .service(post_secret)
        .app_data(web::Data::new(app_data));
}

#[get("/secret/{id}")]
async fn get_secret(req: web::Path<Uuid>, app_data: web::Data<AppData>) -> Result<String> {
    let id = req.into_inner();
    let mut data_store = app_data.data_store.as_mut();

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

    let data_store = app_data.data_store.as_mut();
    data_store
        .put(id, req.data.clone(), req.expires_in)
        .await
        .map_err(|e| error::ErrorInternalServerError(e))?;

    Ok(web::Json(PostSecretResponse { id }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};
    use async_trait::async_trait;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;
    use uuid::Uuid;

    use crate::data_store::{DataStore, DataStoreError};

    struct MockDataStore {
        get_result: Option<Option<String>>,
        get_error: bool,
        put_error: bool,
        stored_data: Arc<Mutex<Vec<(Uuid, String, Duration)>>>,
    }

    impl MockDataStore {
        fn new() -> Self {
            MockDataStore {
                get_result: None,
                get_error: false,
                put_error: false,
                stored_data: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn with_get_result(mut self, result: Option<String>) -> Self {
            self.get_result = Some(result);
            self
        }

        fn with_get_error(mut self) -> Self {
            self.get_error = true;
            self
        }

        fn with_put_error(mut self) -> Self {
            self.put_error = true;
            self
        }
    }

    #[async_trait]
    impl DataStore for MockDataStore {
        async fn get(&mut self, _id: Uuid) -> Result<Option<String>, DataStoreError> {
            if self.get_error {
                Err(DataStoreError::InternalError("mock error".to_string()))
            } else {
                Ok(self.get_result.clone().unwrap_or(None))
            }
        }

        async fn put(
            &mut self,
            id: Uuid,
            data: String,
            expires_in: Duration,
        ) -> Result<(), DataStoreError> {
            if self.put_error {
                Err(DataStoreError::InternalError("mock error".to_string()))
            } else {
                let mut stored = self.stored_data.lock().unwrap();
                stored.push((id, data, expires_in));
                Ok(())
            }
        }
    }

    #[actix_web::test]
    async fn test_get_secret_found() {
        let mock_store = MockDataStore::new().with_get_result(Some("test_secret".to_string()));

        let app =
            test::init_service(App::new().configure(|cfg| configure(cfg, Box::new(mock_store))))
                .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", uuid::Uuid::new_v4()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        assert_eq!(body, "test_secret");
    }

    #[actix_web::test]
    async fn test_get_secret_not_found() {
        let mock_store = MockDataStore::new().with_get_result(None);

        let app =
            test::init_service(App::new().configure(|cfg| configure(cfg, Box::new(mock_store))))
                .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", uuid::Uuid::new_v4()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_get_secret_error() {
        let mock_store = MockDataStore::new().with_get_error();

        let app =
            test::init_service(App::new().configure(|cfg| configure(cfg, Box::new(mock_store))))
                .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", uuid::Uuid::new_v4()))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);
    }

    #[actix_web::test]
    async fn test_post_secret_success() {
        let mock_store = MockDataStore::new();
        let stored_data = mock_store.stored_data.clone();

        let app =
            test::init_service(App::new().configure(|cfg| configure(cfg, Box::new(mock_store))))
                .await;

        let payload = PostSecretRequest {
            data: "test_secret".to_string(),
            expires_in: Duration::from_secs(3600),
        };

        let req = test::TestRequest::post()
            .uri("/secret")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: PostSecretResponse = test::read_body_json(resp).await;
        assert!(!body.id.is_nil());

        let stored = stored_data.lock().unwrap();
        assert_eq!(stored.len(), 1);
        assert_eq!(stored[0].1, "test_secret");
        assert_eq!(stored[0].2, Duration::from_secs(3600));
    }

    #[actix_web::test]
    async fn test_post_secret_error() {
        let mock_store = MockDataStore::new().with_put_error();

        let app =
            test::init_service(App::new().configure(|cfg| configure(cfg, Box::new(mock_store))))
                .await;

        let payload = PostSecretRequest {
            data: "test_secret".to_string(),
            expires_in: Duration::from_secs(3600),
        };

        let req = test::TestRequest::post()
            .uri("/secret")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);
    }
}
