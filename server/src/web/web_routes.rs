// SPDX-License-Identifier: Apache-2.0

use actix_web::http::header;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use tracing::error;

use super::app_data::AppData;
use super::filters;
use super::web_assets::AssetManager;

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
        .route("/openapi.yaml", web::get().to(serve_openapi_yaml))
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
        include_bytes!("../../includes/get-secret.html"),
        "text/html",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_create_secret_html() -> HttpResponse {
    serve_with_caching_header(
        include_bytes!("../../includes/create-secret.html"),
        "text/html",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_js_client() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/hakanai-client.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_css(asset_manager: web::Data<AssetManager>) -> impl Responder {
    let asset_res = asset_manager
        .get_embedded_asset_append_custom("style.css", include_bytes!("../../includes/style.css"))
        .await;

    match asset_res {
        Ok(content) => serve_with_caching_header(&content, "text/css", VOLATILE_CACHE_MAX_AGE),
        Err(e) => {
            error!("Failed to load CSS asset: {e}");
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

async fn serve_banner(asset_manager: web::Data<AssetManager>) -> impl Responder {
    let asset_res = asset_manager
        .get_embedded_asset_or_custom("banner.svg", include_bytes!("../../../banner.svg"))
        .await;

    match asset_res {
        Ok(content) => serve_with_caching_header(&content, "image/svg+xml", DEFAULT_CACHE_MAX_AGE),
        Err(e) => {
            error!("Failed to load banner asset: {e}");
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

async fn serve_logo(asset_manager: web::Data<AssetManager>) -> impl Responder {
    let asset_res = asset_manager
        .get_embedded_asset_or_custom("logo.svg", include_bytes!("../../../logo.svg"))
        .await;

    match asset_res {
        Ok(content) => serve_with_caching_header(&content, "image/svg+xml", DEFAULT_CACHE_MAX_AGE),
        Err(e) => {
            error!("Failed to load logo asset: {e}");
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

async fn serve_icon(asset_manager: web::Data<AssetManager>) -> impl Responder {
    let asset_res = asset_manager
        .get_embedded_asset_or_custom("icon.svg", include_bytes!("../../../icon.svg"))
        .await;

    match asset_res {
        Ok(content) => serve_with_caching_header(&content, "image/svg+xml", DEFAULT_CACHE_MAX_AGE),
        Err(e) => {
            error!("Failed to load icon asset: {e}");
            HttpResponse::InternalServerError().body("Internal Server Error")
        }
    }
}

async fn serve_get_secret_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/get-secret.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_create_secret_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/create-secret.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_docs_html() -> impl Responder {
    serve_with_caching_header(
        include_str!("../../includes/docs.html").as_bytes(),
        "text/html",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_openapi_yaml() -> impl Responder {
    serve_with_caching_header(
        include_str!("../../includes/openapi.yaml").as_bytes(),
        "application/yaml",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_index() -> HttpResponse {
    serve_with_caching_header(
        include_bytes!("../../includes/index.html"),
        "text/html",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_manifest() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/manifest.json"),
        "application/manifest+json",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_robots_txt() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/robots.txt"),
        "text/plain",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_impressum(app_data: web::Data<AppData>) -> impl Responder {
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

async fn serve_privacy(app_data: web::Data<AppData>) -> impl Responder {
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

async fn serve_config(app_data: web::Data<AppData>, req: HttpRequest) -> impl Responder {
    let whitelisted = filters::is_request_from_whitelisted_ip(&req, &app_data);
    let size_limit = if whitelisted {
        app_data.upload_size_limit
    } else if app_data.anonymous_usage.allowed {
        app_data.anonymous_usage.upload_size_limit
    } else {
        0
    };

    let config = serde_json::json!({
        "showTokenInput": app_data.show_token_input || !app_data.anonymous_usage.allowed,
        "features": {
            "impressum": app_data.impressum_html.is_some(),
            "privacy": app_data.privacy_html.is_some(),
            "restrictions": {
              "country": app_data.country_header.is_some(),
              "asn": app_data.asn_header.is_some(),
            }
        },
        "secretSizeLimit": size_limit,
    });

    HttpResponse::Ok()
        .content_type("application/json")
        .insert_header((header::CACHE_CONTROL, "public, max-age=300")) // 5 minutes cache
        .insert_header((header::ETAG, env!("CARGO_PKG_VERSION")))
        .json(config)
}

async fn serve_share_html() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/share.html"),
        "text/html",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_share_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/share.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_shortcut() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../../share.shortcut"),
        "application/octet-stream",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_common_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/common.js"),
        "application/javascript",
        VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_wasm_js() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/hakanai_wasm.js"),
        "application/javascript",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

async fn serve_wasm_binary() -> impl Responder {
    serve_with_caching_header(
        include_bytes!("../../includes/hakanai_wasm_bg.wasm"),
        "application/wasm",
        HIGHLY_VOLATILE_CACHE_MAX_AGE,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{App, test, web};

    use hakanai_lib::utils::test::MustParse;

    use crate::web::app_data::{AnonymousOptions, AppData};

    fn create_test_app_data() -> AppData {
        AppData::default()
            .with_max_ttl(std::time::Duration::from_secs(7200))
            .with_anonymous_usage(AnonymousOptions {
                allowed: true,
                upload_size_limit: 32 * 1024,
            })
    }

    #[actix_web::test]
    async fn test_serve_config_defaults() {
        let app_data = create_test_app_data();
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
    async fn test_serve_config_with_impressum() {
        let app_data = create_test_app_data().with_impressum_html("Test impressum content");
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
        let app_data = create_test_app_data();
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
    }

    #[actix_web::test]
    async fn test_serve_impressum_configured() {
        let app_data =
            create_test_app_data().with_impressum_html("<h1>Impressum</h1><p>Test content</p>");

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/impressum", web::get().to(serve_impressum)),
        )
        .await;

        let req = test::TestRequest::get().uri("/impressum").to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).expect("Response body is not valid UTF-8");
        assert!(body_str.contains("Impressum"));
        assert!(body_str.contains("Test content"));
    }

    #[actix_web::test]
    async fn test_serve_impressum_not_configured() {
        let app_data = create_test_app_data();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/impressum", web::get().to(serve_impressum)),
        )
        .await;

        let req = test::TestRequest::get().uri("/impressum").to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_serve_config_with_privacy() {
        let app_data = create_test_app_data().with_privacy_html("Test privacy content");

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
        let app_data =
            create_test_app_data().with_privacy_html("<h1>Privacy Policy</h1><p>Test content</p>");

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
        let body_str = std::str::from_utf8(&body).expect("Response body is not valid UTF-8");
        assert!(body_str.contains("Privacy Policy"));
        assert!(body_str.contains("Test content"));
    }

    #[actix_web::test]
    async fn test_serve_privacy_not_configured() {
        let app_data = create_test_app_data();
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
        let mut app_data = create_test_app_data();
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
        assert_eq!(body["showTokenInput"], false);
    }

    #[actix_web::test]
    async fn test_serve_config_token_input_explicitly_shown() {
        // show_token_input = true, anonymous allowed
        let mut app_data = create_test_app_data();
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
        assert_eq!(body["showTokenInput"], true);
    }

    #[actix_web::test]
    async fn test_serve_config_token_input_forced_without_anonymous() {
        // show_token_input = false, anonymous NOT allowed (should force show)
        let mut app_data = create_test_app_data();
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
        assert_eq!(body["showTokenInput"], true);
    }

    #[actix_web::test]
    async fn test_serve_config_secret_size_limit_anonymous() {
        let expected = 1024 as usize;
        let app_data = create_test_app_data().with_anonymous_usage(AnonymousOptions {
            allowed: true,
            upload_size_limit: expected,
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/config.json", web::get().to(serve_config)),
        )
        .await;

        let req = test::TestRequest::get().uri("/config.json").to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["secretSizeLimit"], expected);
    }

    #[actix_web::test]
    async fn test_serve_config_secret_size_limit_whitelisted() {
        let limit = 1024 as usize;
        let mut app_data = create_test_app_data()
            .with_anonymous_usage(AnonymousOptions {
                allowed: true,
                upload_size_limit: limit,
            })
            .with_trusted_ip_header("x-real-ip".to_string())
            .with_trusted_ip_ranges(Some(vec!["127.0.0.1/32".must_parse()]));
        app_data.upload_size_limit = 2048;

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_data))
                .route("/config.json", web::get().to(serve_config)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/config.json")
            .insert_header(("x-real-ip", "127.0.0.1"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["secretSizeLimit"], 2048);
    }
}
