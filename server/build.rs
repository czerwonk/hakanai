// SPDX-License-Identifier: Apache-2.0

use std::fs;

use anyhow::{Context, Result};

mod build {
    pub mod cache_buster;
    pub mod docs_generator;
    pub mod static_pages;
    pub mod typescript;
    pub mod wasm;
}

use build::docs_generator;
use build::static_pages;
use build::typescript;
use build::wasm;

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
    println!("cargo:rerun-if-changed=includes/openapi.json");

    register_files_for_recompilation("templates", "html")?;
    register_files_for_recompilation("templates/docs", "html")?;
    register_files_for_recompilation("templates/partials", "html")?;
    register_files_for_recompilation("../typescript", "ts")?;
    println!("cargo:rerun-if-changed=../typescript/tsconfig.json");
    println!("cargo:rerun-if-changed=../typescript/rollup.config.js");
    println!("cargo:rerun-if-changed=../typescript/package.json");
    println!("cargo:rerun-if-changed=../wasm/src/lib.rs");
    println!("cargo:rerun-if-changed=../wasm/Cargo.toml");

    let start = std::time::Instant::now();
    typescript::compile()?;
    wasm::compile()?;
    docs_generator::generate()?;
    static_pages::generate_html_files()?;
    println!("cargo:warning=Build completed in {:?}", start.elapsed());

    Ok(())
}
