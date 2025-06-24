use std::collections::HashMap;
use std::io::Error;
use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use uuid::Uuid;

/// `DataStore` is a trait that defines the contract for a simple, asynchronous,
/// key-value storage system. Implementations of this trait are expected to be
/// thread-safe.
#[async_trait]
pub trait DataStore: Send + Sync {
    /// Retrieves a value from the data store based on its `Uuid`.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` of the item to retrieve.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(Some(String))` if the item is found, `Ok(None)`
    /// if the item is not found, or an `Err` if an error occurs during the
    /// operation.
    async fn get(&self, id: Uuid) -> Result<Option<String>, Error>;

    /// Stores a value in the data store with a given `Uuid` and an expiration
    /// duration.
    ///
    /// # Arguments
    ///
    /// * `id` - The `Uuid` to use as the key for the stored data.
    /// * `data` - The `String` data to store.
    /// * `expires_in` - A `Duration` after which the stored item should be
    ///   considered expired. Note that the implementation of the data store
    ///   determines how expiration is handled.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` on successful insertion, or an `Err` if an
    /// error occurs.
    async fn put(&self, id: Uuid, data: String, expires_in: Duration) -> Result<(), Error>;
}

/// An in-memory implementation of the `DataStore` trait.
///
/// This implementation uses a `std::collections::HashMap` protected by a
/// `std::sync::Mutex` to store key-value pairs in memory. It is intended for
/// use in environments where a simple, non-persistent data store is sufficient,
/// such as for testing or in applications with short-lived data.
///
/// Note that this implementation does not enforce item expiration. The
/// `expires_in` parameter of the `put` method is ignored.
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
    use tokio;

    #[tokio::test]
    async fn test_in_memory_store_new() {
        let store = InMemoryDataStore::new();
        let data = store.data.lock().unwrap();
        assert!(data.is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_store_put_and_get() {
        let store = InMemoryDataStore::new();
        let id = Uuid::new_v4();
        let data = "test_data".to_string();
        let expires_in = Duration::from_secs(3600);

        let result = store.put(id, data.clone(), expires_in).await;
        assert!(result.is_ok());

        let get_result = store.get(id).await;
        assert!(get_result.is_ok());
        assert_eq!(get_result.unwrap(), Some(data));
    }

    #[tokio::test]
    async fn test_in_memory_store_get_non_existent() {
        let store = InMemoryDataStore::new();
        let id = Uuid::new_v4();

        let result = store.get(id).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[tokio::test]
    async fn test_in_memory_store_multiple_entries() {
        let store = InMemoryDataStore::new();
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        let data1 = "data1".to_string();
        let data2 = "data2".to_string();

        store
            .put(id1, data1.clone(), Duration::from_secs(60))
            .await
            .unwrap();
        store
            .put(id2, data2.clone(), Duration::from_secs(120))
            .await
            .unwrap();

        let result1 = store.get(id1).await.unwrap();
        let result2 = store.get(id2).await.unwrap();

        assert_eq!(result1, Some(data1));
        assert_eq!(result2, Some(data2));
    }

    #[tokio::test]
    async fn test_in_memory_store_overwrite() {
        let store = InMemoryDataStore::new();
        let id = Uuid::new_v4();
        let data1 = "original".to_string();
        let data2 = "updated".to_string();

        store.put(id, data1, Duration::from_secs(60)).await.unwrap();
        store
            .put(id, data2.clone(), Duration::from_secs(120))
            .await
            .unwrap();

        let result = store.get(id).await.unwrap();
        assert_eq!(result, Some(data2));
    }

    #[tokio::test]
    async fn test_in_memory_store_thread_safety() {
        use std::sync::Arc;
        use tokio::task;

        let store = Arc::new(InMemoryDataStore::new());
        let store1 = store.clone();
        let store2 = store.clone();

        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        let handle1 = task::spawn(async move {
            store1
                .put(id1, "data1".to_string(), Duration::from_secs(60))
                .await
        });

        let handle2 = task::spawn(async move {
            store2
                .put(id2, "data2".to_string(), Duration::from_secs(60))
                .await
        });

        let result1 = handle1.await.unwrap();
        let result2 = handle2.await.unwrap();

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let get1 = store.get(id1).await.unwrap();
        let get2 = store.get(id2).await.unwrap();

        assert_eq!(get1, Some("data1".to_string()));
        assert_eq!(get2, Some("data2".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_store_ignores_expiration() {
        let store = InMemoryDataStore::new();
        let id = Uuid::new_v4();
        let data = "test_data".to_string();

        store
            .put(id, data.clone(), Duration::from_millis(1))
            .await
            .unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;

        let result = store.get(id).await.unwrap();
        assert_eq!(result, Some(data));
    }
}
