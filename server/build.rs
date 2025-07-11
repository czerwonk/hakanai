use std::collections::HashMap;
use std::fs;

use serde_json::Value;
use tinytemplate::TinyTemplate;

fn main() {
    println!("cargo:rerun-if-changed=src/includes/openapi.json");
    println!("cargo:rerun-if-changed=templates/docs.html");
    println!("cargo:rerun-if-changed=templates/endpoint.html");

    let openapi_content =
        fs::read_to_string("src/includes/openapi.json").expect("Failed to read openapi.json");

    let openapi: Value =
        serde_json::from_str(&openapi_content).expect("Failed to parse openapi.json");

    let html = generate_docs_html(&openapi);

    fs::write("src/includes/docs_generated.html", html)
        .expect("Failed to write generated docs.html");
}

fn generate_docs_html(openapi: &Value) -> String {
    let mut tt = TinyTemplate::new();

    // Load templates
    let docs_template =
        fs::read_to_string("templates/docs.html").expect("Failed to read docs template");
    let endpoint_template =
        fs::read_to_string("templates/endpoint.html").expect("Failed to read endpoint template");

    tt.add_template("docs", &docs_template).unwrap();
    tt.add_template("endpoint", &endpoint_template).unwrap();

    // Extract data from OpenAPI
    let info = &openapi["info"];
    let title = info["title"].as_str().unwrap_or("API Documentation");
    let version = info["version"].as_str().unwrap_or("1.0.0");
    let description = info["description"].as_str().unwrap_or("");

    // Generate endpoints HTML
    let mut endpoints_html = String::new();
    if let Some(paths) = openapi["paths"].as_object() {
        for (path, methods) in paths {
            if let Some(methods_obj) = methods.as_object() {
                for (method, operation) in methods_obj {
                    endpoints_html.push_str(&generate_endpoint_html(&tt, path, method, operation));
                }
            }
        }
    }

    // Prepare context for main template
    let mut context = HashMap::new();
    context.insert("title", title);
    context.insert("version", version);
    context.insert("description", description);
    context.insert("endpoints", endpoints_html.as_str());

    tt.render("docs", &context).unwrap()
}

fn generate_endpoint_html(
    tt: &TinyTemplate,
    path: &str,
    method: &str,
    operation: &Value,
) -> String {
    let summary = operation["summary"].as_str().unwrap_or("");
    let description = operation["description"].as_str().unwrap_or("");

    let method_class = method.to_lowercase();
    let method_upper = method.to_uppercase();

    // Generate status codes
    let mut status_codes_html = String::new();
    if let Some(responses) = operation["responses"].as_object() {
        for (code, response) in responses {
            let desc = response["description"].as_str().unwrap_or("");
            status_codes_html.push_str(&format!(
                "<li><strong>{} {}</strong> - {}</li>\n",
                code,
                get_status_text(code),
                desc
            ));
        }
    }

    // Generate request body section
    let mut request_body_html = String::new();
    if let Some(request_body) = operation["requestBody"].as_object() {
        if let Some(content) = request_body["content"]["application/json"].as_object() {
            if let Some(example) = content["example"].as_object() {
                request_body_html = format!(
                    r#"<h4>Request Body</h4>
                    <div class="code-block">
                      <code>{}</code>
                    </div>"#,
                    serde_json::to_string_pretty(example).unwrap_or_default()
                );
            }
        }
    }

    // Prepare context for endpoint template
    let mut context = HashMap::new();
    context.insert("summary", summary);
    context.insert("method_class", method_class.as_str());
    context.insert("method_upper", method_upper.as_str());
    context.insert("path", path);
    context.insert("description", description);
    context.insert("request_body", request_body_html.as_str());
    context.insert("status_codes", status_codes_html.as_str());

    tt.render("endpoint", &context).unwrap()
}

fn get_status_text(code: &str) -> &'static str {
    match code {
        "200" => "OK",
        "400" => "Bad Request",
        "401" => "Unauthorized",
        "404" => "Not Found",
        "410" => "Gone",
        _ => "",
    }
}
