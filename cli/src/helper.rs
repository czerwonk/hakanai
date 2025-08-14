// SPDX-License-Identifier: MIT

/// Returns the user agent name for the CLI application.
pub fn get_user_agent_name() -> String {
    format!("hakanai-cli/{}", env!("CARGO_PKG_VERSION"))
}
