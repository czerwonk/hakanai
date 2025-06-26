use crate::data_store::DataStore;

/// AppData stores the application's shared state.
pub struct AppData {
    /// The data store for persisting application data.
    pub data_store: Box<dyn DataStore>,

    /// A list of valid authentication tokens.
    pub tokens: Vec<String>,
}
