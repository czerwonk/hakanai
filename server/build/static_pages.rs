// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};
use tinytemplate::TinyTemplate;

use super::cache_buster;

pub fn generate_html_files() -> Result<()> {
    println!("cargo:warning=Generate static HTML pages...");

    let partials = load_partials()?;
    let mut tt = TinyTemplate::new();
    let context = create_version_context();

    discover_and_generate_templates(&mut tt, &context, &partials)?;

    Ok(())
}

fn load_partials() -> Result<TemplatePartials> {
    let head = fs::read_to_string("templates/partials/head.html")
        .context("failed to read head partial")?;
    let theme_switcher = fs::read_to_string("templates/partials/theme-switcher.html")
        .context("failed to read theme-switcher partial")?;
    let language_selector = fs::read_to_string("templates/partials/language-selector.html")
        .context("failed to read language-selector partial")?;
    let footer = fs::read_to_string("templates/partials/footer.html")
        .context("failed to read footer partial")?;
    let header = fs::read_to_string("templates/partials/header.html")
        .context("failed to read header partial")?;
    let ttl_selector = fs::read_to_string("templates/partials/ttl-selector.html")
        .context("failed to read ttl-selector partial")?;
    let restrictions_tabs = fs::read_to_string("templates/partials/restrictions-tabs.html")
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
    let templates_dir = std::path::Path::new("templates");

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

    let output_path = format!("includes/{template_name}.html");
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
    context.insert("cache_buster", cache_buster::generate());

    context
}
