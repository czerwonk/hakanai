// SPDX-License-Identifier: Apache-2.0

/// User type for authentication and tracing
#[derive(Clone, Debug)]
pub enum UserType {
    Anonymous,
    Authenticated,
    Whitelisted,
}

impl std::fmt::Display for UserType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserType::Anonymous => write!(f, "anonymous"),
            UserType::Authenticated => write!(f, "authenticated"),
            UserType::Whitelisted => write!(f, "whitelisted"),
        }
    }
}
