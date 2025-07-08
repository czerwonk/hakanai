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
/// ## Basic Progress Percentage
///
/// ```
/// use hakanai_lib::observer::DataTransferObserver;
/// use async_trait::async_trait;
///
/// struct PercentageLogger;
///
/// #[async_trait]
/// impl DataTransferObserver for PercentageLogger {
///     async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64) {
///         let percentage = (bytes_transferred as f64 / total_bytes as f64) * 100.0;
///         println!("Progress: {:.1}%", percentage);
///     }
/// }
/// ```
///
/// ## Rate-Limited Updates
///
/// ```
/// use hakanai_lib::observer::DataTransferObserver;
/// use async_trait::async_trait;
/// use std::sync::Mutex;
/// use std::time::{Duration, Instant};
///
/// struct ThrottledObserver {
///     last_update: Mutex<Instant>,
///     min_interval: Duration,
/// }
///
/// impl ThrottledObserver {
///     fn new(min_interval: Duration) -> Self {
///         Self {
///             last_update: Mutex::new(Instant::now()),
///             min_interval,
///         }
///     }
/// }
///
/// #[async_trait]
/// impl DataTransferObserver for ThrottledObserver {
///     async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64) {
///         let mut last = self.last_update.lock().unwrap();
///         let now = Instant::now();
///         
///         // Only update if enough time has passed or transfer is complete
///         if now.duration_since(*last) >= self.min_interval
///            || bytes_transferred == total_bytes {
///             *last = now;
///             let percentage = (bytes_transferred as f64 / total_bytes as f64) * 100.0;
///             println!("Progress: {:.1}% ({}/{})", percentage, bytes_transferred, total_bytes);
///         }
///     }
/// }
/// ```
///
/// ## Bandwidth Monitoring
///
/// ```
/// use hakanai_lib::observer::DataTransferObserver;
/// use async_trait::async_trait;
/// use std::sync::Mutex;
/// use std::time::Instant;
///
/// struct BandwidthMonitor {
///     start_time: Mutex<Option<Instant>>,
///     last_bytes: Mutex<u64>,
/// }
///
/// impl Default for BandwidthMonitor {
///     fn default() -> Self {
///         Self {
///             start_time: Mutex::new(None),
///             last_bytes: Mutex::new(0),
///         }
///     }
/// }
///
/// #[async_trait]
/// impl DataTransferObserver for BandwidthMonitor {
///     async fn on_progress(&self, bytes_transferred: u64, total_bytes: u64) {
///         let mut start_time = self.start_time.lock().unwrap();
///         let start = start_time.get_or_insert_with(Instant::now);
///         
///         let elapsed = start.elapsed().as_secs_f64();
///         if elapsed > 0.0 {
///             let bandwidth_mbps = (bytes_transferred as f64 / elapsed) / (1024.0 * 1024.0);
///             let percentage = (bytes_transferred as f64 / total_bytes as f64) * 100.0;
///             
///             if bytes_transferred == total_bytes {
///                 println!("Transfer complete: {:.1} MB in {:.1}s ({:.2} MB/s)",
///                     total_bytes as f64 / (1024.0 * 1024.0),
///                     elapsed,
///                     bandwidth_mbps
///                 );
///             } else {
///                 println!("Progress: {:.1}% at {:.2} MB/s", percentage, bandwidth_mbps);
///             }
///         }
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
