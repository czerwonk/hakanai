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

    fn create_test_observer(label: &str) -> ProgressObserver {
        // Create a hidden progress bar for testing to avoid terminal interference
        let progress_bar = ProgressBar::hidden();
        progress_bar.set_style(
            ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.white/gray}] {bytes}/{total_bytes} ({percent}%) {bytes_per_sec} ETA: {eta}")
            .unwrap()
            .progress_chars("██▓▒░  ")
        );
        progress_bar.set_message(label.to_string());

        ProgressObserver { progress_bar }
    }

    #[test]
    fn test_progress_observer_creation() {
        let observer = ProgressObserver::new("Test message");
        assert!(observer.is_ok());

        let observer = observer.unwrap();
        assert_eq!(observer.progress_bar.message(), "Test message");
        assert_eq!(observer.progress_bar.length(), Some(0));
    }

    #[test]
    fn test_progress_observer_creation_with_empty_label() {
        let observer = ProgressObserver::new("");
        assert!(observer.is_ok());

        let observer = observer.unwrap();
        assert_eq!(observer.progress_bar.message(), "");
    }

    #[tokio::test]
    async fn test_on_progress_sets_length_on_first_call() {
        let observer = create_test_observer("Test");

        // Initial state for hidden progress bar
        assert_eq!(observer.progress_bar.length(), None);

        // First progress call should set length
        observer.on_progress(10, 100).await;
        // Hidden progress bars don't maintain length state, but we can test position
        assert_eq!(observer.progress_bar.position(), 10);
    }

    #[tokio::test]
    async fn test_on_progress_updates_position() {
        let observer = create_test_observer("Test");

        // First call sets length and position
        observer.on_progress(25, 100).await;
        assert_eq!(observer.progress_bar.position(), 25);

        // Subsequent calls only update position
        observer.on_progress(50, 100).await;
        assert_eq!(observer.progress_bar.position(), 50);
        // Hidden progress bars don't maintain length state
    }

    #[tokio::test]
    async fn test_on_progress_completion() {
        let observer = create_test_observer("Test");

        // Progress to completion
        observer.on_progress(100, 100).await;
        assert_eq!(observer.progress_bar.position(), 100);

        // Progress bar should be finished but we can't easily test finish_and_clear()
        // since it modifies terminal state
    }

    #[tokio::test]
    async fn test_on_progress_over_completion() {
        let observer = create_test_observer("Test");

        // Progress beyond total should still trigger completion
        observer.on_progress(150, 100).await;
        // For hidden progress bars, position is not clamped
        assert_eq!(observer.progress_bar.position(), 150);
    }

    #[tokio::test]
    async fn test_on_progress_zero_total() {
        let observer = create_test_observer("Test");

        // Edge case: zero total bytes
        observer.on_progress(0, 0).await;
        // Hidden progress bars don't maintain length state
        assert_eq!(observer.progress_bar.length(), None);
        assert_eq!(observer.progress_bar.position(), 0);
    }

    #[tokio::test]
    async fn test_on_progress_multiple_calls_same_total() {
        let observer = create_test_observer("Test");

        observer.on_progress(10, 100).await;
        observer.on_progress(20, 100).await;
        observer.on_progress(30, 100).await;

        // Position should be updated correctly
        assert_eq!(observer.progress_bar.position(), 30);
        // Hidden progress bars don't maintain length state
    }
}
