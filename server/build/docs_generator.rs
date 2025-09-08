// SPDX-License-Identifier: Apache-2.0

use std::fs;

use anyhow::{Context, Result};
use handlebars::Handlebars;
use serde_json::{Value, json};

use super::cache_buster;

// Main function to generate documentation
pub fn generate() -> Result<()> {
    println!("cargo:warning=Generate docs...");

    let openapi = load_openapi()?;
    let html = generate_docs_html(&openapi).context("failed to generate docs HTML")?;

    fs::write("includes/docs_generated.html", html)
        .context("failed to write docs_generated.html")?;

    Ok(())
}

fn load_openapi() -> Result<Value> {
    let content =
        fs::read_to_string("includes/openapi.json").context("failed to read openapi.json")?;
    serde_json::from_str(&content).context("failed to parse openapi.json")
}

fn generate_docs_html(openapi: &Value) -> Result<String> {
    let mut hb = Handlebars::new();

    // Register partials (needed for head, footer, theme_switcher)
    register_partials(&mut hb)?;

    // Register all docs templates
    register_templates(&mut hb)?;

    // Generate the documentation
    let endpoints_html = generate_all_endpoints(&hb, openapi)?;
    let schemas_html = generate_schemas_section(openapi, &hb);

    // Get API info from OpenAPI spec
    let title = openapi
        .get("info")
        .and_then(|i| i.get("title"))
        .and_then(|t| t.as_str())
        .unwrap_or("API Documentation");

    let version = openapi
        .get("info")
        .and_then(|i| i.get("version"))
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0");

    let description = openapi
        .get("info")
        .and_then(|i| i.get("description"))
        .and_then(|d| d.as_str())
        .unwrap_or("API Documentation");

    let context = json!({
        "title": title,
        "version": version,
        "description": description,
        "endpoints": endpoints_html,
        "schemas": schemas_html,
        "cache_buster": cache_buster::generate(),
    });

    hb.render("docs", &context)
        .context("failed to render docs template")
}

fn register_partials(hb: &mut Handlebars) -> Result<()> {
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
        hb.register_partial(name, content)
            .context(format!("failed to register partial {}", name))?;
    }

    Ok(())
}

fn register_templates(hb: &mut Handlebars) -> Result<()> {
    let templates = [
        ("docs", "templates/docs/docs.html"),
        ("endpoint", "templates/docs/endpoint.html"),
        ("schema_section", "templates/docs/schema-section.html"),
        ("schema_definition", "templates/docs/schema-definition.html"),
        ("schema_property", "templates/docs/schema-property.html"),
        ("example_single", "templates/docs/example-single.html"),
        ("example_multiple", "templates/docs/example-multiple.html"),
        ("example_item", "templates/docs/example-item.html"),
        ("status_code", "templates/docs/status-code.html"),
        ("property_example", "templates/docs/property-example.html"),
        ("schema_reference", "templates/docs/schema-reference.html"),
        (
            "schema_description",
            "templates/docs/schema-description.html",
        ),
        (
            "schema_properties_header",
            "templates/docs/schema-properties-header.html",
        ),
        (
            "request_body_header",
            "templates/docs/request-body-header.html",
        ),
        ("examples_header", "templates/docs/examples-header.html"),
    ];

    for (name, path) in templates {
        let content = load_template(path, name)?;
        hb.register_template_string(name, content)
            .context(format!("failed to register template {}", name))?;
    }

    Ok(())
}

fn load_template(path: &str, name: &str) -> Result<String> {
    fs::read_to_string(path).context(format!("failed to load {}", name))
}

fn generate_all_endpoints(hb: &Handlebars, openapi: &Value) -> Result<String> {
    let mut all_endpoints = String::new();

    if let Some(paths) = openapi.get("paths").and_then(|p| p.as_object()) {
        for (path, path_item) in paths {
            if let Some(obj) = path_item.as_object() {
                for (method, operation) in obj {
                    if let Some(_op) = operation.as_object() {
                        let endpoint_html =
                            generate_single_endpoint(hb, path, method, operation, openapi)?;
                        all_endpoints.push_str(&endpoint_html);
                    }
                }
            }
        }
    }

    Ok(all_endpoints)
}

fn generate_single_endpoint(
    hb: &Handlebars,
    path: &str,
    method: &str,
    operation: &Value,
    openapi: &Value,
) -> Result<String> {
    let operation_id = operation
        .get("operationId")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let summary = operation
        .get("summary")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let description = operation
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let request_body_html = generate_request_body_section(operation, openapi, hb);
    let status_codes_html = generate_status_codes_section(operation, hb);

    let context = json!({
        "method": method.to_uppercase(),
        "method_class": method.to_lowercase(),
        "path": path,
        "operation_id": operation_id,
        "summary": summary,
        "description": description,
        "request_body": request_body_html,
        "status_codes": status_codes_html,
    });

    hb.render("endpoint", &context)
        .context("failed to render endpoint template")
}

fn generate_status_codes_section(operation: &Value, hb: &Handlebars) -> String {
    let mut status_codes_html = String::new();

    if let Some(responses) = operation.get("responses").and_then(|r| r.as_object()) {
        for (status_code, response) in responses {
            let description = response
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let examples = generate_response_examples(response, hb);

            let context = json!({
                "status_code": status_code,
                "description": description,
                "examples": examples,
            });

            if let Ok(rendered) = hb.render("status_code", &context) {
                status_codes_html.push_str(&rendered);
            }
        }
    }

    status_codes_html
}

fn generate_response_examples(response: &Value, hb: &Handlebars) -> String {
    if let Some(content) = response.get("content").and_then(|c| c.as_object()) {
        if content.len() == 1 {
            // Single example
            if let Some((content_type, media)) = content.iter().next() {
                if let Some(example) = media.get("example") {
                    let formatted = serde_json::to_string_pretty(example).unwrap_or_default();
                    return format_single_example(content_type, &formatted, hb);
                }
            }
        } else if content.len() > 1 {
            // Multiple examples
            return format_multiple_examples(content, hb);
        }
    }
    String::new()
}

fn format_single_example(content_type: &str, example: &str, hb: &Handlebars) -> String {
    let context = json!({
        "content_type": content_type,
        "example": example,
    });

    hb.render("example_single", &context).unwrap_or_default()
}

fn format_multiple_examples(content: &serde_json::Map<String, Value>, hb: &Handlebars) -> String {
    let mut items_html = String::new();

    for (content_type, media) in content {
        if let Some(example) = media.get("example") {
            let formatted = serde_json::to_string_pretty(example).unwrap_or_default();
            let item_context = json!({
                "content_type": content_type,
                "example": formatted,
            });

            if let Ok(rendered) = hb.render("example_item", &item_context) {
                items_html.push_str(&rendered);
            }
        }
    }

    let context = json!({
        "items": items_html,
    });

    hb.render("example_multiple", &context).unwrap_or_default()
}

fn generate_request_body_section(operation: &Value, _openapi: &Value, hb: &Handlebars) -> String {
    if let Some(request_body) = operation.get("requestBody") {
        let description = request_body
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let schema_ref = generate_schema_reference(request_body, hb);
        let examples = generate_request_examples(request_body, hb);

        let context = json!({
            "description": description,
        });

        let header = hb
            .render("request_body_header", &context)
            .unwrap_or_default();

        format!("{}{}{}", header, schema_ref, examples)
    } else {
        String::new()
    }
}

fn generate_schema_reference(request_body: &Value, hb: &Handlebars) -> String {
    if let Some(content) = request_body.get("content").and_then(|c| c.as_object()) {
        if let Some(json_content) = content.get("application/json") {
            if let Some(schema_ref) = json_content
                .get("schema")
                .and_then(|s| s.get("$ref"))
                .and_then(|r| r.as_str())
            {
                let schema_name = schema_ref.split('/').last().unwrap_or("");
                let context = json!({
                    "schema_name": schema_name,
                });
                return hb.render("schema_reference", &context).unwrap_or_default();
            }
        }
    }
    String::new()
}

fn generate_request_examples(request_body: &Value, hb: &Handlebars) -> String {
    if let Some(content) = request_body.get("content").and_then(|c| c.as_object()) {
        if let Some(json_content) = content.get("application/json") {
            if let Some(example) = json_content.get("example") {
                let formatted = serde_json::to_string_pretty(example).unwrap_or_default();
                let context = json!({});
                let header = hb.render("examples_header", &context).unwrap_or_default();
                let example_html = format_single_example("application/json", &formatted, hb);
                return format!("{}{}", header, example_html);
            } else if let Some(examples) = json_content.get("examples").and_then(|e| e.as_object())
            {
                let mut items_html = String::new();
                for (name, example_obj) in examples {
                    if let Some(value) = example_obj.get("value") {
                        let formatted = serde_json::to_string_pretty(value).unwrap_or_default();
                        items_html.push_str(&format!("<h5>{}</h5>", name));
                        items_html.push_str(&format_single_example(
                            "application/json",
                            &formatted,
                            hb,
                        ));
                    }
                }
                if !items_html.is_empty() {
                    let context = json!({});
                    let header = hb.render("examples_header", &context).unwrap_or_default();
                    return format!("{}{}", header, items_html);
                }
            }
        }
    }
    String::new()
}

fn generate_schemas_section(openapi: &Value, hb: &Handlebars) -> String {
    let mut schemas_html = String::new();

    if let Some(schemas) = openapi
        .get("components")
        .and_then(|c| c.get("schemas"))
        .and_then(|s| s.as_object())
    {
        for (name, schema) in schemas {
            let single_schema = generate_single_schema(name, schema, hb);
            schemas_html.push_str(&single_schema);
        }
    }

    let context = json!({
        "schemas": schemas_html,
    });

    hb.render("schema_section", &context).unwrap_or_default()
}

fn generate_single_schema(name: &str, schema: &Value, hb: &Handlebars) -> String {
    let description = schema
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let desc_context = json!({
        "description": description,
    });
    let description_html = if !description.is_empty() {
        hb.render("schema_description", &desc_context)
            .unwrap_or_default()
    } else {
        String::new()
    };

    let properties_html = generate_schema_properties(schema, hb);

    let context = json!({
        "schema_name": name,
        "description": description_html,
        "properties": properties_html,
    });

    hb.render("schema_definition", &context).unwrap_or_default()
}

fn generate_schema_properties(schema: &Value, hb: &Handlebars) -> String {
    let mut properties_html = String::new();

    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        let required = schema
            .get("required")
            .and_then(|r| r.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        let context = json!({});
        let header = hb
            .render("schema_properties_header", &context)
            .unwrap_or_default();
        properties_html.push_str(&header);

        for (prop_name, prop_schema) in properties {
            let prop_html = generate_single_property(
                prop_name,
                prop_schema,
                required.contains(&prop_name.as_str()),
                hb,
            );
            properties_html.push_str(&prop_html);
        }
    }

    properties_html
}

fn generate_single_property(
    name: &str,
    prop_schema: &Value,
    is_required: bool,
    hb: &Handlebars,
) -> String {
    let prop_type = determine_property_type(prop_schema);
    let description = prop_schema
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let mut constraints = Vec::new();

    if is_required {
        constraints.push("required".to_string());
    }

    if let Some(min_len) = prop_schema.get("minLength").and_then(|v| v.as_u64()) {
        constraints.push(format!("min length: {}", min_len));
    }

    if let Some(max_len) = prop_schema.get("maxLength").and_then(|v| v.as_u64()) {
        constraints.push(format!("max length: {}", max_len));
    }

    if let Some(pattern) = prop_schema.get("pattern").and_then(|v| v.as_str()) {
        constraints.push(format!("pattern: {}", pattern));
    }

    if let Some(min) = prop_schema.get("minimum").and_then(|v| v.as_u64()) {
        constraints.push(format!("min: {}", min));
    }

    if let Some(max) = prop_schema.get("maximum").and_then(|v| v.as_u64()) {
        constraints.push(format!("max: {}", max));
    }

    let constraints_html = if !constraints.is_empty() {
        constraints
            .iter()
            .map(|c| {
                if c == "required" {
                    format!(
                        "<span class=\"constraint constraint-required\">{}</span>",
                        c
                    )
                } else {
                    format!("<span class=\"constraint\">{}</span>", c)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        String::new()
    };

    let example_html = generate_property_example(prop_schema, hb);

    let context = json!({
        "name": name,
        "type": prop_type,
        "constraints": constraints_html,
        "description": description,
        "example": example_html,
    });

    hb.render("schema_property", &context).unwrap_or_default()
}

fn determine_property_type(prop_schema: &Value) -> String {
    if let Some(type_val) = prop_schema.get("type").and_then(|v| v.as_str()) {
        if type_val == "array" {
            if let Some(items) = prop_schema.get("items") {
                let item_type = determine_property_type(items);
                return format!("array of {}", item_type);
            }
            return "array".to_string();
        } else if type_val == "object" {
            if let Some(additional_props) = prop_schema.get("additionalProperties") {
                let value_type = determine_property_type(additional_props);
                return format!("object (map of {})", value_type);
            }
        }
        return type_val.to_string();
    }

    if let Some(schema_ref) = prop_schema.get("$ref").and_then(|v| v.as_str()) {
        return schema_ref.split('/').last().unwrap_or("object").to_string();
    }

    "unknown".to_string()
}

fn generate_property_example(prop_schema: &Value, hb: &Handlebars) -> String {
    if let Some(example) = prop_schema.get("example") {
        let formatted = if example.is_string() {
            format!("\"{}\"", example.as_str().unwrap_or(""))
        } else {
            serde_json::to_string(example).unwrap_or_default()
        };

        let context = json!({
            "example": formatted,
        });

        hb.render("property_example", &context).unwrap_or_default()
    } else {
        String::new()
    }
}
