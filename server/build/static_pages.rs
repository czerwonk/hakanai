// SPDX-License-Identifier: Apache-2.0

use std::fs;

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::json;

use super::cache_buster;

pub fn generate_html_files() -> Result<()> {
    println!("cargo:warning=Generate static HTML pages...");

    let mut handlebars = Handlebars::new();
    register_partials(&mut handlebars)?;

    let context = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "cache_buster": cache_buster::generate(),
    });

    discover_and_generate_templates(&mut handlebars, &context)?;

    Ok(())
}

fn register_partials(handlebars: &mut Handlebars) -> Result<()> {
    let partials = [
        ("head", "templates/partials/head.html"),
        ("theme_switcher", "templates/partials/theme-switcher.html"),
        (
            "language_selector",
            "templates/partials/language-selector.html",
        ),
        ("footer", "templates/partials/footer.html"),
        ("header", "templates/partials/header.html"),
        ("ttl_selector", "templates/partials/ttl-selector.html"),
        (
            "restrictions_tabs",
            "templates/partials/restrictions-tabs.html",
        ),
    ];

    for (name, path) in partials {
        let content =
            fs::read_to_string(path).context(format!("failed to read partial {}", path))?;
        handlebars
            .register_partial(name, content)
            .context(format!("failed to register partial {}", name))?;
    }

    Ok(())
}

fn discover_and_generate_templates(
    handlebars: &mut Handlebars,
    context: &serde_json::Value,
) -> Result<()> {
    let templates_dir = std::path::Path::new("templates");

    if let Ok(entries) = fs::read_dir(templates_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();

            if should_process_template(&path)? {
                let template_name = get_template_name(&path)?;
                process_single_template(handlebars, context, &template_name, &path)?;
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
    handlebars: &mut Handlebars,
    context: &serde_json::Value,
    template_name: &str,
    template_path: &std::path::Path,
) -> Result<()> {
    println!("cargo:warning=Processing template: {template_name}");

    let template_content = fs::read_to_string(template_path)
        .context(format!("failed to read template {template_name}"))?;

    // Register and render the template
    handlebars
        .register_template_string(template_name, &template_content)
        .context(format!("failed to register template {}", template_name))?;

    let output_path = format!("includes/{template_name}.html");
    let html = handlebars
        .render(template_name, context)
        .context(format!("failed to render template {template_name}"))?;

    fs::write(output_path, html).context(format!("failed to write {template_name}.html"))
}
