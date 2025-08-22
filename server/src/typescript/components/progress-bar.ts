// SPDX-License-Identifier: Apache-2.0

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
  private percentageElement: HTMLElement | null = null;
  private sizeElement: HTMLElement | null = null;
  private hasReceivedProgress: boolean = false;
  private showTimeout: number | null = null;
  private pendingText: string = "";
  private isVisible: boolean = false;
  private isHiding: boolean = false;
  private currentBytes: number = 0;
  private totalBytes: number | undefined = 0;

  /**
   * Show the progress bar overlay after a delay to avoid flickering
   * @param text - Text to display
   */
  show(text: string = "Processing..."): void {
    this.showDelayed(text);
  }

  /**
   * Internal method to handle delayed show
   * @param text - Text to display
   * @param delayMs - Delay in milliseconds before showing (default 500ms)
   */
  private showDelayed(
    text: string = "Processing...",
    delayMs: number = 500,
  ): void {
    this.cancelDelayedShow();
    this.pendingText = text;
    this.currentBytes = 0;
    this.totalBytes = 0;
    this.hasReceivedProgress = false;

    // Set a timeout to show the progress bar after the delay
    this.showTimeout = window.setTimeout(() => {
      this.showImmediately(this.pendingText);
      this.showTimeout = null;
    }, delayMs);
  }

  private showImmediately(text: string): void {
    // Don't show if we're in the process of hiding or already visible
    if (this.isHiding || this.isVisible) {
      return;
    }

    this.isVisible = true;
    this.createOverlay(text);
    document.body.appendChild(this.overlay!);

    if (this.hasReceivedProgress) {
      this.updateProgressDisplay();
    }
  }

  private cancelDelayedShow(): void {
    if (this.showTimeout !== null) {
      window.clearTimeout(this.showTimeout);
      this.showTimeout = null;
    }
  }

  /**
   * Hide and remove the progress bar overlay
   */
  hide(): void {
    this.isHiding = true;
    this.cancelDelayedShow();
    this.hideOverlay();
    this.isHiding = false;
  }

  private hideOverlay(): void {
    if (this.overlay && this.overlay.parentNode) {
      this.overlay.parentNode.removeChild(this.overlay);
    }
    this.overlay = null;
    this.progressCircle = null;
    this.percentageElement = null;
    this.sizeElement = null;
    this.isVisible = false;
  }

  updateProgress(current: number, total: number | undefined): void {
    this.currentBytes = current;
    this.totalBytes = total;

    // We've received progress regardless of whether we know the total
    if (!this.hasReceivedProgress) {
      this.hasReceivedProgress = true;
    }

    if (this.isVisible) {
      this.updateProgressDisplay();
    }
  }

  private updateProgressDisplay(): void {
    if (!this.progressCircle) return;

    const isTotalSizeKnown =
      this.totalBytes !== undefined && this.totalBytes > 0;
    const percentage = isTotalSizeKnown
      ? Math.min((this.currentBytes / this.totalBytes!) * 100, 100)
      : 0;

    // On first real progress, stop the pulse animation
    if (
      this.hasReceivedProgress &&
      this.progressCircle.classList.contains("progress-logo-pulsing")
    ) {
      this.progressCircle.classList.remove("progress-logo-pulsing");
      this.progressCircle.classList.add("progress-logo-loading");
    }

    // If we have progress, show it; otherwise keep pulsing
    if (this.hasReceivedProgress && isTotalSizeKnown) {
      // Update opacity based on progress using CSS custom property
      this.progressCircle.style.setProperty(
        "--progress-opacity",
        (percentage / 100).toString(),
      );
    }

    if (this.percentageElement) {
      this.percentageElement.textContent = isTotalSizeKnown
        ? `${Math.round(percentage)}%`
        : "";
    }

    if (this.sizeElement) {
      this.sizeElement.textContent = isTotalSizeKnown
        ? `${formatFileSize(this.currentBytes)} / ${formatFileSize(this.totalBytes!)}`
        : formatFileSize(this.currentBytes);
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

    // Progress percentage (large, standalone)
    this.percentageElement = document.createElement("div");
    this.percentageElement.className = "progress-percentage";
    this.percentageElement.textContent = "";
    content.appendChild(this.percentageElement);

    // Size info (smaller, below percentage)
    this.sizeElement = document.createElement("div");
    this.sizeElement.className = "progress-size";
    this.sizeElement.textContent = "";
    content.appendChild(this.sizeElement);

    this.overlay.appendChild(content);
  }

  private createCircularProgressElement(): HTMLElement {
    const progressContainer = document.createElement("div");
    progressContainer.className = "circular-progress";

    const logoImg = document.createElement("img");
    logoImg.src = "/logo.svg";
    logoImg.className = "progress-logo progress-logo-pulsing";

    this.progressCircle = logoImg;

    progressContainer.appendChild(logoImg);
    return progressContainer;
  }

  /**
   * Implementation of DataTransferObserver interface
   * Called when data transfer progress is made
   */
  async onProgress(
    bytesTransferred: number,
    totalBytes: number | undefined,
  ): Promise<void> {
    // Just update the progress values, display will be handled when/if overlay shows
    this.updateProgress(bytesTransferred, totalBytes);
  }
}
