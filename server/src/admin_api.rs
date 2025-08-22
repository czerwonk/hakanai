// SPDX-License-Identifier: Apache-2.0

//! Admin API endpoints for token management.
//!
//! Provides REST endpoints for administrative operations like creating user tokens.
//! All endpoints require admin token authentication.

use std::time::Duration;

use actix_web::{HttpResponse, Result, web};
use tracing::info;

use hakanai_lib::models::{CreateTokenRequest, CreateTokenResponse};

use crate::admin_user::AdminUser;
use crate::app_data::AppData;
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

    let token_data = TokenData {
        upload_size_limit: request.upload_size_limit,
    };

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
    use actix_web::{App, test, web};
    use std::time::Duration;

    use crate::app_data::{AnonymousOptions, AppData};
    use crate::observer::ObserverManager;
    use crate::test_utils::{MockDataStore, MockTokenManager};

    fn create_test_app_data(token_manager: MockTokenManager) -> AppData {
        AppData {
            data_store: Box::new(MockDataStore::new()),
            token_validator: Box::new(token_manager.clone()),
            token_creator: Box::new(token_manager),
            max_ttl: Duration::from_secs(7200),
            anonymous_usage: AnonymousOptions {
                allowed: true,
                upload_size_limit: 32 * 1024,
            },
            impressum_html: None,
            privacy_html: None,
            observer_manager: ObserverManager::new(),
            show_token_input: false,
            trusted_ip_ranges: None,
            trusted_ip_header: "x-forwarded-for".to_string(),
        }
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
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
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
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer invalid_admin_token"))
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
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "admin_token")) // Missing "Bearer " prefix
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
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);

        let response: serde_json::Value = test::read_body_json(resp).await;
        assert!(
            response["error"]
                .as_str()
                .unwrap()
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
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let response: CreateTokenResponse = test::read_body_json(resp).await;
        assert_eq!(response.token, "minimal_token");
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
        };

        let req = test::TestRequest::post()
            .uri("/api/v1/admin/tokens")
            .insert_header(("Authorization", "Bearer admin_token"))
            .set_json(&request_body)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let response: CreateTokenResponse = test::read_body_json(resp).await;
        assert_eq!(response.token, "long_lived_token");
    }
}
