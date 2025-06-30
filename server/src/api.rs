use std::time::Duration;

use actix_web::{HttpRequest, Result, error, get, post, web};
use subtle::ConstantTimeEq;
use tracing::error;
use uuid::Uuid;

use hakanai_lib::models::{PostSecretRequest, PostSecretResponse};

use crate::app_data::AppData;

/// Configures the Actix Web services for the application.
///
/// This function registers the API routes and sets up the application data,
/// including the data store that will be shared across all handlers.
pub fn configure(cfg: &mut web::ServiceConfig) -> &mut web::ServiceConfig {
    cfg.service(get_secret).service(post_secret)
}

/// Retrieves and consumes a secret from the data store.
///
/// This function handles the core logic for the `GET /secret/{id}` endpoint.
/// It parses the UUID from the request path, retrieves the corresponding secret
/// from the data store, and returns it. Upon successful retrieval, the secret
/// is consumed and can no longer be accessed.
///
/// # Arguments
///
/// * `req` - The request path containing the secret's ID.
/// * `app_data` - The application's shared data, including the data store.
///
/// # Errors
///
/// This function will return an error if:
/// - The provided ID is not a valid UUID (`ErrorBadRequest`).
/// - The secret is not found in the data store (`ErrorNotFound`).
/// - An internal error occurs while accessing the data store (`ErrorInternalServerError`).
pub async fn get_secret_from_request(
    req: web::Path<String>,
    app_data: web::Data<AppData>,
) -> Result<String> {
    let id = Uuid::parse_str(&req.into_inner())
        .map_err(|_| error::ErrorBadRequest("Invalid link format"))?;

    match app_data.data_store.pop(id).await {
        Ok(Some(secret)) => Ok(secret),
        Ok(None) => Err(error::ErrorNotFound("Secret not found")),
        Err(e) => {
            error!("Error retrieving secret: {}", e);
            Err(error::ErrorInternalServerError("Operation failed"))
        }
    }
}

#[get("/secret/{id}")]
async fn get_secret(req: web::Path<String>, app_data: web::Data<AppData>) -> Result<String> {
    get_secret_from_request(req, app_data).await
}

#[post("/secret")]
async fn post_secret(
    http_req: HttpRequest,
    req: web::Json<PostSecretRequest>,
    app_data: web::Data<AppData>,
) -> Result<web::Json<PostSecretResponse>> {
    ensure_is_authorized(&http_req, &app_data.tokens)?;
    ensure_ttl_is_valid(req.expires_in, app_data.max_ttl)?;

    let id = uuid::Uuid::new_v4();

    app_data
        .data_store
        .put(id, req.data.clone(), req.expires_in)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(web::Json(PostSecretResponse { id }))
}

fn ensure_ttl_is_valid(expires_in: Duration, max_ttl: Duration) -> Result<()> {
    if expires_in > max_ttl {
        Err(error::ErrorBadRequest(format!(
            "TTL exceeds maximum allowed duration of {} seconds",
            max_ttl.as_secs()
        )))
    } else {
        Ok(())
    }
}

fn ensure_is_authorized(req: &HttpRequest, tokens: &[String]) -> Result<()> {
    if tokens.is_empty() {
        // no tokens required
        return Ok(());
    }

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| error::ErrorUnauthorized("Unauthorized: No token provided"))?
        .trim_start_matches("Bearer ")
        .trim();

    for valid_token in tokens {
        if valid_token.as_bytes().ct_eq(token.as_bytes()).into() {
            return Ok(());
        }
    }

    Err(error::ErrorForbidden("Forbidden: Invalid token"))
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
        async fn pop(&self, _id: Uuid) -> Result<Option<String>, DataStoreError> {
            if self.get_error {
                Err(DataStoreError::InternalError("mock error".to_string()))
            } else {
                Ok(self.get_result.clone().unwrap_or(None))
            }
        }

        async fn put(
            &self,
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
        let app_data = AppData {
            data_store: Box::new(mock_store),
            tokens: vec![],
            max_ttl: Duration::from_secs(7200),
        };

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
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
        let app_data = AppData {
            data_store: Box::new(mock_store),
            tokens: vec![],
            max_ttl: Duration::from_secs(7200),
        };

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
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
        let app_data = AppData {
            data_store: Box::new(mock_store),
            tokens: vec![],
            max_ttl: Duration::from_secs(7200),
        };

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
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
        let app_data = AppData {
            data_store: Box::new(mock_store),
            tokens: vec![],
            max_ttl: Duration::from_secs(7200),
        };

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
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
        let app_data = AppData {
            data_store: Box::new(mock_store),
            tokens: vec![],
            max_ttl: Duration::from_secs(7200),
        };

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
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

    #[actix_web::test]
    async fn test_post_secret_with_valid_token() {
        let mock_store = MockDataStore::new();
        let stored_data = mock_store.stored_data.clone();
        let app_data = AppData {
            data_store: Box::new(mock_store),
            tokens: vec!["valid_token_123".to_string()],
            max_ttl: Duration::from_secs(7200),
        };

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let payload = PostSecretRequest {
            data: "test_secret".to_string(),
            expires_in: Duration::from_secs(3600),
        };

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("Authorization", "Bearer valid_token_123"))
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: PostSecretResponse = test::read_body_json(resp).await;
        assert!(!body.id.is_nil());

        let stored = stored_data.lock().unwrap();
        assert_eq!(stored.len(), 1);
    }

    #[actix_web::test]
    async fn test_post_secret_missing_auth_header() {
        let mock_store = MockDataStore::new();
        let app_data = AppData {
            data_store: Box::new(mock_store),
            tokens: vec!["valid_token_123".to_string()],
            max_ttl: Duration::from_secs(7200),
        };

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
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
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_post_secret_invalid_token() {
        let mock_store = MockDataStore::new();
        let app_data = AppData {
            data_store: Box::new(mock_store),
            tokens: vec!["valid_token_123".to_string()],
            max_ttl: Duration::from_secs(7200),
        };

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let payload = PostSecretRequest {
            data: "test_secret".to_string(),
            expires_in: Duration::from_secs(3600),
        };

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("Authorization", "Bearer invalid_token"))
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);
    }

    #[actix_web::test]
    async fn test_post_secret_invalid_ttl() {
        let mock_store = MockDataStore::new();

        let max_ttl = Duration::from_secs(30);
        let app_data = AppData {
            data_store: Box::new(mock_store),
            tokens: vec![],
            max_ttl,
        };

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let payload = PostSecretRequest {
            data: "test_secret".to_string(),
            expires_in: Duration::from_secs(max_ttl.as_secs() + 1),
        };

        let req = test::TestRequest::post()
            .uri("/secret")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }
}
