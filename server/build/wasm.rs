// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::process::Command;

use anyhow::Result;

/// Compile the WASM module using wasm-pack
pub fn compile() -> Result<()> {
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
            "includes/hakanai_wasm_bg.wasm",
        ),
        ("../wasm/pkg/hakanai_wasm.js", "includes/hakanai_wasm.js"),
    ];

    for (src, dst) in &wasm_files {
        if fs::copy(src, dst).is_ok() {
            println!("cargo:warning=Copied {src} to {dst}");
        }
    }

    println!("cargo:warning=WASM module built successfully");
    Ok(())
}
