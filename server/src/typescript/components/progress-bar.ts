/**
 * Progress Bar Overlay Component
 *
 * Creates a full-screen overlay with progress bar or spinner, independent of existing UI elements
 */

import { type DataTransferObserver } from "../client/progress-observer";
import { formatFileSize } from "../core/formatters";

export class ProgressBar implements DataTransferObserver {
  private overlay: HTMLElement | null = null;
  private progressCircle: HTMLImageElement | null = null;
  private textElement: HTMLElement | null = null;
  private lastUpdate: number = 0;
  private lastBytes: number = 0;
  private hasReceivedProgress: boolean = false;

  /**
   * Show the progress bar overlay
   * @param text - Text to display
   */
  show(text: string = "Processing..."): void {
    if (this.overlay) {
      this.hide(); // Remove any existing overlay
    }

    this.lastUpdate = Date.now();
    this.lastBytes = 0;
    this.hasReceivedProgress = false;
    this.createOverlay(text);
    document.body.appendChild(this.overlay!);
  }

  /**
   * Hide and remove the progress bar overlay
   */
  hide(): void {
    if (this.overlay && this.overlay.parentNode) {
      this.overlay.parentNode.removeChild(this.overlay);
    }
    this.overlay = null;
    this.progressCircle = null;
    this.textElement = null;
  }

  /**
   * Update progress (0-100) using simple opacity fade-in
   */
  updateProgress(current: number, total: number): void {
    if (!this.progressCircle) return;

    const percentage = total > 0 ? Math.min((current / total) * 100, 100) : 0;

    // On first real progress, stop the pulse animation
    if (!this.hasReceivedProgress && percentage > 0) {
      this.hasReceivedProgress = true;
      this.progressCircle.style.animation = "none";
    }

    // If we have progress, show it; otherwise keep pulsing
    if (this.hasReceivedProgress) {
      // Fade in from 0% to 100% opacity based on progress
      const opacity = percentage / 100;
      this.progressCircle.style.opacity = opacity.toString();
    }
  }

  /**
   * Set the progress text
   */
  setText(text: string): void {
    if (this.textElement) {
      this.textElement.textContent = text;
    }
  }

  private createOverlay(text: string): void {
    this.overlay = document.createElement("div");
    this.overlay.className = "progress-overlay";

    const content = document.createElement("div");
    content.className = "progress-content";

    // Title at the top
    const titleElement = document.createElement("div");
    titleElement.className = "progress-title";
    titleElement.textContent = text;
    content.appendChild(titleElement);

    // Logo in the middle
    content.appendChild(this.createCircularProgressElement());

    // Status text at the bottom (smaller)
    this.textElement = document.createElement("div");
    this.textElement.className = "progress-status";
    this.textElement.textContent = "";
    content.appendChild(this.textElement);

    this.overlay.appendChild(content);
  }

  private createCircularProgressElement(): HTMLElement {
    const progressContainer = document.createElement("div");
    progressContainer.className = "circular-progress";

    // Use the cached icon.svg directly
    const iconImg = document.createElement("img");
    iconImg.src = "/icon.svg";
    iconImg.alt = "Loading";
    iconImg.style.width = "100px";
    iconImg.style.height = "100px";
    iconImg.style.transition = "opacity 0.3s ease";

    // Start with pulsing animation until we get real progress
    iconImg.style.opacity = "0.3";
    iconImg.style.animation = "pulse 1.5s ease-in-out infinite";

    // Store reference for opacity updates
    this.progressCircle = iconImg;

    progressContainer.appendChild(iconImg);
    return progressContainer;
  }

  /**
   * Implementation of DataTransferObserver interface
   * Called when data transfer progress is made
   */
  async onProgress(
    bytesTransferred: number,
    totalBytes: number,
  ): Promise<void> {
    this.updateProgress(bytesTransferred, totalBytes);

    const percentage =
      totalBytes > 0 ? Math.round((bytesTransferred / totalBytes) * 100) : 0;

    // Status text shows just percentage and size info (without the title)
    let statusText = `${percentage}%`;
    if (totalBytes > 0) {
      statusText += ` • ${formatFileSize(bytesTransferred)} / ${formatFileSize(totalBytes)}`;
    }

    if (bytesTransferred > this.lastBytes) {
      const now = Date.now();
      const timeDiff = (now - this.lastUpdate) / 1000; // Convert to seconds

      if (timeDiff > 0.1) {
        // Only update speed every 100ms to avoid jitter
        const bytesDiff = bytesTransferred - this.lastBytes;
        const speed = bytesDiff / timeDiff;

        if (speed > 0) {
          statusText += ` • ${formatFileSize(speed)}/s`;
        }

        this.lastUpdate = now;
        this.lastBytes = bytesTransferred;
      }
    }

    this.setText(statusText);
  }
}
