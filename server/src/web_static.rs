use actix_web::{HttpResponse, Responder, web};

pub const SECRET_HTML_CONTENT: &str = include_str!("includes/get-secret.html");

/// Configures the Actix Web services for the application.
///
/// This function registers the API routes and sets up the application data,
/// including the data store that will be shared across all handlers.
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.route("/", web::get().to(serve_get_secret_html))
        .route("/create", web::get().to(serve_create_secret_html))
        .route("/scripts/hakanai-client.js", web::get().to(serve_js_client))
        .route("/i18n.js", web::get().to(serve_i18n_js))
        .route("/style.css", web::get().to(serve_css))
        .route("/get-secret.js", web::get().to(serve_get_secret_js))
        .route("/create-secret.js", web::get().to(serve_create_secret_js))
        .route("/icon.svg", web::get().to(serve_icon))
        .route("/logo.svg", web::get().to(serve_logo));
}

async fn serve_js_client() -> impl Responder {
    const CONTENT: &str = include_str!("includes/hakanai-client.js");
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(CONTENT)
}

async fn serve_i18n_js() -> impl Responder {
    const CONTENT: &str = include_str!("includes/i18n.js");
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(CONTENT)
}

async fn serve_css() -> impl Responder {
    const CONTENT: &str = include_str!("includes/style.css");
    HttpResponse::Ok().content_type("text/css").body(CONTENT)
}

async fn serve_logo() -> impl Responder {
    const CONTENT: &str = include_str!("../../logo.svg");
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(CONTENT)
}

async fn serve_icon() -> impl Responder {
    const CONTENT: &str = include_str!("../../icon.svg");
    HttpResponse::Ok()
        .content_type("image/svg+xml")
        .body(CONTENT)
}

async fn serve_get_secret_html() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(SECRET_HTML_CONTENT)
}

async fn serve_create_secret_html() -> impl Responder {
    const CONTENT: &str = include_str!("includes/create-secret.html");
    HttpResponse::Ok().content_type("text/html").body(CONTENT)
}

async fn serve_get_secret_js() -> impl Responder {
    const CONTENT: &str = include_str!("includes/get-secret.js");
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(CONTENT)
}

async fn serve_create_secret_js() -> impl Responder {
    const CONTENT: &str = include_str!("includes/create-secret.js");
    HttpResponse::Ok()
        .content_type("application/javascript")
        .body(CONTENT)
}
