/**
 * Reusable TTL selector component with preset options and custom input
 */

import { formatTTL } from "../core/formatters";
import { hideElement, showElement } from "../core/dom-utils";
import { PreferenceStorage } from "../core/preferences";

export class TTLSelector {
  private selectElement: HTMLSelectElement;
  private customContainer: HTMLElement;
  private customValueInput: HTMLInputElement;
  private customUnitSelect: HTMLSelectElement;
  private currentValue: number = 0;

  constructor(container: HTMLElement) {
    this.selectElement = container.querySelector(
      "#ttl-selector-select",
    ) as HTMLSelectElement;

    this.customContainer = container.querySelector(
      "#ttl-custom",
    ) as HTMLElement;

    this.customValueInput = container.querySelector(
      ".ttl-custom-value",
    ) as HTMLInputElement;

    this.customUnitSelect = container.querySelector(
      ".ttl-custom-unit",
    ) as HTMLSelectElement;

    if (
      !this.selectElement ||
      !this.customContainer ||
      !this.customValueInput ||
      !this.customUnitSelect
    ) {
      throw new Error("TTL selector elements not found");
    }

    // Try to restore last used TTL
    const lastTTL = PreferenceStorage.getLastTTL();
    if (lastTTL !== undefined) {
      this.setValue(lastTTL);
    } else {
      this.currentValue = parseInt(this.selectElement.value);
    }

    this.setupEventListeners();
  }

  private setupEventListeners(): void {
    this.selectElement.addEventListener("change", () => {
      const value = this.selectElement.value;
      if (value === "custom") {
        this.showCustomInput();
      } else {
        this.hideCustomInput();
        this.currentValue = parseInt(value);
        PreferenceStorage.saveLastTTL(this.currentValue);
      }
    });

    const updateCustomValue = () => {
      const value = parseInt(this.customValueInput.value) || 1;
      const unit = parseInt(this.customUnitSelect.value);
      this.currentValue = value * unit;
      PreferenceStorage.saveLastTTL(this.currentValue);
    };
    this.customValueInput.addEventListener("input", updateCustomValue);
    this.customUnitSelect.addEventListener("change", updateCustomValue);
  }

  /**
   * Get the currently selected TTL value in seconds
   */
  getValue(): number {
    return this.currentValue;
  }

  /**
   * Set the TTL value, handling custom values if needed
   */
  setValue(ttl: number): void {
    this.currentValue = ttl;

    if (this.isCustomValue(ttl)) {
      this.selectElement.value = "custom";
      this.showCustomInput();
      this.setCustomInputValue(ttl);
    } else {
      this.selectElement.value = ttl.toString();
      this.hideCustomInput();
    }
  }

  isCustomValue(ttl: number): boolean {
    const options = Array.from(this.selectElement.options);
    const matchingOption = options.find(
      (opt) => opt.value === ttl.toString() && opt.value !== "custom",
    );

    return !matchingOption;
  }

  private showCustomInput(): void {
    showElement(this.customContainer);
    this.customValueInput.focus();
    this.customValueInput.select();
  }

  private hideCustomInput(): void {
    hideElement(this.customContainer);
  }

  private setCustomInputValue(totalSeconds: number): void {
    // Try to find the best unit for display
    if (totalSeconds % 86400 === 0) {
      // Days
      this.customValueInput.value = (totalSeconds / 86400).toString();
      this.customUnitSelect.value = "86400";
    } else if (totalSeconds % 3600 === 0) {
      // Hours
      this.customValueInput.value = (totalSeconds / 3600).toString();
      this.customUnitSelect.value = "3600";
    } else if (totalSeconds % 60 === 0) {
      // Minutes
      this.customValueInput.value = (totalSeconds / 60).toString();
      this.customUnitSelect.value = "60";
    } else {
      // Default to seconds
      this.customValueInput.value = totalSeconds.toString();
      this.customUnitSelect.value = "1";
    }
  }

  /**
   * Enable or disable the selector
   */
  setEnabled(enabled: boolean): void {
    this.selectElement.disabled = !enabled;
    this.customValueInput.disabled = !enabled;
    this.customUnitSelect.disabled = !enabled;
  }

  /**
   * Get formatted display text for current value
   */
  getDisplayText(): string {
    return formatTTL(this.currentValue);
  }
}
