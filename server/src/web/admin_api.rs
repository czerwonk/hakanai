// SPDX-License-Identifier: Apache-2.0

//! Admin API endpoints for token management.
//!
//! Provides REST endpoints for administrative operations like creating user tokens.
//! All endpoints require admin token authentication.

use std::time::Duration;

use actix_web::{HttpResponse, Result, web};
use tracing::info;

use hakanai_lib::models::{CreateTokenRequest, CreateTokenResponse};

use super::admin_user::AdminUser;
use super::app_data::AppData;
use crate::token::TokenData;

/// Configure admin API routes
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/admin").route("/tokens", web::post().to(create_token)));
}

/// Create a new user token
///
/// POST /api/v1/admin/tokens
///
/// Requires admin authentication via Authorization header.
/// Creates a new user token with optional size limit and TTL.
pub async fn create_token(
    admin_user: AdminUser,
    request: web::Json<CreateTokenRequest>,
    app_data: web::Data<AppData>,
) -> Result<HttpResponse> {
    let _ = admin_user; // Ensure admin user is authenticated

    let mut token_data = TokenData::default();
    if let Some(size_limit) = request.upload_size_limit {
        token_data = token_data.with_upload_size_limit(size_limit);
    }
    token_data.one_time = request.one_time;

    let ttl_seconds = request.ttl_seconds;
    let ttl = Duration::from_secs(ttl_seconds);

    let token = match app_data
        .token_creator
        .create_user_token(token_data, ttl)
        .await
    {
        Ok(token) => token,
        Err(e) => {
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": format!("Failed to create token: {}", e)
            })));
        }
    };

    info!("Admin created new user token with TTL: {}s", ttl_seconds);

    let response = CreateTokenResponse { token };

    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    use actix_web::{App, test, web};

    use hakanai_lib::utils::test::MustParse;

    use crate::token::MockTokenManager;
    use crate::web::app_data::{AnonymousOptions, AppData};

    fn create_test_app_data(token_manager: MockTokenManager) -> AppData {
        // Configure with localhost trusted IP for tests
        let trusted_ranges = vec!["127.0.0.0/8".must_parse()];

        AppData::default()
            .with_token_validator(Box::new(token_manager.clone()))
            .with_token_creator(Box::new(token_manager))
            .with_max_ttl(Duration::from_secs(7200))
            .with_anonymous_usage(AnonymousOptions {
                allowed: true,
                upload_size_limit: 32 * 1024,
            })
            .with_trusted_ip_ranges(Some(trusted_ranges))
            .with_trusted_ip_header("x-forwarded-for".to_string())
    }

    #[actix_web::test]
    async fn test_create_token_success() {
        let token_manager = MockTokenManager::new()
            .with_admin_token("admin_token")
            .with_created_token("new_user_token");

        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: Some(1048576), // 1MB
            ttl_seconds: 2592000,             // 30 days
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let response: CreateTokenResponse = test::read_body_json(resp).await;
        assert_eq!(response.token, "new_user_token");
    }

    #[actix_web::test]
    async fn test_create_token_missing_auth_header() {
        let token_manager = MockTokenManager::new();
        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: None,
            ttl_seconds: 3600,
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_create_token_invalid_admin_token() {
        let token_manager = MockTokenManager::new().with_admin_token("valid_admin_token");

        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: Some(2048),
            ttl_seconds: 1800,
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer invalid_admin_token"))
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);
    }

    #[actix_web::test]
    async fn test_create_token_malformed_auth_header() {
        let token_manager = MockTokenManager::new().with_admin_token("admin_token");

        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: None,
            ttl_seconds: 3600,
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "admin_token")) // Missing "Bearer " prefix
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
    }

    #[actix_web::test]
    async fn test_create_token_creation_failure() {
        let token_manager = MockTokenManager::new()
            .with_admin_token("admin_token")
            .with_creation_failure();

        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: Some(5120),
            ttl_seconds: 7200,
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        let response: serde_json::Value = test::read_body_json(resp).await;
        assert!(
            response["error"]
                .as_str()
                .expect("Error message should be a string")
                .contains("Failed to create token")
        );
    }

    #[actix_web::test]
    async fn test_create_token_with_only_ttl() {
        let token_manager = MockTokenManager::new()
            .with_admin_token("admin_token")
            .with_created_token("minimal_token");

        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: None, // No size limit
            ttl_seconds: 900,        // 15 minutes
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let response: CreateTokenResponse = test::read_body_json(resp).await;
        assert_eq!(response.token, "minimal_token");
    }

    #[actix_web::test]
    async fn test_create_token_as_one_time() {
        let token_manager = MockTokenManager::new()
            .with_admin_token("admin_token")
            .with_created_token("one_time_token");

        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: None, // No size limit
            ttl_seconds: 900,        // 15 minutes
            one_time: true,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let response: CreateTokenResponse = test::read_body_json(resp).await;
        assert_eq!(response.token, "one_time_token");
    }

    #[actix_web::test]
    async fn test_create_token_bad_json() {
        let token_manager = MockTokenManager::new().with_admin_token("admin_token");

        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .insert_header(("Content-Type", "application/json"))
            .set_payload(r#"{"invalid": json}"#) // Invalid JSON
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_create_token_missing_required_field() {
        let token_manager = MockTokenManager::new().with_admin_token("admin_token");

        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .set_json(serde_json::json!({
                "upload_size_limit": 1024
                // Missing ttl_seconds field
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_create_token_large_ttl() {
        let token_manager = MockTokenManager::new()
            .with_admin_token("admin_token")
            .with_created_token("long_lived_token");

        let app_data = create_test_app_data(token_manager);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: Some(10 * 1024 * 1024), // 10MB
            ttl_seconds: 365 * 24 * 3600,              // 1 year
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let response: CreateTokenResponse = test::read_body_json(resp).await;
        assert_eq!(response.token, "long_lived_token");
    }

    #[actix_web::test]
    async fn test_admin_api_with_valid_token_but_untrusted_ip() {
        let token_manager = MockTokenManager::new()
            .with_admin_token("admin_token")
            .with_created_token("new_user_token");
        let trusted_ranges = vec!["10.0.0.0/8".must_parse()];

        let app_data = create_test_app_data(token_manager)
            .with_trusted_ip_ranges(Some(trusted_ranges))
            .with_trusted_ip_header("x-forwarded-for".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: Some(1024),
            ttl_seconds: 3600,
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "192.168.1.1")) // Not in trusted range
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Request IP not allowed"));
    }

    #[actix_web::test]
    async fn test_admin_api_no_ip_header_with_trusted_ranges() {
        let token_manager = MockTokenManager::new().with_admin_token("admin_token");
        let trusted_ranges = vec!["10.0.0.0/8".must_parse()];

        let app_data = create_test_app_data(token_manager)
            .with_trusted_ip_ranges(Some(trusted_ranges))
            .with_trusted_ip_header("x-forwarded-for".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: Some(1024),
            ttl_seconds: 3600,
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            // No x-forwarded-for header
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 403);

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains("Request IP not allowed"));
    }

    #[actix_web::test]
    async fn test_admin_api_ipv6_trusted_range() {
        let token_manager = MockTokenManager::new()
            .with_admin_token("admin_token")
            .with_created_token("new_user_token");
        let trusted_ranges = vec!["2001:db8::/32".must_parse()];

        let app_data = create_test_app_data(token_manager)
            .with_trusted_ip_ranges(Some(trusted_ranges))
            .with_trusted_ip_header("x-forwarded-for".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: Some(1024),
            ttl_seconds: 3600,
            one_time: false,
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "2001:db8::1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let response: CreateTokenResponse = test::read_body_json(resp).await;
        assert_eq!(response.token, "new_user_token");
    }

    #[actix_web::test]
    async fn test_admin_api_multiple_trusted_ranges() {
        let token_manager = MockTokenManager::new()
            .with_admin_token("admin_token")
            .with_created_token("new_user_token");
        let trusted_ranges = vec!["10.0.0.0/8".must_parse(), "192.168.1.0/24".must_parse()];

        let app_data = create_test_app_data(token_manager)
            .with_trusted_ip_ranges(Some(trusted_ranges))
            .with_trusted_ip_header("x-forwarded-for".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: Some(1024),
            ttl_seconds: 3600,
            one_time: false,
        };

        // Test first range
        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "10.1.2.3"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Test second range
        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "192.168.1.100"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }

    #[actix_web::test]
    async fn test_admin_api_localhost_trusted() {
        let token_manager = MockTokenManager::new()
            .with_admin_token("admin_token")
            .with_created_token("new_user_token");
        let trusted_ranges = vec!["127.0.0.0/8".must_parse(), "::1/128".must_parse()];

        let app_data = create_test_app_data(token_manager)
            .with_trusted_ip_ranges(Some(trusted_ranges))
            .with_trusted_ip_header("x-forwarded-for".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .service(web::scope("/api/v1").configure(configure_routes)),
        )
        .await;

        let request_body = CreateTokenRequest {
            upload_size_limit: Some(1024),
            ttl_seconds: 3600,
            one_time: false,
        };

        // Test IPv4 localhost
        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "127.0.0.1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        // Test IPv6 localhost
        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .insert_header(("x-forwarded-for", "::1"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);
    }
}
