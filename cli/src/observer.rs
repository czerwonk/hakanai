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
            self.progress_bar.finish_with_message("✓ Complete!");
        }
    }
}
