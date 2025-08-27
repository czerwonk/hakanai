// SPDX-License-Identifier: Apache-2.0

import { showElement, hideElement } from "../core/dom-utils";
import { fetchAppConfig } from "../core/app-config";
import { RestrictionData } from "../core/restriction-data.js";
import { I18nKeys } from "../core/i18n";

export interface RestrictionsTabsConfig {
  container: HTMLElement;
}

export class RestrictionsTabs {
  private readonly container: HTMLElement;
  private readonly tabs: Map<string, HTMLElement> = new Map();
  private readonly tabContents: Map<string, HTMLElement> = new Map();
  private currentActiveTab: string = "passphrase";

  constructor(config: RestrictionsTabsConfig) {
    this.container = config.container;
    this.initializeTabs();
    this.setupEventHandlers();
    this.setActiveTab("passphrase"); // Start with passphrase tab active
    this.hideUnsupportedRestrictions();
  }

  private initializeTabs(): void {
    this.container.querySelectorAll(".tab-button").forEach((button) => {
      const tabId = button.getAttribute("data-tab");
      if (tabId) {
        this.tabs.set(tabId, button as HTMLElement);
      }
    });

    this.container.querySelectorAll(".tab-content").forEach((content) => {
      const contentId = content.getAttribute("data-content");
      if (contentId) {
        this.tabContents.set(contentId, content as HTMLElement);
      }
    });
  }

  private setupEventHandlers(): void {
    this.tabs.forEach((button, tabId) => {
      button.addEventListener("click", () => {
        this.setActiveTab(tabId);
      });
    });
  }

  private setActiveTab(tabId: string): void {
    // Remove active class from all tabs and contents
    this.tabs.forEach((button, id) => {
      if (id === tabId) {
        button.classList.add("active");
      } else {
        button.classList.remove("active");
      }
    });

    this.tabContents.forEach((content, id) => {
      if (id === tabId) {
        content.classList.add("active");
        showElement(content);
      } else {
        content.classList.remove("active");
        hideElement(content);
      }
    });

    this.currentActiveTab = tabId;
    this.focusActiveTab();
  }

  /**
   * Get all restriction data from the tabs
   */
  getRestrictions(): RestrictionData | undefined {
    const data: RestrictionData = {};
    let hasRestrictions = false;

    hasRestrictions = this.addIPRestrictions(data) || hasRestrictions;
    hasRestrictions = this.addCountryRestrictions(data) || hasRestrictions;
    hasRestrictions = this.addASNRestrictions(data) || hasRestrictions;
    hasRestrictions = this.addPassphraseRestrictions(data) || hasRestrictions;

    return hasRestrictions ? data : undefined;
  }

  /**
   * Add IP restrictions to the restrictions object
   */
  private addIPRestrictions(data: RestrictionData): boolean {
    const ipInput = this.container.querySelector(
      "#allowedIPs",
    ) as HTMLTextAreaElement;
    if (!ipInput?.value.trim()) {
      return false;
    }

    const ips = ipInput.value
      .split("\n")
      .map((line) => line.trim())
      .filter((line) => line.length > 0);

    if (ips.length > 0) {
      data.allowed_ips = ips;
      return true;
    }

    return false;
  }

  /**
   * Add country restrictions to the restrictions object
   */
  private addCountryRestrictions(data: RestrictionData): boolean {
    const countryInput = this.container.querySelector(
      "#allowedCountries",
    ) as HTMLTextAreaElement;
    if (!countryInput?.value.trim()) {
      return false;
    }

    const countries = countryInput.value
      .split("\n")
      .map((line) => line.trim().toUpperCase())
      .filter((line) => line.length > 0);

    if (countries.length > 0) {
      data.allowed_countries = countries;
      return true;
    }

    return false;
  }

  /**
   * Add ASN restrictions to the restrictions object
   */
  private addASNRestrictions(data: RestrictionData): boolean {
    const asnInput = this.container.querySelector(
      "#allowedASNs",
    ) as HTMLTextAreaElement;
    if (!asnInput?.value.trim()) {
      return false;
    }

    const asns = asnInput.value
      .split("\n")
      .map((line) => {
        const trimmed = line.trim();
        const parsed = parseInt(trimmed, 10);
        return isNaN(parsed) ? null : parsed;
      })
      .filter(
        // ASN range: 1 to 4294967295 (ASN 0 is reserved and not allowed)
        (asn): asn is number => asn !== null && asn >= 1 && asn <= 4294967295,
      );

    if (asns.length > 0) {
      data.allowed_asns = asns;
      return true;
    }

    return false;
  }

  /**
   * Add passphrase restrictions to the restrictions object
   */
  private addPassphraseRestrictions(data: RestrictionData): boolean {
    const passphraseInput = this.container.querySelector(
      "#passphraseRestriction",
    ) as HTMLInputElement;
    const passphrase = passphraseInput?.value.trim();
    if (!passphrase) {
      return false; // empty passphrase means no restriction
    }

    data.passphrase = passphrase;
    return true;
  }

  /**
   * Validates all user inputs across tabs
   */
  public validateUserInput(): boolean {
    return this.validatePassphrase();
  }

  private validatePassphrase(): boolean {
    const passphraseInput = this.container.querySelector(
      "#passphraseRestriction",
    ) as HTMLInputElement;
    const passphrase = passphraseInput?.value.trim();
    if (!passphrase) {
      return true; // empty passphrase is allowed (no restriction)
    }

    if (passphrase.length >= 8) {
      return true; // valid passphrase
    }

    const message = (window as any).i18n.t(I18nKeys.Error.PassphraseTooShort);
    this.showPassphraseError(message);
    return false;
  }

  /**
   * Show passphrase validation error message
   */
  private showPassphraseError(message: string): void {
    const passphraseInput = this.container.querySelector(
      "#passphraseRestriction",
    ) as HTMLInputElement;

    if (!passphraseInput) {
      return;
    }

    passphraseInput.classList.add("error");
    passphraseInput.setCustomValidity(message);
    passphraseInput.reportValidity();

    // Clear error after user starts typing again
    const clearError = () => {
      passphraseInput.classList.remove("error");
      passphraseInput.setCustomValidity("");
      passphraseInput.removeEventListener("input", clearError);
    };
    passphraseInput.addEventListener("input", clearError);
  }

  /**
   * Set enabled/disabled state for all inputs
   */
  setEnabled(enabled: boolean): void {
    const inputs = this.container.querySelectorAll("textarea, button");
    inputs.forEach((input) => {
      (input as HTMLInputElement | HTMLButtonElement).disabled = !enabled;
    });
  }

  /**
   * Clear all input fields
   */
  clear(): void {
    const inputs = this.container.querySelectorAll("textarea");
    inputs.forEach((input) => {
      (input as HTMLTextAreaElement).value = "";
    });
  }

  /**
   * Get the currently active tab
   */
  getActiveTab(): string {
    return this.currentActiveTab;
  }

  /**
   * Check if any restrictions are set
   */
  hasRestrictions(): boolean {
    return this.getRestrictions() !== undefined;
  }

  /**
   * Focus the input field in the currently active tab
   */
  focusActiveTab(): void {
    const activeContent = this.tabContents.get(this.currentActiveTab);
    if (!activeContent) {
      return;
    }

    const inputField = activeContent.querySelector("input, textarea") as
      | HTMLInputElement
      | HTMLTextAreaElement;
    if (inputField) {
      inputField.focus();
    }
  }

  /**
   * Fetch server configuration and configure tab visibility
   */
  private async hideUnsupportedRestrictions(): Promise<void> {
    const config = await fetchAppConfig();

    const showCountry = config?.features?.restrictions?.country ?? false;
    this.setTabVisible("country", showCountry);

    const showASN = config?.features?.restrictions?.asn ?? false;
    this.setTabVisible("asn", showASN);
  }

  /**
   * Configure tab visibility
   */
  private setTabVisible(name: string, enabled: boolean): void {
    const tab = this.tabs.get(name);
    const content = this.tabContents.get(name);

    if (tab && content) {
      if (enabled) {
        showElement(tab);
        showElement(content);
      } else {
        hideElement(tab);
        hideElement(content);
      }
    }
  }
}
