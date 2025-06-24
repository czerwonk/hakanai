use std::collections::HashMap;
use std::io::Error;
use std::time::Duration;

use uuid::Uuid;

pub trait DataStore: Send + Sync {
    fn get(&self, id: uuid::Uuid) -> Result<Option<String>, Error>;
    fn put(&mut self, data: String, expires_in: Option<Duration>) -> Result<uuid::Uuid, Error>;
}

#[derive(Clone)]
pub struct InMemoryDataStore {
    data: HashMap<uuid::Uuid, String>,
}

impl InMemoryDataStore {
    pub fn new() -> Self {
        InMemoryDataStore {
            data: HashMap::new(),
        }
    }
}

impl DataStore for InMemoryDataStore {
    fn get(&self, id: uuid::Uuid) -> Result<Option<String>, Error> {
        match self.data.get(&id) {
            Some(data) => Ok(Some(data.clone())),
            None => Ok(None),
        }
    }

    fn put(&mut self, data: String, _expires_in: Option<Duration>) -> Result<uuid::Uuid, Error> {
        let id = Uuid::new_v4();
        self.data.insert(id, data);
        Ok(id)
    }
}
