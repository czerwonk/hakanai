// SPDX-License-Identifier: Apache-2.0

import { createButton, generateRandomId } from "../core/dom-utils";
import { copyToClipboard, copyToClipboardByElementId } from "../core/clipboard";
import { QRCodeGenerator } from "../core/qr-generator";
import { I18nKeys } from "../core/i18n";
import type { RestrictionData } from "../core/restriction-data.js";

/**
 * Options for success result display
 */
interface SuccessDisplayOptions {
  separateKeyMode?: boolean;
  container: HTMLElement;
  restrictionData?: RestrictionData;
}

/**
 * Display unified success result with URL, QR code, and security note
 * @param url - The secret URL to display
 * @param options - Configuration options
 */
// Flag to ensure we only add the cleanup listener once
let cleanupListenerAdded = false;

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

  if (options.separateKeyMode) {
    createSeparateUrlDisplay(urlContainer, url);
  } else {
    createCombinedUrlDisplay(urlContainer, url);
  }

  container.appendChild(urlContainer);
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
    "copy-button",
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
    "secondary-button",
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
  qrFullscreenContainer.innerHTML = qrSvg;

  overlay.appendChild(qrFullscreenContainer);

  // Close on click anywhere
  overlay.addEventListener("click", () => {
    overlay.remove();
  });

  // Close on Escape key
  const escapeHandler = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      overlay.remove();
      document.removeEventListener("keydown", escapeHandler);
    }
  };
  document.addEventListener("keydown", escapeHandler);

  document.body.appendChild(overlay);
}

/**
 * Create security note section
 */
function createNoteSection(container: HTMLElement): void {
  const note = document.createElement("p");
  note.className = "secret-note";

  const noteText = window.i18n.t(I18nKeys.Msg.CreateNote);
  const colonIndex = noteText.indexOf(":");

  if (colonIndex > 0) {
    const strong = document.createElement("strong");
    strong.textContent = noteText.substring(0, colonIndex + 1);
    note.appendChild(strong);

    const remaining = document.createTextNode(
      noteText.substring(colonIndex + 1),
    );
    note.appendChild(remaining);
  } else {
    note.textContent = noteText;
  }

  container.appendChild(note);
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
    "secondary-button",
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
    "copy-button",
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
