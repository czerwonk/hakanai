// SPDX-License-Identifier: Apache-2.0

//! Serde utility functions for custom serialization and deserialization.

use serde::{Deserialize, Deserializer};

/// Custom deserializer for converting JSON string arrays to Vec<ipnet::IpNet>
pub fn deserialize_ip_nets<'de, D>(deserializer: D) -> Result<Option<Vec<ipnet::IpNet>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    // Handle both Vec<String> and null/missing cases
    let strings_opt = Option::<Vec<String>>::deserialize(deserializer)?;

    match strings_opt {
        Some(strings) => {
            let mut ip_nets = Vec::new();
            for s in strings {
                let ip_net =
                    crate::utils::ip_parser::parse_ipnet(&s).map_err(|e| Error::custom(e))?;
                ip_nets.push(ip_net);
            }
            Ok(Some(ip_nets))
        }
        None => Ok(None),
    }
}
