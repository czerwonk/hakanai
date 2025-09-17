// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Debug, str::FromStr};

/// Trait for parsing string literals into types with better ergonomics in tests
pub trait MustParse {
    /// Parse the string into a generic implementing FromStr, panicking if parsing fails
    ///
    /// # Examples
    /// ```
    /// use hakanai_lib::utils::test::MustParse;
    /// let ip: ipnet::IpNet = "127.0.0.1/32".must_parse();
    /// ```
    fn must_parse<T: FromStr>(&self) -> T
    where
        <T as FromStr>::Err: Debug;
}

impl MustParse for &str {
    fn must_parse<T: FromStr>(&self) -> T
    where
        <T as FromStr>::Err: Debug,
    {
        self.parse()
            .unwrap_or_else(|_| panic!("Failed to parse: {self}"))
    }
}

impl MustParse for String {
    fn must_parse<T: FromStr>(&self) -> T
    where
        <T as FromStr>::Err: Debug,
    {
        self.as_str().must_parse()
    }
}
