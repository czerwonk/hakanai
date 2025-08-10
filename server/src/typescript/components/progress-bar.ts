/**
 * Progress Bar Overlay Component
 *
 * Creates a full-screen overlay with progress bar or spinner, independent of existing UI elements
 */

export class ProgressBar {
  private overlay: HTMLElement | null = null;
  private progressFill: HTMLElement | null = null;
  private textElement: HTMLElement | null = null;
  private mode: 'progress' | 'spinner' = 'progress';

  /**
   * Show the progress bar overlay
   * @param text - Text to display
   * @param mode - 'progress' for progress bar, 'spinner' for indeterminate spinner
   */
  show(text: string = "Processing...", mode: 'progress' | 'spinner' = 'progress'): void {
    if (this.overlay) {
      this.hide(); // Remove any existing overlay
    }

    this.mode = mode;
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
    this.progressFill = null;
    this.textElement = null;
  }

  /**
   * Update progress (0-100)
   * Note: Only works in 'progress' mode, ignored in 'spinner' mode
   */
  updateProgress(current: number, total: number): void {
    if (!this.progressFill || this.mode === 'spinner') return;

    const percentage = total > 0 ? Math.min((current / total) * 100, 100) : 0;
    this.progressFill.style.width = `${percentage}%`;
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

    this.textElement = document.createElement("div");
    this.textElement.className = "progress-text";
    this.textElement.textContent = text;
    content.appendChild(this.textElement);

    if (this.mode === 'spinner') {
      content.appendChild(this.createSpinnerElement());
    } else {
      content.appendChild(this.createProgressBarElement());
    }
    
    this.overlay.appendChild(content);
  }

  private createSpinnerElement(): HTMLElement {
    const spinnerContainer = document.createElement("div");
    spinnerContainer.className = "spinner-container";
    
    const spinner = document.createElement("div");
    spinner.className = "spinner";
    spinnerContainer.appendChild(spinner);
    
    return spinnerContainer;
  }

  private createProgressBarElement(): HTMLElement {
    const progressContainer = document.createElement("div");
    progressContainer.className = "progress-bar";

    this.progressFill = document.createElement("div");
    this.progressFill.className = "progress-fill";
    progressContainer.appendChild(this.progressFill);

    return progressContainer;
  }
}
