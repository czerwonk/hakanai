// SPDX-License-Identifier: Apache-2.0

import { createButton, generateRandomId, expandView } from "../core/dom-utils";
import { copyToClipboard } from "../core/clipboard";
import { I18nKeys } from "../core/i18n";
import type { RestrictionData } from "../core/restriction-data";
import { PreferenceStorage } from "../core/preferences";
import { createLabeledInputWithCopy } from "../core/result-utils";

/**
 * Options for success result display
 */
interface SuccessDisplayOptions {
  container: HTMLElement;
  restrictionData?: RestrictionData;
  initialSeparateKeyModeState?: boolean;
}

export function displaySuccessResult(url: string, options: SuccessDisplayOptions): void {
  const { container } = options;

  container.className = "result success";
  container.innerHTML = "";

  createSuccessHeader(container);

  createUrlSection(container, url, options);

  createNoteSection(container);

  if (options.restrictionData) {
    createRestrictionsSection(container, options.restrictionData);
  }

  expandView();
}

/**
 * Create success header with title and instructions
 */
function createSuccessHeader(container: HTMLElement): void {
  const title = document.createElement("h3");
  title.textContent = window.i18n.t(I18nKeys.Msg.SuccessTitle);
  container.appendChild(title);

  const instructions = document.createElement("p");
  instructions.className = "share-instructions";
  instructions.textContent = window.i18n.t(I18nKeys.Msg.ShareInstructions);
  container.appendChild(instructions);
}

/**
 * Create URL display section with copy functionality
 */
function createUrlSection(container: HTMLElement, url: string, options: SuccessDisplayOptions): void {
  const urlContainer = document.createElement("div");
  urlContainer.className = "url-container";

  // Create URL display container
  const urlDisplayContainer = document.createElement("div");
  urlDisplayContainer.id = "urlDisplayContainer";

  // Initial display with saved preference
  const savedPreference = PreferenceStorage.getSeparateKeyMode() ?? options?.initialSeparateKeyModeState ?? false;
  updateUrlDisplay(urlDisplayContainer, url, savedPreference);

  urlContainer.appendChild(urlDisplayContainer);
  container.appendChild(urlContainer);

  // Add separate key checkbox below the main URL display
  const checkboxElement = createSeparateKeyCheckbox(
    (checked) => updateUrlDisplay(urlDisplayContainer, url, checked),
    savedPreference,
  );
  container.appendChild(checkboxElement);
}

/**
 * Create security note section
 */
function createNoteSection(container: HTMLElement): void {
  const note = document.createElement("p");
  note.className = "secret-note";
  note.appendChild(document.createTextNode("⚠️ " + window.i18n.t(I18nKeys.Msg.CreateNote)));
  container.appendChild(note);
}

/**
 * Create access restrictions section
 */
function createRestrictionsSection(container: HTMLElement, restrictionData: RestrictionData): void {
  const restrictionsDiv = document.createElement("div");
  restrictionsDiv.className = "restrictions-info";

  const title = document.createElement("h4");
  title.textContent = window.i18n.t(I18nKeys.Restrictions.Applied);
  restrictionsDiv.appendChild(title);

  const restrictionsList = document.createElement("ul");
  restrictionsList.className = "restrictions-list";

  if (restrictionData.allowed_ips && restrictionData.allowed_ips.length > 0) {
    const ipItem = document.createElement("li");
    const strong = document.createElement("strong");
    strong.textContent = "IP Addresses: ";
    ipItem.appendChild(strong);
    ipItem.appendChild(document.createTextNode(restrictionData.allowed_ips.join(", ")));
    restrictionsList.appendChild(ipItem);
  }

  if (restrictionData.allowed_countries && restrictionData.allowed_countries.length > 0) {
    const countryItem = document.createElement("li");
    const strong = document.createElement("strong");
    strong.textContent = "Countries: ";
    countryItem.appendChild(strong);
    countryItem.appendChild(document.createTextNode(restrictionData.allowed_countries.join(", ")));
    restrictionsList.appendChild(countryItem);
  }

  if (restrictionData.allowed_asns && restrictionData.allowed_asns.length > 0) {
    const asnItem = document.createElement("li");
    const strong = document.createElement("strong");
    strong.textContent = "Networks (ASN): ";
    asnItem.appendChild(strong);
    asnItem.appendChild(document.createTextNode(restrictionData.allowed_asns.join(", ")));
    restrictionsList.appendChild(asnItem);
  }

  if (restrictionData.passphrase && restrictionData.passphrase.trim()) {
    restrictionsList.appendChild(createPassphraseRestrictionItem(restrictionData.passphrase.trim()));
  }

  restrictionsDiv.appendChild(restrictionsList);
  container.appendChild(restrictionsDiv);
}

/**
 * Update URL display based on separate key mode
 */
function updateUrlDisplay(container: HTMLElement, url: string, separateMode: boolean): void {
  container.innerHTML = "";
  if (separateMode) {
    createSeparateUrlDisplay(container, url);
  } else {
    createCombinedUrlDisplay(container, url);
  }
}

/**
 * Create checkbox for separate key mode
 */
function createSeparateKeyCheckbox(onChangeCallback: (checked: boolean) => void, initialState: boolean): HTMLElement {
  const inputGroup = document.createElement("div");
  inputGroup.className = "input-group";

  // Create checkbox with label wrapper
  const label = document.createElement("label");
  label.className = "checkbox-label";

  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.id = "separateKeyCheckbox";
  checkbox.checked = initialState;

  const labelText = document.createElement("span");
  labelText.textContent = window.i18n.t(I18nKeys.Label.SeparateKey);

  label.appendChild(checkbox);
  label.appendChild(labelText);
  inputGroup.appendChild(label);

  // Add helper text
  const helperText = document.createElement("span");
  helperText.className = "input-helper";
  helperText.textContent = window.i18n.t(I18nKeys.Helper.SeparateKey);
  inputGroup.appendChild(helperText);

  checkbox.addEventListener("change", () => {
    PreferenceStorage.saveSeparateKeyMode(checkbox.checked);
    onChangeCallback(checkbox.checked);
  });

  return inputGroup;
}

/**
 * Create combined URL display (traditional mode)
 */
function createCombinedUrlDisplay(container: HTMLElement, url: string): void {
  const urlId = generateRandomId();
  createLabeledInputWithCopy(container, I18nKeys.Label.Url, urlId, url, "Copy secret URL to clipboard");
}

/**
 * Create separate URL and key display (enhanced security mode)
 */
function createSeparateUrlDisplay(container: HTMLElement, fullUrl: string): void {
  const [url, key] = fullUrl.split("#");
  const baseId = generateRandomId();

  // URL section (with QR button for the full URL)
  createLabeledInputWithCopy(container, I18nKeys.Label.Url, baseId, url, "Copy secret URL to clipboard");

  // Key section
  createLabeledInputWithCopy(container, I18nKeys.Label.Key, baseId + "-key", key, "Copy decryption key to clipboard");
}

/**
 * Create passphrase restriction item with show/hide and copy functionality
 */
function createPassphraseRestrictionItem(passphrase: string): HTMLElement {
  const passphraseItem = document.createElement("li");
  const strong = document.createElement("strong");
  strong.textContent = "Passphrase: ";
  passphraseItem.appendChild(strong);

  const passphraseContainer = document.createElement("span");
  passphraseContainer.className = "passphrase-container";

  const passphraseText = document.createElement("span");
  passphraseText.className = "passphrase-text";
  passphraseText.textContent = "••••••";
  passphraseText.setAttribute("data-passphrase", passphrase);
  passphraseContainer.appendChild(passphraseText);

  let isVisible = false;
  const showButton = createButton("btn secondary-btn", "👁", "Show/hide passphrase", () => {
    isVisible = !isVisible;
    passphraseText.textContent = isVisible ? passphrase : "••••••";
    showButton.textContent = isVisible ? "🙈" : "👁";
  });
  passphraseContainer.appendChild(showButton);

  const copyButton = createButton(
    "btn copy-btn",
    window.i18n.t(I18nKeys.Button.Copy),
    "Copy passphrase to clipboard",
    () => {
      copyToClipboard(passphrase, copyButton as HTMLButtonElement);
    },
  );
  passphraseContainer.appendChild(copyButton);

  passphraseItem.appendChild(passphraseContainer);
  return passphraseItem;
}
