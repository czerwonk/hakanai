// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Cache lock poisoned")]
    MutexPoisoned,
}

pub struct AssetManager {
    override_dir: Option<PathBuf>,
    cache: Mutex<HashMap<String, Vec<u8>>>,
}

impl AssetManager {
    /// Create a new AssetManager with an optional override directory.
    pub fn new(override_path: Option<PathBuf>) -> Self {
        AssetManager {
            override_dir: override_path,
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Get the embedded asset or an override if it exists.
    pub async fn get_embedded_asset_or_override(
        &self,
        name: &str,
        original_content: &[u8],
    ) -> Result<Vec<u8>, AssetError> {
        // Check cache first
        if let Some(cached) = self.asset_from_cache(name).await? {
            return Ok(cached.clone());
        }

        let override_content = self.load_override(name).await?;
        if let Some(content) = override_content {
            self.insert_into_cache(name, content.clone()).await?;
            return Ok(content);
        }

        Ok(original_content.to_vec())
    }

    /// Get the embedded asset and append any override content if it exists.
    pub async fn get_embedded_asset_append_override(
        &self,
        name: &str,
        original_content: &[u8],
    ) -> Result<Vec<u8>, AssetError> {
        if let Some(cached) = self.asset_from_cache(name).await? {
            return Ok(cached.clone());
        }

        let mut result_content = original_content.to_vec();

        let override_content = self.load_override(name).await?;
        if let Some(content) = override_content {
            result_content.extend_from_slice(&content);
            self.insert_into_cache(name, result_content.clone()).await?;
        }

        Ok(result_content)
    }

    async fn asset_from_cache(&self, name: &str) -> Result<Option<Vec<u8>>, AssetError> {
        let cache = self.cache.lock().map_err(|_| AssetError::MutexPoisoned)?;
        Ok(cache.get(name).cloned())
    }

    async fn insert_into_cache(&self, name: &str, content: Vec<u8>) -> Result<(), AssetError> {
        let mut cache = self.cache.lock().map_err(|_| AssetError::MutexPoisoned)?;
        cache.insert(name.to_string(), content);
        Ok(())
    }

    async fn load_override(&self, name: &str) -> Result<Option<Vec<u8>>, std::io::Error> {
        if let Some(dir) = &self.override_dir {
            let path = dir.join(name);

            if path.exists() {
                let content = tokio::fs::read(path).await?;
                return Ok(Some(content));
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(
        dir: &TempDir,
        name: &str,
        content: &[u8],
    ) -> Result<PathBuf, std::io::Error> {
        let path = dir.path().join(name);
        fs::write(&path, content)?;
        Ok(path)
    }

    #[tokio::test]
    async fn test_get_embedded_asset_no_override() -> Result<(), AssetError> {
        let manager = AssetManager::new(None);
        let original = b"original content";

        let result = manager
            .get_embedded_asset_or_override("test.txt", original)
            .await?;
        assert_eq!(
            result, original,
            "Should return original content when no override directory is set"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_embedded_asset_with_override() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_file(&temp_dir, "logo.svg", b"custom logo content")?;

        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"original logo";

        let result = manager
            .get_embedded_asset_or_override("logo.svg", original)
            .await?;
        assert_eq!(
            result, b"custom logo content",
            "Should return override content instead of original when override file exists"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_embedded_asset_caching() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_file(&temp_dir, "favicon.ico", b"custom favicon")?;

        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"original favicon";

        // First call loads from file
        let result1 = manager
            .get_embedded_asset_or_override("favicon.ico", original)
            .await?;
        assert_eq!(
            result1, b"custom favicon",
            "First call should load override from filesystem"
        );

        // Delete the file to ensure second call uses cache
        fs::remove_file(temp_dir.path().join("favicon.ico"))?;

        // Second call should use cache
        let result2 = manager
            .get_embedded_asset_or_override("favicon.ico", original)
            .await?;
        assert_eq!(
            result2, b"custom favicon",
            "Second call should return cached content even after file deletion"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_embedded_asset_append_no_override() -> Result<(), AssetError> {
        let manager = AssetManager::new(None);
        let original = b"/* original styles */";

        let result = manager
            .get_embedded_asset_append_override("style.css", original)
            .await?;
        assert_eq!(
            result, original,
            "Should return original content when no override exists"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_embedded_asset_append_with_override() -> Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = TempDir::new()?;
        create_test_file(&temp_dir, "style.css", b"\n/* custom styles */")?;

        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"/* original styles */";

        let result = manager
            .get_embedded_asset_append_override("style.css", original)
            .await?;
        assert_eq!(
            result, b"/* original styles */\n/* custom styles */",
            "Should append override content to original content"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_embedded_asset_append_caching() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_file(&temp_dir, "style.css", b"\n/* custom */")?;

        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"/* base */";

        // First call combines and caches
        let result1 = manager
            .get_embedded_asset_append_override("style.css", original)
            .await?;
        assert_eq!(
            result1, b"/* base */\n/* custom */",
            "First call should combine original and override content"
        );

        // Delete file to ensure cache is used
        fs::remove_file(temp_dir.path().join("style.css"))?;

        // Second call uses cache
        let result2 = manager
            .get_embedded_asset_append_override("style.css", original)
            .await?;
        assert_eq!(
            result2, b"/* base */\n/* custom */",
            "Second call should return cached combined content"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_nonexistent_override_directory() -> Result<(), AssetError> {
        let manager = AssetManager::new(Some(PathBuf::from("/nonexistent/path")));
        let original = b"original";

        let result = manager
            .get_embedded_asset_or_override("test.txt", original)
            .await?;
        assert_eq!(
            result, original,
            "Should return original content when override directory doesn't exist"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_nonexistent_override_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"original";

        let result = manager
            .get_embedded_asset_or_override("missing.txt", original)
            .await?;
        assert_eq!(
            result, original,
            "Should return original content when override file doesn't exist in directory"
        );
        Ok(())
    }
}
