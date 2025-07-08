use actix_web::http::header;
use actix_web::{HttpResponse, Responder, web};

const DEFAULT_CACHE_MAX_AGE: u64 = 86400; // 1 day

/// Configures the Actix Web services for the application.
///
/// This function registers the API routes and sets up the application data,
/// including the data store that will be shared across all handlers.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(serve_get_secret_html))
        .route("/common-utils.js", web::get().to(serve_common_utils_js))
        .route("/create", web::get().to(serve_create_secret_html))
        .route("/create-secret.js", web::get().to(serve_create_secret_js))
        .route("/get-secret.js", web::get().to(serve_get_secret_js))
        .route("/i18n.js", web::get().to(serve_i18n_js))
        .route("/icon.svg", web::get().to(serve_icon))
        .route("/logo.svg", web::get().to(serve_logo))
        .route("/scripts/hakanai-client.js", web::get().to(serve_js_client))
        .route("/style.css", web::get().to(serve_css));
}

fn serve_with_caching_header(content: &str, content_type: &str, max_age: u64) -> HttpResponse {
    static ETAG: &str = concat!("\"", env!("CARGO_PKG_VERSION"), "\"");

    HttpResponse::Ok()
        .content_type(content_type)
        .insert_header((header::CACHE_CONTROL, format!("public, max-age={max_age}")))
        .insert_header((header::ETAG, ETAG))
        .body(content.to_string())
}

/// Serves the HTML page for getting a secret, replacing the version placeholder
pub async fn serve_get_secret_html() -> HttpResponse {
    const CONTENT: &str = include_str!("includes/get-secret.html");
    let content = CONTENT.replace("{{ VERSION }}", env!("CARGO_PKG_VERSION"));

    serve_with_caching_header(&content, "text/html", DEFAULT_CACHE_MAX_AGE)
}

async fn serve_create_secret_html() -> HttpResponse {
    const CONTENT: &str = include_str!("includes/create-secret.html");
    let content = CONTENT.replace("{{ VERSION }}", env!("CARGO_PKG_VERSION"));

    serve_with_caching_header(&content, "text/html", DEFAULT_CACHE_MAX_AGE)
}

async fn serve_js_client() -> impl Responder {
    serve_with_caching_header(
        include_str!("includes/hakanai-client.js"),
        "application/javascript",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_common_utils_js() -> impl Responder {
    serve_with_caching_header(
        include_str!("includes/common-utils.js"),
        "application/javascript",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_i18n_js() -> impl Responder {
    serve_with_caching_header(
        include_str!("includes/i18n.js"),
        "application/javascript",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_css() -> impl Responder {
    serve_with_caching_header(
        include_str!("includes/style.css"),
        "text/css",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_logo() -> impl Responder {
    serve_with_caching_header(
        include_str!("../../logo.svg"),
        "image/svg+xml",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_icon() -> impl Responder {
    serve_with_caching_header(
        include_str!("../../icon.svg"),
        "image/svg+xml",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_get_secret_js() -> impl Responder {
    serve_with_caching_header(
        include_str!("includes/get-secret.js"),
        "application/javascript",
        DEFAULT_CACHE_MAX_AGE,
    )
}

async fn serve_create_secret_js() -> impl Responder {
    serve_with_caching_header(
        include_str!("includes/create-secret.js"),
        "application/javascript",
        DEFAULT_CACHE_MAX_AGE,
    )
}
