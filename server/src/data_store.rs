use std::collections::HashMap;
use std::io::Error;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait DataStore: Send + Sync {
    async fn get(&self, id: Uuid) -> Result<Option<String>, Error>;
    async fn put(&self, id: Uuid, data: String, expires_in: Duration) -> Result<(), Error>;
}

pub struct InMemoryDataStore {
    data: Mutex<HashMap<uuid::Uuid, String>>,
}

impl InMemoryDataStore {
    pub fn new() -> Self {
        InMemoryDataStore {
            data: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl DataStore for InMemoryDataStore {
    async fn get(&self, id: Uuid) -> Result<Option<String>, Error> {
        let d = self.data.lock().unwrap();
        match d.get(&id) {
            Some(data) => Ok(Some(data.clone())),
            None => Ok(None),
        }
    }

    async fn put(&self, id: Uuid, data: String, _expires_in: Duration) -> Result<(), Error> {
        let mut d = self.data.lock().unwrap();
        d.insert(id, data);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_put_and_get() {
        let store = InMemoryDataStore::new();
        let data = "test data".to_string();
        let id = Uuid::new_v4();
        store
            .put(id, data.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        let retrieved = store.get(id).await.unwrap().unwrap();
        assert_eq!(data, retrieved);
    }
}
