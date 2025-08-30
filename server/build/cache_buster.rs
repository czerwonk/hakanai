// SPDX-License-Identifier: Apache-2.0

use std::fs;
use std::time::SystemTime;

/// Generates a cache buster string based on the latest modified times
pub fn generate() -> String {
    let typescript_modified = get_latest_modified_time("typescript", "ts");
    let includes_modified = get_latest_modified_time("includes", "css");
    let templates_modified = get_latest_modified_time("templates", "html");

    [typescript_modified, includes_modified, templates_modified]
        .iter()
        .max()
        .unwrap_or(&SystemTime::UNIX_EPOCH)
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
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
