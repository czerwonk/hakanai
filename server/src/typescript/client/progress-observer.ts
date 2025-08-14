// SPDX-License-Identifier: MIT

/*
 * Progress Observer Interface for Hakanai TypeScript Client
 *
 * This interface matches the DataTransferObserver trait from the Rust library,
 * providing progress tracking capabilities for upload and download operations.
 */

/**
 * Interface for observing the progress of data transfer operations.
 *
 * Implementors of this interface can receive real-time notifications about
 * transfer progress, allowing for features like progress bars, bandwidth
 * monitoring, or logging.
 *
 * @example
 * ```typescript
 * class ProgressLogger implements DataTransferObserver {
 *   async onProgress(bytesTransferred: number, totalBytes: number): Promise<void> {
 *     const percentage = (bytesTransferred / totalBytes) * 100;
 *     console.log(`Progress: ${percentage.toFixed(1)}%`);
 *   }
 * }
 * ```
 */
export interface DataTransferObserver {
  /**
   * Called when data transfer progress is made.
   *
   * This method is invoked periodically during the data transfer process.
   *
   * @param bytesTransferred - The total number of bytes transferred so far
   * @param totalBytes - The total size of the transfer in bytes
   *
   * @remarks
   * - This method is called asynchronously and should not block for extended periods
   * - The frequency of calls depends on the chunk size and browser implementation
   * - `bytesTransferred` will always be ≤ `totalBytes`
   * - The final call will have `bytesTransferred === totalBytes`
   */
  onProgress(bytesTransferred: number, totalBytes: number): Promise<void>;
}
