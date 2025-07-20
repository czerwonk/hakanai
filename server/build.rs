use std::fs;
use std::process::Command;
use std::{collections::HashMap, time::SystemTime};

use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use tinytemplate::TinyTemplate;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/includes/openapi.json");
    println!("cargo:rerun-if-changed=src/templates/docs.html");
    println!("cargo:rerun-if-changed=src/templates/endpoint.html");
    println!("cargo:rerun-if-changed=src/templates/create-secret.html");
    println!("cargo:rerun-if-changed=src/templates/get-secret.html");

    // TypeScript files that should trigger recompilation
    println!("cargo:rerun-if-changed=src/typescript/hakanai-client.ts");
    println!("cargo:rerun-if-changed=src/typescript/common-utils.ts");
    println!("cargo:rerun-if-changed=src/typescript/i18n.ts");
    println!("cargo:rerun-if-changed=src/typescript/get-secret.ts");
    println!("cargo:rerun-if-changed=src/typescript/create-secret.ts");
    println!("cargo:rerun-if-changed=src/typescript/types.ts");
    println!("cargo:rerun-if-changed=tsconfig.json");

    let start = std::time::Instant::now();
    compile_typescript()?;
    generate_docs()?;
    generate_static_html_files()?;
    println!("cargo:warning=Build completed in {:?}", start.elapsed());

    Ok(())
}

fn all_js_files_exist() -> bool {
    let js_files = [
        "src/includes/hakanai-client.js",
        "src/includes/common-utils.js",
        "src/includes/i18n.js",
        "src/includes/get-secret.js",
        "src/includes/create-secret.js",
        "src/includes/types.js",
    ];

    js_files
        .iter()
        .all(|file| std::path::Path::new(file).exists())
}

fn ensure_typescript_is_installed() -> Result<()> {
    let is_installed = Command::new("tsc")
        .arg("--version")
        .output()?
        .status
        .success();

    if is_installed {
        println!("cargo:warning=TypeScript compiler (tsc) is installed");
        Ok(())
    } else {
        Err(anyhow!(
            "TypeScript compiler not available. Install with: npm install -g typescript or set SKIP_TYPESCRIPT_BUILD=1"
        ))
    }
}

fn compile_typescript() -> Result<()> {
    println!("cargo:warning=Compiling TypeScript files...");

    if std::env::var("SKIP_TYPESCRIPT_BUILD").is_ok() && all_js_files_exist() {
        println!("cargo:warning=Skipping TypeScript compilation (SKIP_TYPESCRIPT_BUILD set)");
        return Ok(());
    }

    ensure_typescript_is_installed()?;

    // Compile TypeScript files
    let output = Command::new("tsc")
        .current_dir("..") // Run from workspace root where tsconfig.json is located
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("cargo:warning=TypeScript compilation failed");
        println!("cargo:warning=STDOUT: {stdout}");
        println!("cargo:warning=STDERR: {stderr}");
        panic!("TypeScript compilation failed: {stderr}");
    }

    add_cache_busters_to_js_files()?;

    println!("cargo:warning=TypeScript compilation successful");
    Ok(())
}

fn add_cache_busters_to_js_files() -> Result<()> {
    println!("cargo:warning=Adding cache busters to JavaScript imports...");

    let cache_buster = generate_cache_buster();

    // Find all .js files in src/includes/
    let includes_dir = std::path::Path::new("src/includes");
    if let Ok(entries) = fs::read_dir(includes_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|ext| ext == "js").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Replace any relative .js import with versioned import
                    let updated_content = content
                        .replace(".js\"", &format!(".js?v={}\"", cache_buster))
                        .replace(".js'", &format!(".js?v={}'", cache_buster));

                    fs::write(&path, updated_content)
                        .context(format!("failed to write updated {:?}", path))?;
                }
            }
        }
    }

    println!("cargo:warning=Cache busters added to JavaScript imports");
    Ok(())
}

fn generate_docs() -> Result<()> {
    println!("cargo:warning=Generate docs...");
    let openapi = load_openapi()?;

    let html = generate_docs_html(&openapi).context("failed to generate docs HTML")?;

    fs::write("src/includes/docs_generated.html", html)
        .context("failed to write docs_generated.html")?;
    Ok(())
}

fn load_openapi() -> Result<Value> {
    let content =
        fs::read_to_string("src/includes/openapi.json").context("failed to read openapi.json")?;

    serde_json::from_str(&content).context("failed to parse openapi.json")
}

fn generate_docs_html(openapi: &Value) -> Result<String> {
    let mut tt = TinyTemplate::new();

    let docs_template =
        fs::read_to_string("src/templates/docs.html").context("Failed to read docs template")?;
    let endpoint_template = fs::read_to_string("src/templates/endpoint.html")
        .context("Failed to read endpoint template")?;

    tt.add_template("docs", &docs_template)?;
    tt.add_template("endpoint", &endpoint_template)?;

    let endpoints_html = generate_all_endpoints(&tt, openapi)?;
    let context = create_docs_context(openapi, &endpoints_html);

    Ok(tt.render("docs", &context)?)
}

fn generate_all_endpoints(tt: &TinyTemplate, openapi: &Value) -> Result<String> {
    let mut endpoints_html = String::new();
    if let Some(paths) = openapi["paths"].as_object() {
        for (path, methods) in paths {
            if let Some(methods_obj) = methods.as_object() {
                for (method, operation) in methods_obj {
                    endpoints_html.push_str(&generate_endpoint_html(tt, path, method, operation)?);
                }
            }
        }
    }

    Ok(endpoints_html)
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
    context.insert("version", env!("CARGO_PKG_VERSION"));
    context.insert("description", info["description"].as_str().unwrap_or(""));
    context.insert("endpoints", endpoints_html);

    context
}

fn generate_endpoint_html(
    tt: &TinyTemplate,
    path: &str,
    method: &str,
    operation: &Value,
) -> Result<String> {
    let status_codes_html = generate_status_codes(operation);
    let request_body_html = generate_request_body(operation);
    let context = create_endpoint_context(
        path,
        method,
        operation,
        &status_codes_html,
        &request_body_html,
    );

    tt.render("endpoint", &context).context(format!(
        "failed to render endpoint template for {method} {path}"
    ))
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

fn html_escape_value(input: &Value) -> String {
    input
        .as_str()
        .unwrap_or("")
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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
        html_escape_value(&operation["summary"]),
    );
    context.insert("method_class".to_string(), method_class);
    context.insert("method_upper".to_string(), method_upper);
    context.insert("path".to_string(), path.to_string());
    context.insert(
        "description".to_string(),
        html_escape_value(&operation["description"]),
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

fn generate_static_html_files() -> Result<()> {
    println!("cargo:warning=Generate static HTML pages...");
    let mut tt = TinyTemplate::new();

    let create_secret_template = fs::read_to_string("src/templates/create-secret.html")
        .context("failed to read create-secret template")?;
    let get_secret_template = fs::read_to_string("src/templates/get-secret.html")
        .context("failed to read get-secret template")?;
    let homepage_template = fs::read_to_string("src/templates/homepage.html")
        .context("failed to read homepage template")?;

    tt.add_template("create-secret", &create_secret_template)?;
    tt.add_template("get-secret", &get_secret_template)?;
    tt.add_template("homepage", &homepage_template)?;

    let context = create_version_context();

    generate_html_file(
        &tt,
        "create-secret",
        &context,
        "src/includes/create-secret.html",
    )?;
    generate_html_file(&tt, "get-secret", &context, "src/includes/get-secret.html")?;
    generate_html_file(&tt, "homepage", &context, "src/includes/homepage.html")
}

fn create_version_context() -> HashMap<&'static str, String> {
    let mut context = HashMap::new();
    context.insert("version", env!("CARGO_PKG_VERSION").to_string());
    context.insert("cache_buster", generate_cache_buster());

    context
}

fn get_latest_modified_time(path: &str, ext: &str) -> SystemTime {
    let mut latest_time = SystemTime::UNIX_EPOCH;

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().extension().is_some_and(|e| e == ext) {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        latest_time = latest_time.max(modified);
                    }
                }
            }
        }
    }

    latest_time
}

fn generate_cache_buster() -> String {
    let typescript_modified = get_latest_modified_time("src/typescript", "ts");
    let includes_modified = get_latest_modified_time("src/includes", "css");
    let templates_modified = get_latest_modified_time("src/templates", "html");

    [typescript_modified, includes_modified, templates_modified]
        .iter()
        .max()
        .unwrap_or(&SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}

fn generate_html_file(
    tt: &TinyTemplate,
    template_name: &str,
    context: &HashMap<&'static str, String>,
    output_path: &str,
) -> Result<()> {
    let html = tt
        .render(template_name, context)
        .context(format!("failed to render template {template_name}"))?;

    fs::write(output_path, html).context(format!("failed to write {output_path}"))
}
