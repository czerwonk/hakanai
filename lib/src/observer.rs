/// A trait for observing the progress of upload operations.
///
/// Implementors of this trait can receive real-time notifications about upload progress,
/// allowing for features like progress bars, bandwidth monitoring, or logging.
///
/// # Thread Safety
///
/// This trait requires `Send + Sync` to ensure it can be safely used across async tasks
/// and shared between threads.
///
/// # Examples
///
/// ```
/// use hakanai_lib::observer::DataTransferObserver;
/// use async_trait::async_trait;
///
/// struct ProgressLogger;
///
/// #[async_trait]
/// impl DataTransferObserver for ProgressLogger {
///     async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64) {
///         let percentage = (bytes_transferred as f64 / total_bytes as f64) * 100.0;
///         println!("Progress: {:.1}%", percentage);
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait DataTransferObserver: Send + Sync {
    /// Called when data transfer progress is made.
    ///
    /// This method is invoked periodically during the data transfer process.
    ///
    /// # Arguments
    ///
    /// * `bytes_uploaded` - The total number of bytes transferred so far
    /// * `total_bytes` - The total size of the transfer in bytes
    ///
    /// # Notes
    ///
    /// - This method is called asynchronously and should not block for extended periods
    /// - The frequency of calls depends on the chunk size used
    /// - `bytes_uploaded` will always be ≤ `total_bytes`
    /// - The final call will have `bytes_uploaded == total_bytes`
    async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64);
}
