// SPDX-License-Identifier: Apache-2.0

use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current timestamp in seconds since the Unix epoch.
pub fn now_string() -> Result<String, std::time::SystemTimeError> {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH)?;
    Ok(format!("{}", duration.as_secs()))
}
