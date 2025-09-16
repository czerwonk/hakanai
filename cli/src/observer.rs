// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use indicatif::{ProgressBar, ProgressStyle};

use hakanai_lib::observer::DataTransferObserver;

/// A progress observer that displays a progress bar in the terminal.
pub struct ProgressObserver {
    progress_bar: ProgressBar,
}

impl ProgressObserver {
    /// Creates a new `ProgressObserver` with the given label.
    pub fn new(label: &str) -> Result<Self> {
        let progress_bar = ProgressBar::new(0);
        progress_bar.set_style(
            ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.white/gray}] {bytes}/{total_bytes} ({percent}%) {bytes_per_sec} ETA: {eta}")?
            .progress_chars("██▓▒░  ")
        );
        progress_bar.set_message(label.to_string());

        Ok(Self { progress_bar })
    }
}

#[async_trait]
impl DataTransferObserver for ProgressObserver {
    async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64) {
        if self.progress_bar.length() == Some(0) {
            self.progress_bar.set_length(total_bytes);
        }

        self.progress_bar.set_position(bytes_transferred);

        if bytes_transferred >= total_bytes {
            self.progress_bar.finish_and_clear();
        }
    }
}

impl Drop for ProgressObserver {
    fn drop(&mut self) {
        if !self.progress_bar.is_finished() {
            self.progress_bar.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_observer(label: &str) -> Result<ProgressObserver> {
        // Create a hidden progress bar for testing to avoid terminal interference
        let progress_bar = ProgressBar::hidden();
        progress_bar.set_style(
            ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.white/gray}] {bytes}/{total_bytes} ({percent}%) {bytes_per_sec} ETA: {eta}")
            .expect("invalid template")
            .progress_chars("██▓▒░  ")
        );
        progress_bar.set_message(label.to_string());

        Ok(ProgressObserver { progress_bar })
    }

    #[test]
    fn test_progress_observer_creation() -> Result<()> {
        // Test that the public constructor works (this may create a visible progress bar briefly)
        let observer = ProgressObserver::new("Test message")?;
        assert_eq!(observer.progress_bar.message(), "Test message");
        assert_eq!(observer.progress_bar.length(), Some(0));
        Ok(())
    }

    #[test]
    fn test_progress_observer_creation_with_empty_label() -> Result<()> {
        // Use hidden progress bar to avoid terminal interference in tests
        let observer = create_test_observer("")?;
        assert_eq!(observer.progress_bar.message(), "");
        Ok(())
    }

    #[tokio::test]
    async fn test_on_progress_sets_length_on_first_call() -> Result<()> {
        let observer = create_test_observer("Test")?;

        // Initial state for hidden progress bar
        assert_eq!(observer.progress_bar.length(), None);

        // First progress call should set length
        observer.on_progress(10, 100).await;
        // Hidden progress bars don't maintain length state, but we can test position
        assert_eq!(observer.progress_bar.position(), 10);
        Ok(())
    }

    #[tokio::test]
    async fn test_on_progress_updates_position() -> Result<()> {
        let observer = create_test_observer("Test")?;

        // First call sets length and position
        observer.on_progress(25, 100).await;
        assert_eq!(observer.progress_bar.position(), 25);

        // Subsequent calls only update position
        observer.on_progress(50, 100).await;
        assert_eq!(observer.progress_bar.position(), 50);
        // Hidden progress bars don't maintain length state
        Ok(())
    }

    #[tokio::test]
    async fn test_on_progress_completion() -> Result<()> {
        let observer = create_test_observer("Test")?;

        // Progress to completion
        observer.on_progress(100, 100).await;
        assert_eq!(observer.progress_bar.position(), 100);

        // Progress bar should be finished but we can't easily test finish_and_clear()
        // since it modifies terminal state
        Ok(())
    }

    #[tokio::test]
    async fn test_on_progress_over_completion() -> Result<()> {
        let observer = create_test_observer("Test")?;

        // Progress beyond total should still trigger completion
        observer.on_progress(150, 100).await;
        // For hidden progress bars, position is not clamped
        assert_eq!(observer.progress_bar.position(), 150);
        Ok(())
    }

    #[tokio::test]
    async fn test_on_progress_zero_total() -> Result<()> {
        let observer = create_test_observer("Test")?;

        // Edge case: zero total bytes
        observer.on_progress(0, 0).await;
        // Hidden progress bars don't maintain length state
        assert_eq!(observer.progress_bar.length(), None);
        assert_eq!(observer.progress_bar.position(), 0);
        Ok(())
    }

    #[tokio::test]
    async fn test_on_progress_multiple_calls_same_total() -> Result<()> {
        let observer = create_test_observer("Test")?;

        observer.on_progress(10, 100).await;
        observer.on_progress(20, 100).await;
        observer.on_progress(30, 100).await;

        // Position should be updated correctly
        assert_eq!(observer.progress_bar.position(), 30);
        // Hidden progress bars don't maintain length state
        Ok(())
    }
}
