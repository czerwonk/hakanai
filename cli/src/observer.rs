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
            ProgressStyle::with_template(
                "{msg} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )?
            .progress_chars("#>-"),
        );
        progress_bar.set_message(label.to_string());

        Ok(Self { progress_bar })
    }
}

#[async_trait]
impl DataTransferObserver for ProgressObserver {
    async fn on_progress(&self, bytes_uploaded: u64, total_bytes: u64) {
        if self.progress_bar.length() == Some(0) {
            self.progress_bar.set_length(total_bytes);
        }

        self.progress_bar.set_position(bytes_uploaded);

        if bytes_uploaded >= total_bytes {
            self.progress_bar.finish_with_message("✓ Complete!");
        }
    }
}
