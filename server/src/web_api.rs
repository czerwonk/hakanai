// SPDX-License-Identifier: Apache-2.0

use core::option::Option;
use std::time::Duration;

use actix_web::{HttpRequest, Result, error, get, post, web};
use tracing::{Span, error, instrument};
use uuid::Uuid;

use hakanai_lib::models::{
    PostSecretRequest, PostSecretResponse, SecretRestrictions, restrictions,
};

use crate::app_data::AppData;
use crate::data_store::DataStorePopResult;
use crate::filters;
use crate::observer::SecretEventContext;
use crate::size_limited_json::SizeLimitedJson;
use crate::user::User;

/// Configures the Actix Web services for the application.
///
/// This function registers the API routes and sets up the application data,
/// including the data store that will be shared across all handlers.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(get_secret).service(post_secret);
}

#[get("/secret/{id}")]
async fn get_secret(
    http_req: HttpRequest,
    req: web::Path<String>,
    app_data: web::Data<AppData>,
) -> Result<String> {
    get_secret_from_request(http_req, req, app_data).await
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
#[instrument(skip(app_data, http_req), fields(id = tracing::field::Empty, request_id = tracing::field::Empty), err)]
pub async fn get_secret_from_request(
    http_req: HttpRequest,
    req: web::Path<String>,
    app_data: web::Data<AppData>,
) -> Result<String> {
    let id = Uuid::parse_str(&req.into_inner())
        .map_err(|_| error::ErrorBadRequest("Invalid link format"))?;
    Span::current().record("id", id.to_string());

    if let Some(request_id) = extract_request_id(&http_req) {
        Span::current().record("request_id", request_id);
    }

    verify_restrictions_for_secret(id, &http_req, &app_data).await?;

    match app_data.data_store.pop(id).await {
        Ok(res) => match res {
            DataStorePopResult::Found(secret) => {
                app_data
                    .observer_manager
                    .notify_secret_retrieved(
                        id,
                        &SecretEventContext::new(http_req.headers().clone()),
                    )
                    .await;
                Ok(secret)
            }
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

#[instrument(skip(app_data, http_req), err)]
async fn verify_restrictions_for_secret(
    id: Uuid,
    http_req: &HttpRequest,
    app_data: &AppData,
) -> Result<()> {
    let restrictions = app_data
        .data_store
        .get_restrictions(id)
        .await
        .map_err(|e| {
            error!("Failed to retrieve restrictions for secret {id}: {e}");
            error::ErrorInternalServerError("Operation failed")
        })?;

    // Check IP restrictions if they exist
    if let Some(restrictions) = restrictions {
        ensure_restrictions(restrictions, http_req, app_data)?;
    }

    Ok(())
}

fn ensure_restrictions(
    restrictions: SecretRestrictions,
    http_req: &HttpRequest,
    app_data: &AppData,
) -> Result<()> {
    if let Some(allowed_ips) = restrictions.allowed_ips
        && !allowed_ips.is_empty()
        && !filters::is_request_from_ip_range(http_req, app_data, &allowed_ips)
    {
        return Err(error::ErrorForbidden("Not allowed to access the secret"));
    }

    if let Some(allowed_countries) = restrictions.allowed_countries
        && !allowed_countries.is_empty()
        && !filters::is_request_from_country(http_req, app_data, &allowed_countries)
    {
        return Err(error::ErrorForbidden("Not allowed to access the secret"));
    }

    if let Some(allowed_asns) = restrictions.allowed_asns
        && !allowed_asns.is_empty()
        && !filters::is_request_from_asn(http_req, app_data, &allowed_asns)
    {
        return Err(error::ErrorForbidden("Not allowed to access the secret"));
    }

    if let Some(passphrase_hash) = restrictions.passphrase_hash
        && !passphrase_hash.is_empty()
    {
        let value = filters::extract_header_value(http_req, restrictions::PASSPHRASE_HEADER_NAME)
            .ok_or_else(|| {
            error::ErrorUnauthorized("Missing required passphrase to access the secret")
        })?;

        if value != passphrase_hash {
            return Err(error::ErrorUnauthorized("Not allowed to access the secret"));
        }
    }

    Ok(())
}

#[post("/secret")]
#[instrument(skip(req, app_data, http_req, user), fields(request_id = tracing::field::Empty, user_type = tracing::field::Empty), err)]
async fn post_secret(
    http_req: HttpRequest,
    req: SizeLimitedJson<PostSecretRequest>,
    user: User, // This ensures authentication/authorization happens
    app_data: web::Data<AppData>,
) -> Result<web::Json<PostSecretResponse>> {
    if let Some(request_id) = extract_request_id(&http_req) {
        Span::current().record("request_id", request_id);
    }
    Span::current().record("user_type", user.user_type.to_string());

    let req = req.into_inner();
    ensure_ttl_is_valid(req.expires_in, app_data.max_ttl)?;

    if let Some(ref restrictions) = req.restrictions {
        ensure_restrictions_are_supported(restrictions, &app_data)?;
    }

    let id = uuid::Uuid::new_v4();
    let mut ctx = SecretEventContext::new(http_req.headers().clone())
        .with_user_type(user.user_type)
        .with_ttl(req.expires_in);

    if let Some(ref restrictions) = req.restrictions {
        app_data
            .data_store
            .set_restrictions(id, restrictions, req.expires_in)
            .await
            .map_err(|e| {
                error!("Failed to set restrictions for secret {id}: {e}");
                error::ErrorInternalServerError("Operation failed")
            })?;
        ctx = ctx.with_restrictions(restrictions.clone());
    }

    app_data
        .data_store
        .put(id, req.data.clone(), req.expires_in)
        .await
        .map_err(|e| {
            error!("Error while creating secret: {e}");
            error::ErrorInternalServerError("Operation failed")
        })?;

    app_data
        .observer_manager
        .notify_secret_created(id, &ctx)
        .await;

    Ok(web::Json(PostSecretResponse { id }))
}

fn ensure_restrictions_are_supported(
    restrictions: &SecretRestrictions,
    app_data: &AppData,
) -> Result<()> {
    if restrictions.allowed_countries.is_some() && app_data.country_header.is_none() {
        return Err(error::ErrorNotImplemented(
            "Country restrictions are not supported by the server",
        ));
    }

    if restrictions.allowed_asns.is_some() && app_data.asn_header.is_none() {
        return Err(error::ErrorNotImplemented(
            "ASN restrictions are not supported by the server",
        ));
    }

    Ok(())
}

/// Extracts and validates the X-Request-Id header from the request.
/// Only accepts valid UUID v4 format to prevent log injection.
fn extract_request_id(http_req: &HttpRequest) -> Option<String> {
    http_req
        .headers()
        .get("x-request-id")
        .and_then(|header_value| header_value.to_str().ok())
        .filter(|request_id| Uuid::parse_str(request_id).is_ok())
        .map(|s| s.to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use actix_web::http::header::HeaderMap;
    use actix_web::{App, test};
    use async_trait::async_trait;

    use crate::app_data::AnonymousOptions;
    use crate::data_store::DataStore;
    use crate::observer::SecretObserver;
    use crate::test_utils::{MockDataStore, MockTokenManager};
    use crate::token::TokenData;

    use hakanai_lib::models::SecretRestrictions;

    // Mock observer for testing
    #[derive(Clone)]
    struct MockObserver {
        created_events: Arc<Mutex<Vec<(Uuid, HeaderMap)>>>,
        retrieved_events: Arc<Mutex<Vec<(Uuid, HeaderMap)>>>,
    }

    impl MockObserver {
        fn new() -> Self {
            MockObserver {
                created_events: Arc::new(Mutex::new(Vec::new())),
                retrieved_events: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_created_events(&self) -> Vec<(Uuid, HeaderMap)> {
            self.created_events.lock().unwrap().clone()
        }

        fn get_retrieved_events(&self) -> Vec<(Uuid, HeaderMap)> {
            self.retrieved_events.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl SecretObserver for MockObserver {
        async fn on_secret_created(&self, secret_id: Uuid, context: &SecretEventContext) {
            self.created_events
                .lock()
                .unwrap()
                .push((secret_id, context.headers.clone()));
        }

        async fn on_secret_retrieved(&self, secret_id: Uuid, context: &SecretEventContext) {
            self.retrieved_events
                .lock()
                .unwrap()
                .push((secret_id, context.headers.clone()));
        }
    }

    // Helper function to create test AppData with default values
    fn create_test_app_data(
        data_store: Box<dyn DataStore>,
        token_manager: MockTokenManager,
        allow_anonymous: bool,
    ) -> AppData {
        AppData::default()
            .with_data_store(data_store)
            .with_token_validator(Box::new(token_manager.clone()))
            .with_token_creator(Box::new(token_manager))
            .with_max_ttl(Duration::from_secs(7200))
            .with_anonymous_usage(AnonymousOptions {
                allowed: allow_anonymous,
                upload_size_limit: 32 * 1024, // 32KB in bytes
            })
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

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600));

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

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600));

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

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600));

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

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600));

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

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600));

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

        let payload = PostSecretRequest::new(
            "test_secret".to_string(),
            Duration::from_secs(max_ttl.as_secs() + 1),
        );

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

        // Create a payload larger than 32KB * 1.5 = 48KB effective limit
        let large_data = "x".repeat(50 * 1024); // 50KB (exceeds 48KB effective limit)
        let payload = PostSecretRequest::new(large_data, Duration::from_secs(3600));

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

        // Create a payload larger than 1KB * 1.5 = 1.5KB effective limit
        let large_data = "x".repeat(2048); // 2KB (exceeds 1.5KB effective limit)
        let payload = PostSecretRequest::new(large_data, Duration::from_secs(3600));

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

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600));

        let req = test::TestRequest::post()
            .uri("/secret")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401); // Unauthorized
    }

    #[actix_web::test]
    async fn test_observer_notification_on_secret_creation() {
        let mock_store = MockDataStore::new();
        let mock_observer = MockObserver::new();
        let observer_clone = mock_observer.clone();

        let mut app_data = create_test_app_data(
            Box::new(mock_store.clone()),
            MockTokenManager::new(),
            true, // Allow anonymous
        );
        app_data
            .observer_manager
            .register_observer(Box::new(mock_observer));

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600));

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("User-Agent", "test-agent"))
            .insert_header(("X-Request-Id", "test-123"))
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: PostSecretResponse = test::read_body_json(resp).await;

        // Verify observer was notified
        let created_events = observer_clone.get_created_events();
        assert_eq!(created_events.len(), 1);
        assert_eq!(created_events[0].0, body.id);

        // Verify headers were passed
        let headers = &created_events[0].1;
        assert_eq!(headers.get("user-agent").unwrap(), "test-agent");
        assert_eq!(headers.get("x-request-id").unwrap(), "test-123");
    }

    #[actix_web::test]
    async fn test_observer_notification_on_secret_retrieval() {
        let secret_id = Uuid::new_v4();
        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()));
        let mock_observer = MockObserver::new();
        let observer_clone = mock_observer.clone();

        let mut app_data =
            create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);
        app_data
            .observer_manager
            .register_observer(Box::new(mock_observer));

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{secret_id}"))
            .insert_header(("User-Agent", "hakanai-cli"))
            .insert_header(("X-Forwarded-For", "192.168.1.1"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Verify observer was notified
        let retrieved_events = observer_clone.get_retrieved_events();
        assert_eq!(retrieved_events.len(), 1);
        assert_eq!(retrieved_events[0].0, secret_id);

        let headers = &retrieved_events[0].1;
        assert_eq!(headers.get("user-agent").unwrap(), "hakanai-cli");
        assert_eq!(headers.get("x-forwarded-for").unwrap(), "192.168.1.1");
    }

    #[actix_web::test]
    async fn test_observer_notification_with_token() {
        let mock_store = MockDataStore::new();
        let mock_observer = MockObserver::new();
        let observer_clone = mock_observer.clone();
        let token_manager = MockTokenManager::new().with_user_token(
            "valid_token",
            TokenData {
                upload_size_limit: None,
            },
        );

        let mut app_data = create_test_app_data(Box::new(mock_store.clone()), token_manager, true);
        app_data
            .observer_manager
            .register_observer(Box::new(mock_observer));

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600));

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("Authorization", "Bearer valid_token"))
            .insert_header(("User-Agent", "authenticated-client"))
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: PostSecretResponse = test::read_body_json(resp).await;

        // Verify observer was notified with auth headers
        let created_events = observer_clone.get_created_events();
        assert_eq!(created_events.len(), 1);
        assert_eq!(created_events[0].0, body.id);

        let headers = &created_events[0].1;
        assert_eq!(headers.get("authorization").unwrap(), "Bearer valid_token");
        assert_eq!(headers.get("user-agent").unwrap(), "authenticated-client");
    }

    #[actix_web::test]
    async fn test_get_secret_with_ip_restriction_allowed() {
        // Create a secret with IP restrictions that allow the request
        let secret_id = Uuid::new_v4();
        let allowed_ips = vec![
            "192.168.1.0/24".parse::<ipnet::IpNet>().unwrap(),
            "10.0.0.0/8".parse::<ipnet::IpNet>().unwrap(),
        ];

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()))
            .with_restrictions(
                secret_id,
                SecretRestrictions::default().with_allowed_ips(allowed_ips),
            );

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header(("x-forwarded-for", "192.168.1.100"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        assert_eq!(body, "test_secret");
    }

    #[actix_web::test]
    async fn test_get_secret_with_ip_restriction_blocked() {
        // Create a secret with IP restrictions that block the request
        let secret_id = Uuid::new_v4();
        let allowed_ips = vec!["192.168.1.0/24".parse::<ipnet::IpNet>().unwrap()];

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()))
            .with_restrictions(
                secret_id,
                SecretRestrictions::default().with_allowed_ips(allowed_ips),
            );

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        // Request from IP not in allowed range
        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header(("x-forwarded-for", "10.0.0.50"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403); // Forbidden
    }

    #[actix_web::test]
    async fn test_get_secret_with_no_ip_restrictions() {
        // Create a secret without IP restrictions - should be accessible from any IP
        let secret_id = Uuid::new_v4();

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()));
        // No IP restrictions set

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header(("x-forwarded-for", "123.45.67.89"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body = test::read_body(resp).await;
        assert_eq!(body, "test_secret");
    }

    #[actix_web::test]
    async fn test_get_secret_with_ipv6_restriction() {
        // Test IPv6 address restrictions
        let secret_id = Uuid::new_v4();
        let allowed_ips = vec!["2001:db8::/32".parse::<ipnet::IpNet>().unwrap()];

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()))
            .with_restrictions(
                secret_id,
                SecretRestrictions::default().with_allowed_ips(allowed_ips),
            );

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        // Request from allowed IPv6 range
        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header(("x-forwarded-for", "2001:db8::1234"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_get_secret_with_multiple_ip_restrictions() {
        // Test with multiple IP ranges allowed
        let secret_id = Uuid::new_v4();
        let allowed_ips = vec![
            "192.168.1.0/24".parse::<ipnet::IpNet>().unwrap(),
            "10.0.0.0/8".parse::<ipnet::IpNet>().unwrap(),
            "172.16.0.0/12".parse::<ipnet::IpNet>().unwrap(),
        ];

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()))
            .with_restrictions(
                secret_id,
                SecretRestrictions::default().with_allowed_ips(allowed_ips),
            );

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        // Test from second allowed range
        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header(("x-forwarded-for", "10.1.2.3"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_get_secret_with_single_ip_restriction() {
        // Test restriction to a single IP address (using /32 for IPv4)
        let secret_id = Uuid::new_v4();
        let allowed_ips = vec!["192.168.1.100/32".parse::<ipnet::IpNet>().unwrap()];

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()))
            .with_restrictions(
                secret_id,
                SecretRestrictions::default().with_allowed_ips(allowed_ips),
            );

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        // Request from exact IP - should succeed
        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header(("x-forwarded-for", "192.168.1.100"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Request from different IP - should fail
        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header(("x-forwarded-for", "192.168.1.101"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);
    }

    #[actix_web::test]
    async fn test_post_secret_with_ip_restrictions() {
        // Test that POST endpoint properly stores IP restrictions
        let mock_store = MockDataStore::new();
        let app_data =
            create_test_app_data(Box::new(mock_store.clone()), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let allowed_ips = vec![
            "192.168.1.0/24".parse::<ipnet::IpNet>().unwrap(),
            "10.0.0.0/8".parse::<ipnet::IpNet>().unwrap(),
        ];

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600))
            .with_restrictions(SecretRestrictions::default().with_allowed_ips(allowed_ips.clone()));

        let req = test::TestRequest::post()
            .uri("/secret")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: PostSecretResponse = test::read_body_json(resp).await;
        assert!(!body.id.is_nil());

        // Verify restrictions were stored
        let restrictions = mock_store.get_restrictions();
        assert!(restrictions.contains_key(&body.id.to_string()));
        assert_eq!(
            restrictions[&body.id.to_string()].allowed_ips,
            Some(allowed_ips)
        );
    }

    #[actix_web::test]
    async fn test_post_secret_without_ip_restrictions() {
        // Test that POST endpoint works without IP restrictions
        let mock_store = MockDataStore::new();
        let app_data =
            create_test_app_data(Box::new(mock_store.clone()), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let payload = PostSecretRequest::new("test_secret".to_string(), Duration::from_secs(3600));
        // No IP restrictions

        let req = test::TestRequest::post()
            .uri("/secret")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: PostSecretResponse = test::read_body_json(resp).await;
        assert!(!body.id.is_nil());

        // Verify no IP restrictions were stored
        let restrictions = mock_store.get_restrictions();
        assert!(!restrictions.contains_key(&body.id.to_string()));
    }

    #[actix_web::test]
    async fn test_get_secret_ip_restriction_with_proxy_chain() {
        // Test IP extraction from proxy chain (multiple IPs in x-forwarded-for)
        let secret_id = Uuid::new_v4();
        let allowed_ips = vec!["192.168.1.0/24".parse::<ipnet::IpNet>().unwrap()];

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()))
            .with_restrictions(
                secret_id,
                SecretRestrictions::default().with_allowed_ips(allowed_ips),
            );

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        // x-forwarded-for with multiple IPs (first one is the client)
        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header(("x-forwarded-for", "192.168.1.50, 10.0.0.1, 172.16.0.1"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200); // First IP is in allowed range
    }

    #[actix_web::test]
    async fn test_get_secret_with_empty_ip_restrictions() {
        // Test that empty IP restrictions array means no access allowed
        let secret_id = Uuid::new_v4();
        let allowed_ips = vec![]; // Empty restrictions

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found("test_secret".to_string()))
            .with_restrictions(
                secret_id,
                SecretRestrictions::default().with_allowed_ips(allowed_ips),
            );

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header(("x-forwarded-for", "192.168.1.100"))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200); // Empty array is treated as no restrictions in ensure_ip_allowed_to_access_secret
    }

    // Tests for support verification functionality
    #[actix_web::test]
    async fn test_post_secret_with_country_restriction_support_enabled() {
        // Test that country restrictions work when country header is configured
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_unlimited_user_tokens(&["valid-token"]);

        // Create app data with country header configured (support enabled)
        let mut app_data = create_test_app_data(Box::new(mock_store), token_manager, false);
        app_data.country_header = Some("cf-ipcountry".to_string());

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("content-type", "application/json"))
            .insert_header(("authorization", "Bearer valid-token"))
            .set_json(&serde_json::json!({
                "data": "dGVzdF9zZWNyZXQ=",
                "expires_in": 3600,
                "restrictions": {
                    "allowed_countries": ["US", "DE", "CA"]
                }
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200); // Should succeed when country support is enabled
    }

    #[actix_web::test]
    async fn test_post_secret_with_country_restriction_support_disabled() {
        // Test that country restrictions return 501 when country header is not configured
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_unlimited_user_tokens(&["valid-token"]);

        // Create app data WITHOUT country header configured (support disabled)
        let app_data = create_test_app_data(Box::new(mock_store), token_manager, false);
        // Ensure country_header is None (default)
        assert!(app_data.country_header.is_none());

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("content-type", "application/json"))
            .insert_header(("authorization", "Bearer valid-token"))
            .set_json(&serde_json::json!({
                "data": "dGVzdF9zZWNyZXQ=",
                "expires_in": 3600,
                "restrictions": {
                    "allowed_countries": ["US"]
                }
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 501); // Should return Not Implemented

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("Country restrictions are not supported by the server"));
    }

    #[actix_web::test]
    async fn test_post_secret_with_combined_restrictions_support_disabled() {
        // Test that combined IP + country restrictions fail when country support is disabled
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_unlimited_user_tokens(&["valid-token"]);

        let app_data = create_test_app_data(Box::new(mock_store), token_manager, false);
        assert!(app_data.country_header.is_none());

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("content-type", "application/json"))
            .insert_header(("authorization", "Bearer valid-token"))
            .set_json(&serde_json::json!({
                "data": "dGVzdF9zZWNyZXQ=",
                "expires_in": 3600,
                "restrictions": {
                    "allowed_ips": ["192.168.1.0/24"],
                    "allowed_countries": ["US", "DE"]
                }
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 501); // Should fail because country restrictions are not supported
    }

    #[actix_web::test]
    async fn test_post_secret_with_combined_restrictions_support_enabled() {
        // Test that combined IP + country restrictions work when country support is enabled
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_unlimited_user_tokens(&["valid-token"]);

        let mut app_data = create_test_app_data(Box::new(mock_store), token_manager, false);
        app_data.country_header = Some("cf-ipcountry".to_string());

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("content-type", "application/json"))
            .insert_header(("authorization", "Bearer valid-token"))
            .set_json(&serde_json::json!({
                "data": "dGVzdF9zZWNyZXQ=",
                "expires_in": 3600,
                "restrictions": {
                    "allowed_ips": ["192.168.1.0/24", "10.0.0.1"],
                    "allowed_countries": ["US", "DE", "CA"]
                }
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200); // Should succeed when both IP and country support are enabled
    }

    #[actix_web::test]
    async fn test_post_secret_with_only_ip_restrictions_no_country_support() {
        // Test that IP-only restrictions work even when country support is disabled
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_unlimited_user_tokens(&["valid-token"]);

        let app_data = create_test_app_data(Box::new(mock_store), token_manager, false);
        assert!(app_data.country_header.is_none());

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("content-type", "application/json"))
            .insert_header(("authorization", "Bearer valid-token"))
            .set_json(&serde_json::json!({
                "data": "dGVzdF9zZWNyZXQ=",
                "expires_in": 3600,
                "restrictions": {
                    "allowed_ips": ["192.168.1.0/24", "10.0.0.1"]
                }
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200); // Should succeed - IP restrictions work without country support
    }

    #[actix_web::test]
    async fn test_post_secret_without_restrictions() {
        // Test that secrets without restrictions work regardless of country support configuration
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_unlimited_user_tokens(&["valid-token"]);

        let app_data = create_test_app_data(Box::new(mock_store), token_manager, false);
        assert!(app_data.country_header.is_none());

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("content-type", "application/json"))
            .insert_header(("authorization", "Bearer valid-token"))
            .set_json(&serde_json::json!({
                "data": "dGVzdF9zZWNyZXQ=",
                "expires_in": 3600
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200); // Should always succeed without restrictions
    }

    #[actix_web::test]
    async fn test_post_secret_with_empty_country_restrictions() {
        // Test that empty country restrictions array doesn't trigger support check
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_unlimited_user_tokens(&["valid-token"]);

        let app_data = create_test_app_data(Box::new(mock_store), token_manager, false);
        assert!(app_data.country_header.is_none());

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("content-type", "application/json"))
            .insert_header(("authorization", "Bearer valid-token"))
            .set_json(&serde_json::json!({
                "data": "dGVzdF9zZWNyZXQ=",
                "expires_in": 3600,
                "restrictions": {
                    "allowed_countries": []
                }
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 501); // Empty array still triggers support check since field is present
    }

    #[actix_web::test]
    async fn test_post_secret_country_support_with_custom_header() {
        // Test that country restrictions work with custom country header configuration
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_unlimited_user_tokens(&["valid-token"]);

        let mut app_data = create_test_app_data(Box::new(mock_store), token_manager, false);
        app_data.country_header = Some("x-country-code".to_string()); // Custom header

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("content-type", "application/json"))
            .insert_header(("authorization", "Bearer valid-token"))
            .set_json(&serde_json::json!({
                "data": "dGVzdF9zZWNyZXQ=",
                "expires_in": 3600,
                "restrictions": {
                    "allowed_countries": ["GB", "FR"]
                }
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200); // Should succeed with custom country header configured
    }

    #[actix_web::test]
    async fn test_post_secret_multiple_country_restrictions_support_enabled() {
        // Test that multiple country restrictions work when support is enabled
        let mock_store = MockDataStore::new();
        let token_manager = MockTokenManager::new().with_unlimited_user_tokens(&["valid-token"]);

        let mut app_data = create_test_app_data(Box::new(mock_store), token_manager, false);
        app_data.country_header = Some("cf-ipcountry".to_string());

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::post()
            .uri("/secret")
            .insert_header(("content-type", "application/json"))
            .insert_header(("authorization", "Bearer valid-token"))
            .set_json(&serde_json::json!({
                "data": "dGVzdF9zZWNyZXQ=",
                "expires_in": 3600,
                "restrictions": {
                    "allowed_countries": ["US", "DE", "CA", "GB", "FR", "JP", "AU"]
                }
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200); // Should succeed with multiple countries
    }

    // Tests for passphrase functionality
    #[actix_web::test]
    async fn test_get_secret_with_correct_passphrase() {
        let secret_id = uuid::Uuid::new_v4();
        let passphrase_hash = "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"; // SHA-256 of "password"

        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some(passphrase_hash.to_string());

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found(
                "passphrase_protected_secret".to_string(),
            ))
            .with_restrictions(secret_id, restrictions);

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header((restrictions::PASSPHRASE_HEADER_NAME, passphrase_hash))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200, "Should succeed with correct passphrase");

        let body = test::read_body(resp).await;
        assert_eq!(body, "passphrase_protected_secret");
    }

    #[actix_web::test]
    async fn test_get_secret_with_wrong_passphrase() {
        let secret_id = uuid::Uuid::new_v4();
        let correct_hash = "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"; // SHA-256 of "password"
        let wrong_hash = "ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f"; // SHA-256 of "secret"

        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some(correct_hash.to_string());

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found(
                "passphrase_protected_secret".to_string(),
            ))
            .with_restrictions(secret_id, restrictions);

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header((restrictions::PASSPHRASE_HEADER_NAME, wrong_hash))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401, "Should return 401 for wrong passphrase");
    }

    #[actix_web::test]
    async fn test_get_secret_missing_required_passphrase() {
        let secret_id = uuid::Uuid::new_v4();
        let passphrase_hash = "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8";

        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some(passphrase_hash.to_string());

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found(
                "passphrase_protected_secret".to_string(),
            ))
            .with_restrictions(secret_id, restrictions);

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            // No passphrase header provided
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            401,
            "Should return 401 when passphrase is missing"
        );
    }

    #[actix_web::test]
    async fn test_get_secret_with_empty_passphrase_hash() {
        let secret_id = uuid::Uuid::new_v4();

        // Empty passphrase hash should be treated as no restriction
        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some("".to_string());

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found(
                "secret_with_empty_hash".to_string(),
            ))
            .with_restrictions(secret_id, restrictions);

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            // No passphrase header needed for empty hash
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            200,
            "Should succeed with empty passphrase hash"
        );

        let body = test::read_body(resp).await;
        assert_eq!(body, "secret_with_empty_hash");
    }

    #[actix_web::test]
    async fn test_get_secret_with_passphrase_and_other_restrictions() {
        let secret_id = uuid::Uuid::new_v4();
        let passphrase_hash = "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8";

        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some(passphrase_hash.to_string());
        // Add IP restriction too
        use std::str::FromStr;
        restrictions.allowed_ips = Some(vec![ipnet::IpNet::from_str("127.0.0.0/8").unwrap()]);

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found(
                "multi_restricted_secret".to_string(),
            ))
            .with_restrictions(secret_id, restrictions);

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header((restrictions::PASSPHRASE_HEADER_NAME, passphrase_hash))
            .insert_header(("x-forwarded-for", "127.0.0.1")) // Ensure IP passes the 127.0.0.0/8 restriction
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            200,
            "Should succeed when all restrictions are satisfied"
        );

        let body = test::read_body(resp).await;
        assert_eq!(body, "multi_restricted_secret");
    }

    #[actix_web::test]
    async fn test_post_secret_with_passphrase_restriction() {
        let mock_store = MockDataStore::new();
        let app_data =
            create_test_app_data(Box::new(mock_store.clone()), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let passphrase_hash = "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8";
        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some(passphrase_hash.to_string());

        let payload =
            PostSecretRequest::new("passphrase_secret".to_string(), Duration::from_secs(3600))
                .with_restrictions(restrictions);

        let req = test::TestRequest::post()
            .uri("/secret")
            .set_json(&payload)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            200,
            "Should create secret with passphrase restriction"
        );

        let body: PostSecretResponse = test::read_body_json(resp).await;
        assert!(!body.id.is_nil(), "Should return valid UUID");

        // Verify the restrictions were stored
        let restrictions_ops = mock_store.get_set_restrictions_operations();
        assert_eq!(
            restrictions_ops.len(),
            1,
            "Should have one set_restrictions operation"
        );
        assert_eq!(
            restrictions_ops[0].0, body.id,
            "Should store restrictions for correct ID"
        );
        assert_eq!(
            restrictions_ops[0].1.passphrase_hash,
            Some(passphrase_hash.to_string()),
            "Should store correct passphrase hash"
        );
    }

    #[actix_web::test]
    async fn test_get_secret_unicode_passphrase() {
        let secret_id = uuid::Uuid::new_v4();
        // Pre-calculated hash for unicode string "パスワード123🔒"
        let unicode_hash = "8c11c547bf7a78f0f6f3e1e67e2b24ef1df0b82e4e3f21e44bb4e8f8e3b5f4a9";

        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some(unicode_hash.to_string());

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found(
                "unicode_protected_secret".to_string(),
            ))
            .with_restrictions(secret_id, restrictions);

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header((restrictions::PASSPHRASE_HEADER_NAME, unicode_hash))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            200,
            "Should succeed with unicode passphrase hash"
        );

        let body = test::read_body(resp).await;
        assert_eq!(body, "unicode_protected_secret");
    }

    #[actix_web::test]
    async fn test_get_secret_case_sensitive_passphrase() {
        let secret_id = uuid::Uuid::new_v4();
        let lowercase_hash = "5e884898da28047151d0e56f8dc6292773603d0d6aabbdd62a11ef721d1542d8"; // "password"
        let uppercase_hash = "E6B87050BDB5543D56C7B06E8C528F73045A0AD81F96AB9B21DF04D9862CB63E"; // "PASSWORD" in uppercase

        let mut restrictions = SecretRestrictions::default();
        restrictions.passphrase_hash = Some(lowercase_hash.to_string());

        let mock_store = MockDataStore::new()
            .with_pop_result(DataStorePopResult::Found(
                "case_sensitive_secret".to_string(),
            ))
            .with_restrictions(secret_id, restrictions);

        let app_data = create_test_app_data(Box::new(mock_store), MockTokenManager::new(), true);

        let app = test::init_service(App::new().app_data(web::Data::new(app_data)).configure(
            |cfg| {
                configure(cfg);
            },
        ))
        .await;

        // Test with uppercase hash (should fail)
        let req = test::TestRequest::get()
            .uri(&format!("/secret/{}", secret_id))
            .insert_header((restrictions::PASSPHRASE_HEADER_NAME, uppercase_hash))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(
            resp.status(),
            401,
            "Should fail with wrong case passphrase hash"
        );
    }
}
