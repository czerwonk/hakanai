use std::collections::HashMap;
use std::fs;
use std::process::Command;

use serde_json::Value;
use tinytemplate::TinyTemplate;

fn main() {
    println!("cargo:rerun-if-changed=src/includes/openapi.json");
    println!("cargo:rerun-if-changed=templates/docs.html");
    println!("cargo:rerun-if-changed=templates/endpoint.html");
    println!("cargo:rerun-if-changed=templates/create-secret.html");
    println!("cargo:rerun-if-changed=templates/get-secret.html");

    // TypeScript files that should trigger recompilation
    println!("cargo:rerun-if-changed=src/typescript/hakanai-client.ts");
    println!("cargo:rerun-if-changed=src/typescript/common-utils.ts");
    println!("cargo:rerun-if-changed=src/typescript/i18n.ts");
    println!("cargo:rerun-if-changed=src/typescript/get-secret.ts");
    println!("cargo:rerun-if-changed=src/typescript/create-secret.ts");
    println!("cargo:rerun-if-changed=src/typescript/types.ts");
    println!("cargo:rerun-if-changed=tsconfig.json");

    compile_typescript();
    generate_docs();
    generate_static_html_files();
}

fn compile_typescript() {
    println!("cargo:warning=Compiling TypeScript files...");

    // Check if we should skip TypeScript compilation (for CI/Docker builds)
    if std::env::var("SKIP_TYPESCRIPT_BUILD").is_ok() {
        println!("cargo:warning=Skipping TypeScript compilation (SKIP_TYPESCRIPT_BUILD set)");
        return;
    }

    // Check if pre-compiled JavaScript files exist
    let js_files = [
        "src/includes/hakanai-client.js",
        "src/includes/common-utils.js",
        "src/includes/i18n.js",
        "src/includes/get-secret.js",
        "src/includes/create-secret.js",
        "src/includes/types.js",
    ];

    let all_js_exist = js_files
        .iter()
        .all(|file| std::path::Path::new(file).exists());

    if all_js_exist {
        println!(
            "cargo:warning=Pre-compiled JavaScript files found, skipping TypeScript compilation"
        );
        return;
    }

    // Check if TypeScript compiler is available
    let tsc_check = Command::new("tsc").arg("--version").output();

    match tsc_check {
        Ok(output) if output.status.success() => {
            println!("cargo:warning=Found TypeScript compiler");
        }
        _ => {
            println!(
                "cargo:warning=TypeScript compiler (tsc) not found. Please install TypeScript with: npm install -g typescript"
            );
            println!(
                "cargo:warning=Or set SKIP_TYPESCRIPT_BUILD=1 to use pre-compiled JavaScript files"
            );
            panic!(
                "TypeScript compiler not available. Install with: npm install -g typescript or set SKIP_TYPESCRIPT_BUILD=1"
            );
        }
    }

    // Compile TypeScript files
    let output = Command::new("tsc")
        .current_dir("..") // Run from workspace root where tsconfig.json is located
        .output()
        .expect("Failed to execute TypeScript compiler");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("cargo:warning=TypeScript compilation failed");
        println!("cargo:warning=STDOUT: {stdout}");
        println!("cargo:warning=STDERR: {stderr}");
        panic!("TypeScript compilation failed: {stderr}");
    }

    println!("cargo:warning=TypeScript compilation successful");
}

fn generate_docs() {
    let openapi = load_openapi();
    let html = generate_docs_html(&openapi);
    fs::write("src/includes/docs_generated.html", html)
        .expect("Failed to write generated docs.html");
}

fn load_openapi() -> Value {
    let content =
        fs::read_to_string("src/includes/openapi.json").expect("Failed to read openapi.json");
    serde_json::from_str(&content).expect("Failed to parse openapi.json")
}

fn generate_docs_html(openapi: &Value) -> String {
    let mut tt = TinyTemplate::new();

    let docs_template =
        fs::read_to_string("templates/docs.html").expect("Failed to read docs template");
    let endpoint_template =
        fs::read_to_string("templates/endpoint.html").expect("Failed to read endpoint template");

    tt.add_template("docs", &docs_template).unwrap();
    tt.add_template("endpoint", &endpoint_template).unwrap();

    let endpoints_html = generate_all_endpoints(&tt, openapi);
    let context = create_docs_context(openapi, &endpoints_html);
    tt.render("docs", &context).unwrap()
}

fn generate_all_endpoints(tt: &TinyTemplate, openapi: &Value) -> String {
    let mut endpoints_html = String::new();
    if let Some(paths) = openapi["paths"].as_object() {
        for (path, methods) in paths {
            if let Some(methods_obj) = methods.as_object() {
                for (method, operation) in methods_obj {
                    endpoints_html.push_str(&generate_endpoint_html(tt, path, method, operation));
                }
            }
        }
    }
    endpoints_html
}

fn create_docs_context<'a>(
    openapi: &'a Value,
    endpoints_html: &'a str,
) -> HashMap<&'static str, &'a str> {
    let info = &openapi["info"];
    let mut context = HashMap::new();
    context.insert(
        "title",
        info["title"].as_str().unwrap_or("API Documentation"),
    );
    context.insert("version", info["version"].as_str().unwrap_or("1.0.0"));
    context.insert("description", info["description"].as_str().unwrap_or(""));
    context.insert("endpoints", endpoints_html);
    context
}

fn generate_endpoint_html(
    tt: &TinyTemplate,
    path: &str,
    method: &str,
    operation: &Value,
) -> String {
    let status_codes_html = generate_status_codes(operation);
    let request_body_html = generate_request_body(operation);
    let context = create_endpoint_context(
        path,
        method,
        operation,
        &status_codes_html,
        &request_body_html,
    );
    tt.render("endpoint", &context).unwrap()
}

fn generate_status_codes(operation: &Value) -> String {
    let mut html = String::new();
    if let Some(responses) = operation["responses"].as_object() {
        for (code, response) in responses {
            let desc = response["description"].as_str().unwrap_or("");
            html.push_str(&format!(
                "<li><strong>{} {}</strong> - {}</li>\n",
                code,
                get_status_text(code),
                desc
            ));
        }
    }
    html
}

fn generate_request_body(operation: &Value) -> String {
    if let Some(request_body) = operation["requestBody"].as_object() {
        if let Some(content) = request_body["content"]["application/json"].as_object() {
            if let Some(example) = content["example"].as_object() {
                return format!(
                    r#"<h4>Request Body</h4>
                    <div class="code-block">
                      <code>{}</code>
                    </div>"#,
                    serde_json::to_string_pretty(example).unwrap_or_default()
                );
            }
        }
    }
    String::new()
}

fn create_endpoint_context<'a>(
    path: &'a str,
    method: &str,
    operation: &'a Value,
    status_codes_html: &'a str,
    request_body_html: &'a str,
) -> HashMap<String, String> {
    let method_class = method.to_lowercase();
    let method_upper = method.to_uppercase();

    let mut context: HashMap<String, String> = HashMap::new();
    context.insert(
        "summary".to_string(),
        operation["summary"].as_str().unwrap_or("").to_string(),
    );
    context.insert("method_class".to_string(), method_class);
    context.insert("method_upper".to_string(), method_upper);
    context.insert("path".to_string(), path.to_string());
    context.insert(
        "description".to_string(),
        operation["description"].as_str().unwrap_or("").to_string(),
    );
    context.insert("request_body".to_string(), request_body_html.to_string());
    context.insert("status_codes".to_string(), status_codes_html.to_string());

    context
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

fn generate_static_html_files() {
    let mut tt = TinyTemplate::new();

    let create_secret_template = fs::read_to_string("templates/create-secret.html")
        .expect("Failed to read create-secret template");
    let get_secret_template = fs::read_to_string("templates/get-secret.html")
        .expect("Failed to read get-secret template");

    tt.add_template("create-secret", &create_secret_template)
        .unwrap();
    tt.add_template("get-secret", &get_secret_template).unwrap();

    let context = create_version_context();
    generate_html_file(
        &tt,
        "create-secret",
        &context,
        "src/includes/create-secret.html",
    );
    generate_html_file(&tt, "get-secret", &context, "src/includes/get-secret.html");
}

fn create_version_context() -> HashMap<&'static str, &'static str> {
    let mut context = HashMap::new();
    context.insert("version", env!("CARGO_PKG_VERSION"));
    context
}

fn generate_html_file(
    tt: &TinyTemplate,
    template_name: &str,
    context: &HashMap<&'static str, &'static str>,
    output_path: &str,
) {
    let html = tt.render(template_name, context).unwrap();
    fs::write(output_path, html).unwrap_or_else(|_| panic!("Failed to write {output_path}"));
}
