// SPDX-License-Identifier: Apache-2.0

use actix_web::http::header;
use actix_web::{HttpResponse, Responder, web};

const DEFAULT_CACHE_MAX_AGE: u64 = 604800; // 7 days
const VOLATILE_CACHE_MAX_AGE: u64 = 86400; // 1 day
const HIGHLY_VOLATILE_CACHE_MAX_AGE: u64 = 300; // 5 minutes

/// Configures the Actix Web services for the application.
///
/// This function registers the API routes and sets up the application data,
/// including the data store that will be shared across all handlers.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(serve_index))
        .route("/banner.svg", web::get().to(serve_banner))
        .route("/config.json", web::get().to(serve_config))
        .route("/create", web::get().to(serve_create_secret_html))
        .route("/create-secret.js", web::get().to(serve_create_secret_js))
        .route("/common.js", web::get().to(serve_common_js))
        .route("/docs", web::get().to(serve_docs_html))
        .route("/get", web::get().to(serve_get_secret_html))
        .route("/get-secret.js", web::get().to(serve_get_secret_js))
        .route("/hakanai-client.js", web::get().to(serve_js_client))
        .route("/icon.svg", web::get().to(serve_icon))
        .route("/impressum", web::get().to(serve_impressum))
        .route("/logo.svg", web::get().to(serve_logo))
        .route("/manifest.json", web::get().to(serve_manifest))
        .route("/openapi.json", web::get().to(serve_openapi_json))
        .route("/privacy", web::get().to(serve_privacy))
        .route("/robots.txt", web::get().to(serve_robots_txt))
        .route("/share", web::get().to(serve_share_html))
        .route("/share.js", web::get().to(serve_share_js))
        .route("/share.shortcut", web::get().to(serve_shortcut))
        .route("/style.css", web::get().to(serve_css))
        .route("/hakanai_wasm.js", web::get().to(serve_wasm_js))
        .route("/hakanai_wasm_bg.wasm", web::get().to(serve_wasm_binary));
}

fn serve_with_caching_header(content: &[u8], content_type: &str, max_age: u64) -> HttpResponse {
    static ETAG: &str = concat!("\"", env!("CARGO_PKG_VERSION"), "\"");

    HttpResponse::Ok()
        .content_type(content_type)
        .insert_header((header::CACHE_CONTROL, format!("public, max-age={max_age}")))
        .insert_header((header::ETAG, ETAG))
        .body(content.to_vec())
}

/// Serves the HTML page for getting a secret
pub async fn serve_get_secret_html() -> HttpResponse {
    serve_with_caching_header(
        include_bytes!("includes/get-secret.html"),
        "text/html",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_create_secret_html() -> HttpResponse {
    serve_with_caching_header(
        include_bytes!("includes/create-secret.html"),
        "text/html",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_js_client() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/hakanai-client.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_css() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/style.css"),
        "text/css",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_banner() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../banner.svg"),
        "image/svg+xml",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_logo() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../logo.svg"),
        "image/svg+xml",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_icon() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../icon.svg"),
        "image/svg+xml",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_get_secret_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/get-secret.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_create_secret_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/create-secret.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_docs_html() -> impl Responder {
    serve_with_caching_header(
        include_str!("includes/docs_generated.html").as_bytes(),
        "text/html",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_openapi_json() -> impl Responder {
    serve_with_caching_header(
        include_str!("includes/openapi.json").as_bytes(),
        "application/json",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_index() -> HttpResponse {
    serve_with_caching_header(
        include_bytes!("includes/index.html"),
        "text/html",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_manifest() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/manifest.json"),
        "application/manifest+json",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_robots_txt() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/robots.txt"),
        "text/plain",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_impressum(app_data: web::Data<crate::app_data::AppData>) -> impl Responder {
    match &app_data.impressum_html {
        Some(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header((
                header::CACHE_CONTROL,
                format!("public, max-age={DEFAULT_CACHE_MAX_AGE}"),
            ))
            .insert_header((
                header::ETAG,
                concat!("\"", env!("CARGO_PKG_VERSION"), "-impressum\""),
            ))
            .body(html.clone()),
        None => HttpResponse::NotFound().body("No impressum configured"),
    }
}

async fn serve_privacy(app_data: web::Data<crate::app_data::AppData>) -> impl Responder {
    match &app_data.privacy_html {
        Some(html) => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .insert_header((
                header::CACHE_CONTROL,
                format!("public, max-age={DEFAULT_CACHE_MAX_AGE}"),
            ))
            .insert_header((
                header::ETAG,
                concat!("\"", env!("CARGO_PKG_VERSION"), "-privacy\""),
            ))
            .body(html.clone()),
        None => HttpResponse::NotFound().body("No privacy policy configured"),
    }
}

async fn serve_config(app_data: web::Data<crate::app_data::AppData>) -> impl Responder {
    let config = serde_json::json!({
        "features": {
            "impressum": app_data.impressum_html.is_some(),
            "privacy": app_data.privacy_html.is_some(),
            "showTokenInput": app_data.show_token_input || !app_data.anonymous_usage.allowed,
        }
    });

    HttpResponse::Ok()
        .content_type("application/json")
        .insert_header((header::CACHE_CONTROL, "public, max-age=300")) // 5 minutes cache
        .insert_header((header::ETAG, env!("CARGO_PKG_VERSION")))
        .json(config)
}

async fn serve_share_html() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/share.html"),
        "text/html",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_share_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/share.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_shortcut() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../share.shortcut"),
        "application/octet-stream",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_common_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/common.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_wasm_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/hakanai_wasm.js"),
        "application/javascript",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_wasm_binary() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("includes/hakanai_wasm_bg.wasm"),
        "application/wasm",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_data::{AnonymousOptions, AppData};
    use crate::test_utils::MockDataStore;
    use actix_web::{App, test, web};

    fn create_test_app_data(impressum_html: Option<String>) -> AppData {
        AppData {
            data_store: Box::new(MockDataStore::new()),
            token_validator: Box::new(crate::test_utils::MockTokenManager::new()),
            token_creator: Box::new(crate::test_utils::MockTokenManager::new()),
            max_ttl: std::time::Duration::from_secs(7200),
            anonymous_usage: AnonymousOptions {
                allowed: true,
                upload_size_limit: 32 * 1024,
            },
            impressum_html,
            privacy_html: None,
            observer_manager: crate::observer::ObserverManager::new(),
            show_token_input: false,
            trusted_ip_ranges: None,
            trusted_ip_header: "x-forwarded-for".to_string(),
            country_header: None,
        }
    }

    #[actix_web::test]
    async fn test_serve_config_with_impressum() {
        let app_data = create_test_app_data(Some("Test impressum content".to_string()));
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/config.json", web::get().to(serve_config)),
        )
        .await;

        let req = test::TestRequest::get().uri("/config.json").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["features"]["impressum"], true);
    }

    #[actix_web::test]
    async fn test_serve_config_without_impressum() {
        let app_data = create_test_app_data(None);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/config.json", web::get().to(serve_config)),
        )
        .await;

        let req = test::TestRequest::get().uri("/config.json").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["features"]["impressum"], false);
        assert_eq!(body["features"]["privacy"], false);
    }

    #[actix_web::test]
    async fn test_serve_config_with_privacy() {
        let mut app_data = create_test_app_data(None);
        app_data.privacy_html = Some("Test privacy content".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/config.json", web::get().to(serve_config)),
        )
        .await;

        let req = test::TestRequest::get().uri("/config.json").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["features"]["privacy"], true);
    }

    #[actix_web::test]
    async fn test_serve_privacy_configured() {
        let mut app_data = create_test_app_data(None);
        app_data.privacy_html = Some("<h1>Privacy Policy</h1><p>Test content</p>".to_string());

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/privacy", web::get().to(serve_privacy)),
        )
        .await;

        let req = test::TestRequest::get().uri("/privacy").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        assert!(body_str.contains("Privacy Policy"));
        assert!(body_str.contains("Test content"));
    }

    #[actix_web::test]
    async fn test_serve_privacy_not_configured() {
        let app_data = create_test_app_data(None);
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/privacy", web::get().to(serve_privacy)),
        )
        .await;

        let req = test::TestRequest::get().uri("/privacy").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_serve_config_token_input_hidden_with_anonymous() {
        // show_token_input = false, anonymous allowed
        let mut app_data = create_test_app_data(None);
        app_data.show_token_input = false;
        app_data.anonymous_usage.allowed = true;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/config.json", web::get().to(serve_config)),
        )
        .await;

        let req = test::TestRequest::get().uri("/config.json").to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["features"]["showTokenInput"], false);
    }

    #[actix_web::test]
    async fn test_serve_config_token_input_explicitly_shown() {
        // show_token_input = true, anonymous allowed
        let mut app_data = create_test_app_data(None);
        app_data.show_token_input = true;
        app_data.anonymous_usage.allowed = true;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/config.json", web::get().to(serve_config)),
        )
        .await;

        let req = test::TestRequest::get().uri("/config.json").to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["features"]["showTokenInput"], true);
    }

    #[actix_web::test]
    async fn test_serve_config_token_input_forced_without_anonymous() {
        // show_token_input = false, anonymous NOT allowed (should force show)
        let mut app_data = create_test_app_data(None);
        app_data.show_token_input = false;
        app_data.anonymous_usage.allowed = false;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/config.json", web::get().to(serve_config)),
        )
        .await;

        let req = test::TestRequest::get().uri("/config.json").to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["features"]["showTokenInput"], true);
    }
}
