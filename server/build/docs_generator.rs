// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::fs;

use anyhow::{Context, Result};
use serde_json::Value;
use tinytemplate::TinyTemplate;

/// Main function to generate documentation HTML from OpenAPI spec
pub fn generate_docs_html(openapi: &Value) -> Result<String> {
    let docs_template = load_template("src/templates/docs/docs.html", "docs template")?;
    let endpoint_template = load_template("src/templates/docs/endpoint.html", "endpoint template")?;

    // Load component templates
    let schema_section_template = load_template(
        "src/templates/docs/schema-section.html",
        "schema section template",
    )?;
    let schema_definition_template = load_template(
        "src/templates/docs/schema-definition.html",
        "schema definition template",
    )?;
    let schema_property_template = load_template(
        "src/templates/docs/schema-property.html",
        "schema property template",
    )?;
    let example_single_template = load_template(
        "src/templates/docs/example-single.html",
        "single example template",
    )?;
    let example_multiple_template = load_template(
        "src/templates/docs/example-multiple.html",
        "multiple example template",
    )?;
    let example_item_template = load_template(
        "src/templates/docs/example-item.html",
        "example item template",
    )?;
    let status_code_template = load_template(
        "src/templates/docs/status-code.html",
        "status code template",
    )?;
    let property_example_template = load_template(
        "src/templates/docs/property-example.html",
        "property example template",
    )?;
    let schema_reference_template = load_template(
        "src/templates/docs/schema-reference.html",
        "schema reference template",
    )?;
    let schema_description_template = load_template(
        "src/templates/docs/schema-description.html",
        "schema description template",
    )?;
    let schema_properties_header_template = load_template(
        "src/templates/docs/schema-properties-header.html",
        "schema properties header template",
    )?;
    let request_body_header_template = load_template(
        "src/templates/docs/request-body-header.html",
        "request body header template",
    )?;
    let examples_header_template = load_template(
        "src/templates/docs/examples-header.html",
        "examples header template",
    )?;

    let mut tt = TinyTemplate::new();
    tt.add_template("docs", &docs_template)?;
    tt.add_template("endpoint", &endpoint_template)?;
    tt.add_template("schema_section", &schema_section_template)?;
    tt.add_template("schema_definition", &schema_definition_template)?;
    tt.add_template("schema_property", &schema_property_template)?;
    tt.add_template("example_single", &example_single_template)?;
    tt.add_template("example_multiple", &example_multiple_template)?;
    tt.add_template("example_item", &example_item_template)?;
    tt.add_template("status_code", &status_code_template)?;
    tt.add_template("property_example", &property_example_template)?;
    tt.add_template("schema_reference", &schema_reference_template)?;
    tt.add_template("schema_description", &schema_description_template)?;
    tt.add_template(
        "schema_properties_header",
        &schema_properties_header_template,
    )?;
    tt.add_template("request_body_header", &request_body_header_template)?;
    tt.add_template("examples_header", &examples_header_template)?;

    let endpoints_html = generate_all_endpoints(&tt, openapi)?;
    let schemas_html = generate_schemas_section(openapi, &tt);
    let context = create_docs_context(openapi, &endpoints_html, &schemas_html);

    tt.render("docs", &context)
        .context("Failed to render main docs template")
}

/// Load and prepare a template file with partials applied
fn load_template(path: &str, description: &str) -> Result<String> {
    let mut template =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", description))?;

    let partials = load_partials()?;
    template = apply_partials(template, &partials);

    Ok(template)
}

/// Generate HTML for all API endpoints
fn generate_all_endpoints(tt: &TinyTemplate, openapi: &Value) -> Result<String> {
    let mut endpoints_html = String::new();

    if let Some(paths) = openapi["paths"].as_object() {
        for (path, methods) in paths {
            if let Some(methods_obj) = methods.as_object() {
                for (method, operation) in methods_obj {
                    let endpoint_html =
                        generate_single_endpoint(tt, path, method, operation, openapi)?;
                    endpoints_html.push_str(&endpoint_html);
                }
            }
        }
    }

    Ok(endpoints_html)
}

/// Generate HTML for a single API endpoint
fn generate_single_endpoint(
    tt: &TinyTemplate,
    path: &str,
    method: &str,
    operation: &Value,
    openapi: &Value,
) -> Result<String> {
    let status_codes_html = generate_status_codes_section(operation, tt);
    let request_body_html = generate_request_body_section(operation, openapi, tt);

    let context = create_endpoint_context(
        path,
        method,
        operation,
        &status_codes_html,
        &request_body_html,
    );

    tt.render("endpoint", &context)
        .with_context(|| format!("Failed to render endpoint template for {} {}", method, path))
}

/// Generate the status codes section for an endpoint
fn generate_status_codes_section(operation: &Value, tt: &TinyTemplate) -> String {
    let mut html = String::new();

    if let Some(responses) = operation["responses"].as_object() {
        for (code, response) in responses {
            let desc = response["description"].as_str().unwrap_or("");
            let examples_html = generate_response_examples(response, tt);

            let mut context = HashMap::new();
            context.insert("code".to_string(), code.to_string());
            context.insert(
                "status_text".to_string(),
                get_http_status_text(code).to_string(),
            );
            context.insert("description".to_string(), desc.to_string());
            context.insert("examples".to_string(), examples_html);

            if let Ok(rendered) = tt.render("status_code", &context) {
                html.push_str(&rendered);
            }
        }
    }

    html
}

/// Generate examples for a response
fn generate_response_examples(response: &Value, tt: &TinyTemplate) -> String {
    let mut examples_html = String::new();

    if let Some(content) = response["content"].as_object() {
        for (content_type, content_data) in content {
            if let Some(example) = content_data["example"].as_str() {
                examples_html.push_str(&format_single_example(content_type, example, tt));
            } else if let Some(examples) = content_data["examples"].as_object() {
                examples_html.push_str(&format_multiple_examples(content_type, examples, tt));
            }
        }
    }

    examples_html
}

/// Format a single example
fn format_single_example(content_type: &str, example: &str, tt: &TinyTemplate) -> String {
    let mut context = HashMap::new();
    context.insert("content_type".to_string(), content_type.to_string());
    context.insert("example".to_string(), html_escape(example));

    tt.render("example_single", &context).unwrap_or_default()
}

/// Format multiple examples
fn format_multiple_examples(
    content_type: &str,
    examples: &serde_json::Map<String, Value>,
    tt: &TinyTemplate,
) -> String {
    let mut example_items = String::new();

    for (example_name, example_data) in examples {
        let summary = example_data["summary"].as_str().unwrap_or(example_name);
        let value = example_data["value"].as_str().unwrap_or("");

        let mut item_context = HashMap::new();
        item_context.insert("summary".to_string(), html_escape(summary));
        item_context.insert("value".to_string(), html_escape(value));

        if let Ok(rendered) = tt.render("example_item", &item_context) {
            example_items.push_str(&rendered);
        }
    }

    let mut context = HashMap::new();
    context.insert("content_type".to_string(), content_type.to_string());
    context.insert("examples".to_string(), example_items);

    tt.render("example_multiple", &context).unwrap_or_default()
}

/// Generate the request body section for an endpoint
fn generate_request_body_section(operation: &Value, _openapi: &Value, tt: &TinyTemplate) -> String {
    let mut html = String::new();

    if let Some(request_body) = operation["requestBody"].as_object() {
        html.push_str(
            &tt.render("request_body_header", &HashMap::<String, String>::new())
                .unwrap_or_default(),
        );

        // Add schema reference link
        let request_body_value = Value::Object(request_body.clone());
        html.push_str(&generate_schema_reference(&request_body_value, tt));

        // Add examples
        html.push_str(&generate_request_examples(&request_body_value, tt));
    }

    html
}

/// Generate schema reference link for request body
fn generate_schema_reference(request_body: &Value, tt: &TinyTemplate) -> String {
    if let Some(content) = request_body["content"]["application/json"].as_object()
        && let Some(schema) = content["schema"].as_object()
        && let Some(ref_str) = schema["$ref"].as_str()
    {
        let schema_name = extract_schema_name(ref_str);
        let mut context = HashMap::new();
        context.insert("schema_id".to_string(), schema_name.to_lowercase());
        context.insert("schema_name".to_string(), schema_name.to_string());
        return tt.render("schema_reference", &context).unwrap_or_default();
    }
    String::new()
}

/// Generate examples for request body
fn generate_request_examples(request_body: &Value, tt: &TinyTemplate) -> String {
    let mut html = String::new();

    if let Some(content) = request_body["content"]["application/json"].as_object()
        && let Some(examples) = content.get("examples").and_then(|v| v.as_object())
    {
        if examples.len() == 1 {
            // Single example - use existing template
            if let Some(first_example) = examples.values().next()
                && let Some(example_value) = first_example.get("value")
            {
                let mut context = HashMap::new();
                context.insert("example".to_string(), format_json_pretty(example_value));
                html.push_str(&tt.render("example_single", &context).unwrap_or_default());
            }
        } else {
            // Multiple examples
            html.push_str(
                &tt.render("examples_header", &HashMap::<String, String>::new())
                    .unwrap_or_default(),
            );
            for (example_name, example_data) in examples {
                let summary = example_data["summary"].as_str().unwrap_or(example_name);
                if let Some(example_value) = example_data.get("value") {
                    let mut context = HashMap::new();
                    context.insert("summary".to_string(), html_escape(summary));
                    context.insert("value".to_string(), format_json_pretty(example_value));
                    html.push_str(&tt.render("example_item", &context).unwrap_or_default());
                }
            }
        }
    }

    html
}

/// Generate the schemas section
fn generate_schemas_section(openapi: &Value, tt: &TinyTemplate) -> String {
    if let Some(components) = openapi["components"].as_object()
        && let Some(schemas) = components["schemas"].as_object() {
            let mut schemas_html = String::new();

            for (schema_name, schema) in schemas {
                schemas_html.push_str(&generate_single_schema(schema_name, schema, tt));
            }

            let mut context = HashMap::new();
            context.insert("schemas".to_string(), schemas_html);

            return tt.render("schema_section", &context).unwrap_or_default();
        }

    String::new()
}

/// Generate HTML for a single schema definition
fn generate_single_schema(name: &str, schema: &Value, tt: &TinyTemplate) -> String {
    let mut context = HashMap::new();
    context.insert("schema_id".to_string(), name.to_lowercase());
    context.insert("schema_name".to_string(), name.to_string());

    // Add description if available
    let description = if let Some(desc) = schema["description"].as_str() {
        let mut desc_context = HashMap::new();
        desc_context.insert("description".to_string(), html_escape(desc));
        tt.render("schema_description", &desc_context)
            .unwrap_or_default()
    } else {
        String::new()
    };
    context.insert("description".to_string(), description);

    // Add properties
    context.insert(
        "properties".to_string(),
        generate_schema_properties(schema, tt),
    );

    tt.render("schema_definition", &context).unwrap_or_default()
}

/// Generate properties section for a schema
fn generate_schema_properties(schema: &Value, tt: &TinyTemplate) -> String {
    if let Some(properties) = schema["properties"].as_object() {
        let mut properties_html = String::new();
        let required_fields = get_required_fields(schema);

        for (prop_name, prop_schema) in properties {
            properties_html.push_str(&generate_single_property(
                prop_name,
                prop_schema,
                &required_fields,
                tt,
            ));
        }

        let mut props_context = HashMap::new();
        props_context.insert("properties".to_string(), properties_html);
        tt.render("schema_properties_header", &props_context)
            .unwrap_or_default()
    } else {
        String::new()
    }
}

/// Generate HTML for a single property
fn generate_single_property(
    name: &str,
    prop_schema: &Value,
    required_fields: &[&str],
    tt: &TinyTemplate,
) -> String {
    let is_required = required_fields.contains(&name);
    let type_str = get_property_type_string(prop_schema);
    let description = prop_schema["description"].as_str().unwrap_or("");
    let example_html = generate_property_example(prop_schema, tt);

    let mut context = HashMap::new();
    context.insert("property_name".to_string(), name.to_string());
    context.insert(
        "required_marker".to_string(),
        if is_required {
            " <em>(required)</em>".to_string()
        } else {
            String::new()
        },
    );
    context.insert("property_type".to_string(), type_str);
    context.insert("description".to_string(), html_escape(description));
    context.insert("example".to_string(), example_html);

    tt.render("schema_property", &context).unwrap_or_default()
}

/// Get required fields list from schema
fn get_required_fields(schema: &Value) -> Vec<&str> {
    schema["required"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default()
}

/// Get property type as string
fn get_property_type_string(prop_schema: &Value) -> String {
    if let Some(type_str) = prop_schema["type"].as_str() {
        match type_str {
            "array" => format_array_type(prop_schema),
            _ => type_str.to_string(),
        }
    } else if let Some(ref_str) = prop_schema["$ref"].as_str() {
        extract_schema_name(ref_str)
    } else {
        "unknown".to_string()
    }
}

/// Format array type string
fn format_array_type(prop_schema: &Value) -> String {
    if let Some(items) = prop_schema["items"].as_object() {
        if let Some(item_type) = items["type"].as_str() {
            format!("array of {}", item_type)
        } else if items["$ref"].is_string() {
            "array of objects".to_string()
        } else {
            "array".to_string()
        }
    } else {
        "array".to_string()
    }
}

/// Generate example HTML for a property
fn generate_property_example(prop_schema: &Value, tt: &TinyTemplate) -> String {
    if let Some(example) = prop_schema["example"].as_str() {
        let mut context = HashMap::new();
        context.insert("example".to_string(), html_escape(example));
        tt.render("property_example", &context).unwrap_or_default()
    } else if let Some(example_array) = prop_schema["example"].as_array() {
        let examples_str = example_array
            .iter()
            .filter_map(|v| v.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        let mut context = HashMap::new();
        context.insert(
            "example".to_string(),
            format!("[{}]", html_escape(&examples_str)),
        );
        tt.render("property_example", &context).unwrap_or_default()
    } else {
        String::new()
    }
}

// Context creation functions

/// Create the main documentation context
fn create_docs_context(
    openapi: &Value,
    endpoints_html: &str,
    schemas_html: &str,
) -> HashMap<String, String> {
    let info = &openapi["info"];
    let mut context = HashMap::new();

    context.insert("title".to_string(), get_api_title(info));
    context.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    context.insert("description".to_string(), get_api_description(info));
    context.insert("endpoints".to_string(), endpoints_html.to_string());
    context.insert("schemas".to_string(), schemas_html.to_string());
    context.insert("cache_buster".to_string(), generate_cache_buster());

    context
}

/// Create context for a single endpoint
fn create_endpoint_context(
    path: &str,
    method: &str,
    operation: &Value,
    status_codes_html: &str,
    request_body_html: &str,
) -> HashMap<String, String> {
    let mut context = HashMap::new();

    context.insert(
        "summary".to_string(),
        html_escape_value(&operation["summary"]),
    );
    context.insert("method_class".to_string(), method.to_lowercase());
    context.insert("method_upper".to_string(), method.to_uppercase());
    context.insert("path".to_string(), path.to_string());
    context.insert(
        "description".to_string(),
        html_escape_value(&operation["description"]),
    );
    context.insert("request_body".to_string(), request_body_html.to_string());
    context.insert("status_codes".to_string(), status_codes_html.to_string());

    context
}

// Utility functions

/// Extract schema name from $ref string
fn extract_schema_name(ref_str: &str) -> String {
    ref_str
        .split('/')
        .next_back()
        .unwrap_or("Schema")
        .to_string()
}

/// Get API title from info section
fn get_api_title(info: &Value) -> String {
    info["title"]
        .as_str()
        .unwrap_or("API Documentation")
        .to_string()
}

/// Get API description from info section
fn get_api_description(info: &Value) -> String {
    info["description"].as_str().unwrap_or("").to_string()
}

/// Format JSON value as pretty-printed string
fn format_json_pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_default()
}

/// Get HTTP status text for a status code
fn get_http_status_text(code: &str) -> &'static str {
    match code {
        "200" => "OK",
        "400" => "Bad Request",
        "401" => "Unauthorized",
        "403" => "Forbidden",
        "404" => "Not Found",
        "410" => "Gone",
        "501" => "Not Implemented",
        _ => "",
    }
}

/// HTML escape a string
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

/// HTML escape a JSON value (string only)
fn html_escape_value(input: &Value) -> String {
    input.as_str().unwrap_or("").to_string()
}

// Template partials handling

struct TemplatePartials {
    head: String,
    theme_switcher: String,
    language_selector: String,
    footer: String,
    header: String,
    ttl_selector: String,
}

/// Load all template partials
fn load_partials() -> Result<TemplatePartials> {
    Ok(TemplatePartials {
        head: fs::read_to_string("src/templates/partials/head.html")
            .context("Failed to read head partial")?,
        theme_switcher: fs::read_to_string("src/templates/partials/theme-switcher.html")
            .context("Failed to read theme-switcher partial")?,
        language_selector: fs::read_to_string("src/templates/partials/language-selector.html")
            .context("Failed to read language-selector partial")?,
        footer: fs::read_to_string("src/templates/partials/footer.html")
            .context("Failed to read footer partial")?,
        header: fs::read_to_string("src/templates/partials/header.html")
            .context("Failed to read header partial")?,
        ttl_selector: fs::read_to_string("src/templates/partials/ttl-selector.html")
            .context("Failed to read ttl-selector partial")?,
    })
}

/// Apply partials to template content
fn apply_partials(template_content: String, partials: &TemplatePartials) -> String {
    template_content
        .replace("[[HEAD]]", &partials.head)
        .replace("[[THEME_SWITCHER]]", &partials.theme_switcher)
        .replace("[[LANGUAGE_SELECTOR]]", &partials.language_selector)
        .replace("[[FOOTER]]", &partials.footer)
        .replace("[[HEADER]]", &partials.header)
        .replace("[[TTL_SELECTOR]]", &partials.ttl_selector)
}

/// Generate cache buster string based on file modification times
fn generate_cache_buster() -> String {
    use std::time::SystemTime;

    let get_latest_modified = |path: &str, ext: &str| -> SystemTime {
        let mut latest = SystemTime::UNIX_EPOCH;
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if entry.path().extension().is_some_and(|e| e == ext)
                    && let Ok(metadata) = entry.metadata()
                    && let Ok(modified) = metadata.modified()
                {
                    latest = latest.max(modified);
                }
            }
        }
        latest
    };

    let typescript_modified = get_latest_modified("src/typescript", "ts");
    let includes_modified = get_latest_modified("src/includes", "css");
    let templates_modified = get_latest_modified("src/templates", "html");

    [typescript_modified, includes_modified, templates_modified]
        .iter()
        .max()
        .unwrap_or(&SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
