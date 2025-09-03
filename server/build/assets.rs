// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::process::Command;

use anyhow::{Context, Result};

use super::cache_buster;

// Build asset files (WASM, JS)
pub fn build() -> Result<()> {
    println!("cargo:warning=Building assets...");

    if std::env::var("SKIP_ASSET_BUILD").is_ok() {
        println!("cargo:warning=Skipping asset build (SKIP_ASSET_BUILD set)");
        return Ok(());
    }

    // Bundle TypeScript files with Rollup
    let output = Command::new("make")
        .args(["build-ts"])
        .current_dir("../")
        .output()?;
    println!("cargo:warning=Output: {:?}", output);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("cargo:warning=Rollup bundling failed");
        println!("cargo:warning=STDOUT: {stdout}");
        println!("cargo:warning=STDERR: {stderr}");
        panic!("Rollup bundling failed: {stderr}");
    }

    add_cache_busters_to_asset_files()?;

    println!("cargo:warning=TypeScript bundling successful");
    Ok(())
}

fn add_cache_busters_to_asset_files() -> Result<()> {
    println!("cargo:warning=Adding cache busters to JavaScript imports and JSON URLs...");

    let cache_buster = cache_buster::generate();

    // Find all .js files in includes/
    let includes_dir = std::path::Path::new("includes");
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
