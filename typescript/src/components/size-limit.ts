// SPDX-License-Identifier: Apache-2.0

import { showElement, hideElement } from "../core/dom-utils";

/**
 * Manages the size limit indicator for anonymous users
 */
export class SizeLimitIndicator {
  private containerElement: HTMLElement;
  private textElement: HTMLElement;
  private limit: number = 0;
  private isVisible: boolean = false;

  constructor() {
    const container = document.getElementById("sizeLimitIndicator");
    const text = document.getElementById("limitText");

    if (!container || !text) {
      throw new Error("Size limit indicator elements not found");
    }

    this.containerElement = container;
    this.textElement = text;
  }

  /**
   * Initialize the size limit indicator with a limit value
   */
  public initialize(limit: number): void {
    this.limit = limit;
    if (limit > 0) {
      this.show();
    } else {
      this.hide();
    }
  }

  /**
   * Update the progress bar based on current content
   */
  public update(content: string): void {
    if (!this.isVisible || this.limit <= 0) {
      return;
    }

    const bytes = new TextEncoder().encode(content).length;
    const percentage = Math.min(100, (bytes / this.limit) * 100);
    this.updateText(percentage);
  }

  private updateText(percentage: number): void {
    this.textElement.textContent = `${Math.round(percentage)}%`;

    this.textElement.classList.remove("warning", "danger");
    if (percentage >= 90) {
      this.textElement.classList.add("danger");
    } else if (percentage >= 70) {
      this.textElement.classList.add("warning");
    }
  }

  /**
   * Show the size limit indicator
   */
  public show(): void {
    if (this.limit <= 0) {
      return;
    }

    showElement(this.containerElement);
    this.isVisible = true;
  }

  /**
   * Hide the size limit indicator
   */
  public hide(): void {
    hideElement(this.containerElement);
    this.isVisible = false;
    this.reset();
  }

  private reset(): void {
    this.textElement.classList.remove("warning", "danger");
    this.textElement.textContent = "";
  }
}
