// SPDX-License-Identifier: Apache-2.0

import { createButton, generateRandomId } from "../core/dom-utils";
import { copyToClipboard, copyToClipboardByElementId } from "../core/clipboard";
import { QRCodeGenerator } from "../core/qr-generator";
import { I18nKeys } from "../core/i18n";
import type { RestrictionData } from "../core/restriction-data.js";
import { PreferenceStorage } from "../core/preferences.js";

/**
 * Options for success result display
 */
interface SuccessDisplayOptions {
  container: HTMLElement;
  restrictionData?: RestrictionData;
  initialSeparateKeyModeState?: boolean;
}

export function displaySuccessResult(
  url: string,
  options: SuccessDisplayOptions,
): void {
  const { container } = options;

  container.className = "result success";
  container.innerHTML = "";

  createSuccessHeader(container);

  createUrlSection(container, url, options);

  createNoteSection(container);

  if (options.restrictionData) {
    createRestrictionsSection(container, options.restrictionData);
  }
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
function createUrlSection(
  container: HTMLElement,
  url: string,
  options: SuccessDisplayOptions,
): void {
  const urlContainer = document.createElement("div");
  urlContainer.className = "url-container";

  // Create URL display container
  const urlDisplayContainer = document.createElement("div");
  urlDisplayContainer.id = "urlDisplayContainer";

  // Initial display with saved preference
  const savedPreference =
    PreferenceStorage.getSeparateKeyMode() ??
    options?.initialSeparateKeyModeState ??
    false;
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
  note.appendChild(
    document.createTextNode("⚠️ " + window.i18n.t(I18nKeys.Msg.CreateNote)),
  );
  container.appendChild(note);
}

/**
 * Create access restrictions section
 */
function createRestrictionsSection(
  container: HTMLElement,
  restrictionData: RestrictionData,
): void {
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
    ipItem.appendChild(
      document.createTextNode(restrictionData.allowed_ips.join(", ")),
    );
    restrictionsList.appendChild(ipItem);
  }

  if (
    restrictionData.allowed_countries &&
    restrictionData.allowed_countries.length > 0
  ) {
    const countryItem = document.createElement("li");
    const strong = document.createElement("strong");
    strong.textContent = "Countries: ";
    countryItem.appendChild(strong);
    countryItem.appendChild(
      document.createTextNode(restrictionData.allowed_countries.join(", ")),
    );
    restrictionsList.appendChild(countryItem);
  }

  if (restrictionData.allowed_asns && restrictionData.allowed_asns.length > 0) {
    const asnItem = document.createElement("li");
    const strong = document.createElement("strong");
    strong.textContent = "Networks (ASN): ";
    asnItem.appendChild(strong);
    asnItem.appendChild(
      document.createTextNode(restrictionData.allowed_asns.join(", ")),
    );
    restrictionsList.appendChild(asnItem);
  }

  if (restrictionData.passphrase && restrictionData.passphrase.trim()) {
    restrictionsList.appendChild(
      createPassphraseRestrictionItem(restrictionData.passphrase.trim()),
    );
  }

  restrictionsDiv.appendChild(restrictionsList);
  container.appendChild(restrictionsDiv);
}

/**
 * Create a labeled input field with copy button
 */
function createLabeledInputWithCopy(
  container: HTMLElement,
  labelKey: string,
  inputId: string,
  value: string,
  ariaLabel: string,
): void {
  const label = document.createElement("label");
  label.textContent = window.i18n.t(labelKey);
  label.setAttribute("for", inputId);
  container.appendChild(label);

  const inputContainer = document.createElement("div");
  inputContainer.className = "input-group";

  const input = document.createElement("input");
  input.type = "text";
  input.id = inputId;
  input.value = value;
  input.readOnly = true;
  input.className = "url-input";
  inputContainer.appendChild(input);

  const copyButton = createButton(
    "btn copy-btn",
    window.i18n.t(I18nKeys.Button.Copy),
    ariaLabel,
    () => copyToClipboardByElementId(inputId, copyButton as HTMLButtonElement),
  );
  inputContainer.appendChild(copyButton);

  const qrButton = createQrButton(value);
  inputContainer.appendChild(qrButton);

  container.appendChild(inputContainer);
}

/**
 * Create a QR code button
 */
function createQrButton(url: string): HTMLButtonElement {
  return createButton(
    "btn secondary-btn",
    "▦ QR",
    window.i18n.t(I18nKeys.Button.ShowQrCode),
    async () => {
      try {
        await QRCodeGenerator.ensureWasmLoaded();
        const qrSvg = QRCodeGenerator.generateQRCode(url);
        if (qrSvg) {
          showQRFullscreen(qrSvg);
        }
      } catch (error) {
        console.error("Failed to generate QR code:", error);
      }
    },
  );
}

/**
 * Update URL display based on separate key mode
 */
function updateUrlDisplay(
  container: HTMLElement,
  url: string,
  separateMode: boolean,
): void {
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
function createSeparateKeyCheckbox(
  onChangeCallback: (checked: boolean) => void,
  initialState: boolean,
): HTMLElement {
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
  createLabeledInputWithCopy(
    container,
    I18nKeys.Label.Url,
    urlId,
    url,
    "Copy secret URL to clipboard",
  );
}

/**
 * Create separate URL and key display (enhanced security mode)
 */
function createSeparateUrlDisplay(
  container: HTMLElement,
  fullUrl: string,
): void {
  const [url, key] = fullUrl.split("#");
  const baseId = generateRandomId();

  // URL section (with QR button for the full URL)
  createLabeledInputWithCopy(
    container,
    I18nKeys.Label.Url,
    baseId,
    url,
    "Copy secret URL to clipboard",
  );

  // Key section
  createLabeledInputWithCopy(
    container,
    I18nKeys.Label.Key,
    baseId + "-key",
    key,
    "Copy decryption key to clipboard",
  );
}

/**
 * Show QR code in fullscreen modal
 */
function showQRFullscreen(qrSvg: string): void {
  // Create fullscreen overlay
  const overlay = document.createElement("div");
  overlay.className = "qr-fullscreen-overlay";

  // Create container for the QR code
  const qrFullscreenContainer = document.createElement("div");
  qrFullscreenContainer.className = "qr-fullscreen-container";

  // Close handler (declare early for use in close button)
  const closeModal = () => {
    overlay.remove();
    document.removeEventListener("keydown", escapeHandler);
  };

  // Add close button
  const closeButton = createButton(
    "btn qr-close-btn",
    "✕",
    window.i18n.t(I18nKeys.Button.Close) || "Close",
    (e: Event) => {
      e.stopPropagation(); // Prevent closing the modal
      closeModal();
    },
  );
  qrFullscreenContainer.appendChild(closeButton);

  // Add QR code
  const qrContainer = document.createElement("div");
  qrContainer.innerHTML = qrSvg;
  qrFullscreenContainer.appendChild(qrContainer);

  // Add download button
  const downloadButton = createButton(
    "btn qr-download-btn",
    window.i18n.t(I18nKeys.Button.Download) || "Download",
    window.i18n.t(I18nKeys.Aria.DownloadQRCode),
    (e: Event) => {
      e.stopPropagation(); // Prevent closing the modal
      downloadQRCode(qrSvg);
    },
  );
  qrFullscreenContainer.appendChild(downloadButton);

  overlay.appendChild(qrFullscreenContainer);

  // Close on click
  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) {
      closeModal();
    }
  });

  // Close on Escape key
  const escapeHandler = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      closeModal();
    }
  };
  document.addEventListener("keydown", escapeHandler);

  document.body.appendChild(overlay);
}

/**
 * Download QR code as SVG file
 */
function downloadQRCode(svgContent: string): void {
  // Create a blob from the SVG content
  const blob = new Blob([svgContent], { type: "image/svg+xml" });
  const url = URL.createObjectURL(blob);

  // Create a temporary link element and trigger download
  const link = document.createElement("a");
  link.href = url;
  link.download = `hakanai-qr-${Date.now()}.svg`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  // Clean up the blob URL
  setTimeout(() => URL.revokeObjectURL(url), 100);
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
  const showButton = createButton(
    "btn secondary-btn",
    "👁",
    "Show/hide passphrase",
    () => {
      isVisible = !isVisible;
      passphraseText.textContent = isVisible ? passphrase : "••••••";
      showButton.textContent = isVisible ? "🙈" : "👁";
    },
  );
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
