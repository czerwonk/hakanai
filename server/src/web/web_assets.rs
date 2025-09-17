// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;
use std::path::PathBuf;

use tokio::sync::RwLock;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct AssetManager {
    custom_dir: Option<PathBuf>,
    cache: RwLock<HashMap<String, Vec<u8>>>,
}

impl AssetManager {
    /// Create a new AssetManager with an optional custom directory.
    pub fn new(custom_dir: Option<PathBuf>) -> Self {
        AssetManager {
            custom_dir,
            cache: RwLock::new(HashMap::new()),
        }
    }

    /// Get the embedded asset or an custom asset if it exists.
    pub async fn get_embedded_asset_or_custom(
        &self,
        name: &str,
        original_content: &[u8],
    ) -> Result<Vec<u8>, AssetError> {
        if let Some(cached) = self.asset_from_cache(name).await? {
            return Ok(cached.clone());
        }

        if let Some(content) = self.get_custom(name).await? {
            self.insert_into_cache(name, content.clone()).await?;
            return Ok(content);
        }

        Ok(original_content.to_vec())
    }

    /// Get the embedded asset and append any custom content if it exists.
    pub async fn get_embedded_asset_append_custom(
        &self,
        name: &str,
        original_content: &[u8],
    ) -> Result<Vec<u8>, AssetError> {
        if let Some(cached) = self.asset_from_cache(name).await? {
            return Ok(cached.clone());
        }

        let mut result_content = original_content.to_vec();

        if let Some(content) = self.get_custom(name).await? {
            result_content.push(b'\n');
            result_content.extend_from_slice(&content);
            self.insert_into_cache(name, result_content.clone()).await?;
        }

        Ok(result_content)
    }

    async fn asset_from_cache(&self, name: &str) -> Result<Option<Vec<u8>>, AssetError> {
        let cache = self.cache.read().await;
        Ok(cache.get(name).cloned())
    }

    async fn insert_into_cache(&self, name: &str, content: Vec<u8>) -> Result<(), AssetError> {
        let mut cache = self.cache.write().await;
        cache.insert(name.to_string(), content);
        Ok(())
    }

    async fn get_custom(&self, name: &str) -> Result<Option<Vec<u8>>, std::io::Error> {
        if let Some(dir) = &self.custom_dir {
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
    async fn test_get_or_custom_no_override() -> Result<(), AssetError> {
        let manager = AssetManager::new(None);
        let original = b"original content";

        let result = manager
            .get_embedded_asset_or_custom("test.txt", original)
            .await?;
        assert_eq!(
            result, original,
            "Should return original content when no override directory is set"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_custom_with_override() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_file(&temp_dir, "logo.svg", b"custom logo content")?;

        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"original logo";

        let result = manager
            .get_embedded_asset_or_custom("logo.svg", original)
            .await?;
        assert_eq!(
            result, b"custom logo content",
            "Should return override content instead of original when override file exists"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_custom_caching() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_file(&temp_dir, "favicon.ico", b"custom favicon")?;

        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"original favicon";

        // First call loads from file
        let result1 = manager
            .get_embedded_asset_or_custom("favicon.ico", original)
            .await?;
        assert_eq!(
            result1, b"custom favicon",
            "First call should load override from filesystem"
        );

        // Delete the file to ensure second call uses cache
        fs::remove_file(temp_dir.path().join("favicon.ico"))?;

        // Second call should use cache
        let result2 = manager
            .get_embedded_asset_or_custom("favicon.ico", original)
            .await?;
        assert_eq!(
            result2, b"custom favicon",
            "Second call should return cached content even after file deletion"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_and_append_no_custom() -> Result<(), AssetError> {
        let manager = AssetManager::new(None);
        let original = b"/* original styles */";

        let result = manager
            .get_embedded_asset_append_custom("style.css", original)
            .await?;
        assert_eq!(
            result, original,
            "Should return original content when no override exists"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_and_append_with_custom() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_file(&temp_dir, "style.css", b"/* custom styles */")?;

        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"/* original styles */";

        let result = manager
            .get_embedded_asset_append_custom("style.css", original)
            .await?;
        assert_eq!(
            result, b"/* original styles */\n/* custom styles */",
            "Should append override content to original content"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_and_append_caching() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        create_test_file(&temp_dir, "style.css", b"/* custom */")?;

        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"/* base */";

        // First call combines and caches
        let result1 = manager
            .get_embedded_asset_append_custom("style.css", original)
            .await?;
        assert_eq!(
            result1, b"/* base */\n/* custom */",
            "First call should combine original and override content"
        );

        // Delete file to ensure cache is used
        fs::remove_file(temp_dir.path().join("style.css"))?;

        // Second call uses cache
        let result2 = manager
            .get_embedded_asset_append_custom("style.css", original)
            .await?;
        assert_eq!(
            result2, b"/* base */\n/* custom */",
            "Second call should return cached combined content"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_custom_nonexistent_dir() -> Result<(), AssetError> {
        let manager = AssetManager::new(Some(PathBuf::from("/nonexistent/path")));
        let original = b"original";

        let result = manager
            .get_embedded_asset_or_custom("test.txt", original)
            .await?;
        assert_eq!(
            result, original,
            "Should return original content when override directory doesn't exist"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_get_or_custom_nonexistent_file() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let manager = AssetManager::new(Some(temp_dir.path().to_path_buf()));
        let original = b"original";

        let result = manager
            .get_embedded_asset_or_custom("missing.txt", original)
            .await?;
        assert_eq!(
            result, original,
            "Should return original content when override file doesn't exist in directory"
        );
        Ok(())
    }
}
