// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::process::Command;
use std::time::SystemTime;

use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use std::collections::HashMap;
use tinytemplate::TinyTemplate;

mod build {
    pub mod docs_generator;
}

use build::docs_generator;

/// Auto-detect and register files with given extension for recompilation tracking
fn register_files_for_recompilation(dir_path: &str, extension: &str) -> Result<()> {
    let dir = std::path::Path::new(dir_path);

    if !dir.exists() {
        return Ok(()); // Directory doesn't exist, nothing to register
    }

    let entries = fs::read_dir(dir)
        .with_context(|| format!("Failed to read directory: {}", dir.display()))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=src/includes/openapi.json");

    register_files_for_recompilation("src/templates", "html")?;
    register_files_for_recompilation("src/templates/docs", "html")?;
    register_files_for_recompilation("src/templates/partials", "html")?;
    register_files_for_recompilation("src/typescript", "ts")?;
    println!("cargo:rerun-if-changed=tsconfig.json");
    println!("cargo:rerun-if-changed=rollup.config.js");
    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=../wasm/src/lib.rs");
    println!("cargo:rerun-if-changed=../wasm/Cargo.toml");

    let start = std::time::Instant::now();
    compile_typescript()?;
    compile_wasm()?;
    generate_docs()?;
    generate_static_html_files()?;
    println!("cargo:warning=Build completed in {:?}", start.elapsed());

    Ok(())
}

fn ensure_rollup_is_installed() -> Result<()> {
    let is_installed = Command::new("npx")
        .args(["rollup", "--version"])
        .output()?
        .status
        .success();

    if is_installed {
        println!("cargo:warning=Rollup bundler is available");
        Ok(())
    } else {
        Err(anyhow!(
            "Rollup bundler not available. Run 'npm install' first or set SKIP_TYPESCRIPT_BUILD=1"
        ))
    }
}

fn compile_typescript() -> Result<()> {
    println!("cargo:warning=Bundling TypeScript files with Rollup...");

    if std::env::var("SKIP_TYPESCRIPT_BUILD").is_ok() {
        println!("cargo:warning=Skipping TypeScript bundling (SKIP_TYPESCRIPT_BUILD set)");
        return Ok(());
    }

    ensure_rollup_is_installed()?;

    // Bundle TypeScript files with Rollup
    let output = Command::new("npx")
        .args(["rollup", "-c"])
        .current_dir("..") // Run from workspace root where rollup.config.js is located
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("cargo:warning=Rollup bundling failed");
        println!("cargo:warning=STDOUT: {stdout}");
        println!("cargo:warning=STDERR: {stderr}");
        panic!("Rollup bundling failed: {stderr}");
    }

    add_cache_busters_to_js_files()?;

    println!("cargo:warning=TypeScript bundling successful");
    Ok(())
}

fn compile_wasm() -> Result<()> {
    println!("cargo:warning=Building WASM module...");

    if std::env::var("SKIP_WASM_BUILD").is_ok() {
        println!("cargo:warning=Skipping WASM build (SKIP_WASM_BUILD set)");
        return Ok(());
    }

    // Check if wasm-pack is installed
    let wasm_pack_check = Command::new("wasm-pack").args(["--version"]).output();

    if wasm_pack_check.is_err() || !wasm_pack_check?.status.success() {
        println!(
            "cargo:warning=wasm-pack not found. Skipping WASM build. Install with: cargo install wasm-pack"
        );
        return Ok(());
    }

    // Build WASM module
    let output = Command::new("wasm-pack")
        .args(["build", "--target", "web", "--out-dir", "pkg", "--release"])
        .current_dir("../wasm")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("cargo:warning=WASM build failed");
        println!("cargo:warning=STDOUT: {stdout}");
        println!("cargo:warning=STDERR: {stderr}");
        // Don't panic - WASM is optional enhancement
        println!("cargo:warning=Continuing without WASM support");
        return Ok(());
    }

    // Copy WASM files to includes directory (root level for direct access)
    let wasm_files = [
        (
            "../wasm/pkg/hakanai_wasm_bg.wasm",
            "src/includes/hakanai_wasm_bg.wasm",
        ),
        (
            "../wasm/pkg/hakanai_wasm.js",
            "src/includes/hakanai_wasm.js",
        ),
    ];

    for (src, dst) in &wasm_files {
        if fs::copy(src, dst).is_ok() {
            println!("cargo:warning=Copied {src} to {dst}");
        }
    }

    println!("cargo:warning=WASM module built successfully");
    Ok(())
}

fn add_cache_busters_to_js_files() -> Result<()> {
    println!("cargo:warning=Adding cache busters to JavaScript imports and JSON URLs...");

    let cache_buster = generate_cache_buster();

    // Find all .js files in src/includes/
    let includes_dir = std::path::Path::new("src/includes");
    if let Ok(entries) = fs::read_dir(includes_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.extension().map(|ext| ext == "js").unwrap_or(false)
                && let Ok(content) = fs::read_to_string(&path)
            {
                // Replace any relative .js import with versioned import
                let updated_content = content
                    .replace(".js\"", &format!(".js?v={cache_buster}\""))
                    .replace(".js'", &format!(".js?v={cache_buster}'"))
                    .replace(".json\"", &format!(".json?v={cache_buster}\""))
                    .replace(".json'", &format!(".json?v={cache_buster}'"));

                fs::write(&path, updated_content)
                    .context(format!("failed to write updated {path:?}"))?;
            }
        }
    }

    println!("cargo:warning=Cache busters added to JavaScript imports and JSON URLs");
    Ok(())
}

fn generate_docs() -> Result<()> {
    println!("cargo:warning=Generate docs...");
    let openapi = load_openapi()?;

    let html =
        docs_generator::generate_docs_html(&openapi).context("failed to generate docs HTML")?;

    fs::write("src/includes/docs_generated.html", html)
        .context("failed to write docs_generated.html")?;
    Ok(())
}

fn load_openapi() -> Result<Value> {
    let content =
        fs::read_to_string("src/includes/openapi.json").context("failed to read openapi.json")?;

    serde_json::from_str(&content).context("failed to parse openapi.json")
}

fn generate_static_html_files() -> Result<()> {
    println!("cargo:warning=Generate static HTML pages...");

    let partials = load_partials()?;
    let mut tt = TinyTemplate::new();
    let context = create_version_context();

    discover_and_generate_templates(&mut tt, &context, &partials)?;

    Ok(())
}

fn load_partials() -> Result<TemplatePartials> {
    let head = fs::read_to_string("src/templates/partials/head.html")
        .context("failed to read head partial")?;
    let theme_switcher = fs::read_to_string("src/templates/partials/theme-switcher.html")
        .context("failed to read theme-switcher partial")?;
    let language_selector = fs::read_to_string("src/templates/partials/language-selector.html")
        .context("failed to read language-selector partial")?;
    let footer = fs::read_to_string("src/templates/partials/footer.html")
        .context("failed to read footer partial")?;
    let header = fs::read_to_string("src/templates/partials/header.html")
        .context("failed to read header partial")?;
    let ttl_selector = fs::read_to_string("src/templates/partials/ttl-selector.html")
        .context("failed to read ttl-selector partial")?;
    let restrictions_tabs = fs::read_to_string("src/templates/partials/restrictions-tabs.html")
        .context("failed to read restrictions-tabs partial")?;

    Ok(TemplatePartials {
        head,
        footer,
        header,
        theme_switcher,
        language_selector,
        ttl_selector,
        restrictions_tabs,
    })
}

fn discover_and_generate_templates(
    _tt: &mut TinyTemplate,
    context: &HashMap<&'static str, String>,
    partials: &TemplatePartials,
) -> Result<()> {
    let templates_dir = std::path::Path::new("src/templates");

    if let Ok(entries) = fs::read_dir(templates_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if should_process_template(&path)? {
                let template_name = get_template_name(&path)?;
                process_single_template(context, partials, &template_name, &path)?;
            }
        }
    }

    Ok(())
}

fn should_process_template(path: &std::path::Path) -> Result<bool> {
    // Skip partials and docs directories, and non-HTML files
    if !path.is_file() || !path.extension().map(|ext| ext == "html").unwrap_or(false) {
        return Ok(false);
    }

    Ok(true)
}

fn get_template_name(path: &std::path::Path) -> Result<String> {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(|s| s.to_string())
        .context("failed to get template name")
}

fn process_single_template(
    context: &HashMap<&'static str, String>,
    partials: &TemplatePartials,
    template_name: &str,
    template_path: &std::path::Path,
) -> Result<()> {
    println!("cargo:warning=Processing template: {template_name}");

    let mut template_content = fs::read_to_string(template_path)
        .context(format!("failed to read template {template_name}"))?;

    template_content = apply_partials(template_content, partials);

    if template_name == "impressum" {
        template_content = apply_impressum_content(template_content);
    } else if template_name == "privacy" {
        template_content = apply_privacy_content(template_content);
    }

    let mut tt = TinyTemplate::new();
    tt.add_template(template_name, &template_content)?;

    let output_path = format!("src/includes/{template_name}.html");
    let html = tt
        .render(template_name, context)
        .context(format!("failed to render template {template_name}"))?;

    fs::write(output_path, html).context(format!("failed to write {template_name}.html"))
}

fn apply_partials(template_content: String, partials: &TemplatePartials) -> String {
    template_content
        .replace("[[HEAD]]", &partials.head)
        .replace("[[THEME_SWITCHER]]", &partials.theme_switcher)
        .replace("[[LANGUAGE_SELECTOR]]", &partials.language_selector)
        .replace("[[FOOTER]]", &partials.footer)
        .replace("[[HEADER]]", &partials.header)
        .replace("[[TTL_SELECTOR]]", &partials.ttl_selector)
        .replace("[[RESTRICTIONS_TABS]]", &partials.restrictions_tabs)
}

fn apply_impressum_content(template_content: String) -> String {
    // Remove build-time impressum content injection - will be handled at runtime
    template_content.replace(
        "[[IMPRESSUM_CONTENT]]",
        "<div id=\"impressum-content-placeholder\"></div>",
    )
}

fn apply_privacy_content(template_content: String) -> String {
    // Remove build-time privacy content injection - will be handled at runtime
    template_content.replace(
        "[[PRIVACY_CONTENT]]",
        "<div id=\"privacy-content-placeholder\"></div>",
    )
}

struct TemplatePartials {
    head: String,
    theme_switcher: String,
    language_selector: String,
    footer: String,
    header: String,
    ttl_selector: String,
    restrictions_tabs: String,
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
            if entry.path().extension().is_some_and(|e| e == ext)
                && let Ok(metadata) = entry.metadata()
                && let Ok(modified) = metadata.modified()
            {
                latest_time = latest_time.max(modified);
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
