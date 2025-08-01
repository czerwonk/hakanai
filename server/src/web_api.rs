use core::option::Option;
use std::time::Duration;

use actix_web::{HttpRequest, Result, error, get, post, web};
use tracing::{Span, error, instrument};
use uuid::Uuid;

use hakanai_lib::models::{PostSecretRequest, PostSecretResponse};

use crate::app_data::AppData;
use crate::data_store::DataStorePopResult;
use crate::token::{TokenData, TokenError};

/// Configures the Actix Web services for the application.
///
/// This function registers the API routes and sets up the application data,
/// including the data store that will be shared across all handlers.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_secret).service(post_secret);
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
#[instrument(skip(app_data), fields(id = tracing::field::Empty), err)]
pub async fn get_secret_from_request(
    req: web::Path<String>,
    app_data: web::Data<AppData>,
) -> Result<String> {
    let id = Uuid::parse_str(&req.into_inner())
        .map_err(|_| error::ErrorBadRequest("Invalid link format"))?;
    Span::current().record("id", id.to_string());

    match app_data.data_store.pop(id).await {
        Ok(res) => match res {
            DataStorePopResult::Found(secret) => Ok(secret),
            DataStorePopResult::NotFound => Err(error::ErrorNotFound("Secret not found")),
            DataStorePopResult::AlreadyAccessed => {
                Err(error::ErrorGone("Secret was already accessed"))
            }
        },
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
#[instrument(skip(req, app_data, http_req), err)]
async fn post_secret(
    http_req: HttpRequest,
    req: web::Json<PostSecretRequest>,
    app_data: web::Data<AppData>,
) -> Result<web::Json<PostSecretResponse>> {
    let token_data = authorize_request(&http_req, &app_data).await?;
    enforce_size_limit(&req, token_data, &app_data)?;
    ensure_ttl_is_valid(req.expires_in, app_data.max_ttl)?;

    let id = uuid::Uuid::new_v4();

    app_data
        .data_store
        .put(id, req.data.clone(), req.expires_in)
        .await
        .map_err(error::ErrorInternalServerError)?;

    Ok(web::Json(PostSecretResponse { id }))
}

#[instrument(skip(req, app_data), err)]
async fn authorize_request(
    req: &HttpRequest,
    app_data: &web::Data<AppData>,
) -> Result<Option<TokenData>> {
    let token_header = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    if let Some(mut token) = token_header {
        token = token.trim_start_matches("Bearer ").trim();

        match app_data.token_validator.validate_user_token(token).await {
            Ok(token_data) => {
                return Ok(Some(token_data));
            }
            Err(TokenError::InvalidToken) => {
                return Err(error::ErrorForbidden("Invalid token"));
            }
            Err(e) => {
                error!("Token validation failed: {}", e);
                return Err(error::ErrorInternalServerError("Token validation failed"));
            }
        }
    }

    if app_data.anonymous_usage.allowed {
        Ok(None)
    } else {
        Err(error::ErrorUnauthorized("Authorization token required"))
    }
}

#[instrument]
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

#[instrument(skip(req, app_data), err)]
fn enforce_size_limit(
    req: &web::Json<PostSecretRequest>,
    token_data: Option<TokenData>,
    app_data: &web::Data<AppData>,
) -> Result<()> {
    let size = req.data.len();

    if let Some(token_data) = token_data {
        if let Some(limit) = token_data.upload_size_limit {
            if size > limit as usize {
                return Err(error::ErrorPayloadTooLarge(
                    "Upload size limit exceeded for user token",
                ));
            }
        }
    } else if size > app_data.anonymous_usage.upload_size_limit as usize {
        return Err(error::ErrorPayloadTooLarge(
            "Upload size limit exceeded for anonymous usage",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test};
    use std::time::Duration;

    use crate::app_data::AnonymousOptions;
    use crate::data_store::DataStore;
    use crate::test_utils::{MockDataStore, MockTokenManager};
    use crate::token::TokenData;

    // Helper function to create test AppData with default values
    fn create_test_app_data(
        data_store: Box<dyn DataStore>,
        token_manager: MockTokenManager,
        allow_anonymous: bool,
    ) -> AppData {
        AppData {
            data_store,
            token_validator: Box::new(token_manager.clone()),
            token_creator: Box::new(token_manager),
            max_ttl: Duration::from_secs(7200),
            anonymous_usage: AnonymousOptions {
                allowed: allow_anonymous,
                upload_size_limit: 32 * 1024, // 32KB in bytes
            },
            impressum_html: None,
            privacy_html: None,
        }
    }

    #[actix_web::test]
    async fn test_get_secret_found() {
        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()));
        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

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
        let mock_store = MockDataStore::new().with_pop_result(DataStorePopResult::NotFound);
        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

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
    async fn test_get_secret_already_accessed() {
        let mock_store = MockDataStore::new().with_pop_result(DataStorePopResult::AlreadyAccessed);
        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

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
        assert_eq!(resp.status(), 410);
    }

    #[actix_web::test]
    async fn test_get_secret_error() {
        let mock_store = MockDataStore::new().with_get_error();
        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

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
        let app_data = create_test_app_data(
            Box::new(mock_store.clone()),
            MockTokenManager::new(),
            true, // Allow anonymous
        );

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

        let put_ops = mock_store.get_put_operations();
        assert_eq!(put_ops.len(), 1);
        assert_eq!(put_ops[0].1, "test_secret");
        assert_eq!(put_ops[0].2, Duration::from_secs(3600));
    }

    #[actix_web::test]
    async fn test_post_secret_error() {
        let mock_store = MockDataStore::new().with_put_error();
        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

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
        let token_manager = MockTokenManager::new().with_user_token(
            "valid_token_123",
            TokenData {
                upload_size_limit: None,
            },
        );
        let app_data = create_test_app_data(Box::new(mock_store.clone()), token_manager, true);

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

        let put_ops = mock_store.get_put_operations();
        assert_eq!(put_ops.len(), 1);
    }

    #[actix_web::test]
    async fn test_post_secret_missing_auth_header() {
        let mock_store = MockDataStore::new();
        let app_data = create_test_app_data(
            Box::new(mock_store),
            MockTokenManager::new(),
            false, // Don't allow anonymous
        );

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
        let app_data = create_test_app_data(
            Box::new(mock_store),
            MockTokenManager::new(), // No valid tokens
            true,
        );

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
        let mut app_data =
            create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);
        app_data.max_ttl = max_ttl; // Override the default TTL

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

    #[actix_web::test]
    async fn test_post_secret_anonymous_size_limit_exceeded() {
        let mock_store = MockDataStore::new();
        let app_data = create_test_app_data(
            Box::new(mock_store),
            MockTokenManager::new(),
            true, // Allow anonymous
        );

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        // Create a payload larger than 32KB anonymous limit
        let large_data = "x".repeat(33 * 1024); // 33KB
        let payload = PostSecretRequest {
            data: large_data,
            expires_in: Duration::from_secs(3600),
        };

        let req = test::TestRequest::post()
            .uri("/secret")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 413); // Payload Too Large
    }

    #[actix_web::test]
    async fn test_post_secret_token_size_limit_exceeded() {
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_user_token(
            "limited_token",
            TokenData {
                upload_size_limit: Some(1024), // 1KB limit
            },
        );
        let app_data = create_test_app_data(Box::new(mock_store), token_manager, true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        // Create a payload larger than 1KB token limit
        let large_data = "x".repeat(2048); // 2KB
        let payload = PostSecretRequest {
            data: large_data,
            expires_in: Duration::from_secs(3600),
        };

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("Authorization", "Bearer limited_token"))
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 413); // Payload Too Large
    }

    #[actix_web::test]
    async fn test_post_secret_anonymous_access_denied() {
        let mock_store = MockDataStore::new();
        let app_data = create_test_app_data(
            Box::new(mock_store),
            MockTokenManager::new(),
            false, // Don't allow anonymous
        );

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
        assert_eq!(resp.status(), 401); // Unauthorized
    }
}
