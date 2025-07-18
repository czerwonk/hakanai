use std::time;

use std::collections::HashMap;

use crate::data_store::DataStore;

/// AppData stores the application's shared state.
pub struct AppData {
    /// The data store for persisting application data.
    pub data_store: Box<dyn DataStore>,

    pub tokens: HashMap<String, ()>,

    /// The maximum time-to-live (TTL) for secrets
    pub max_ttl: time::Duration,
}
