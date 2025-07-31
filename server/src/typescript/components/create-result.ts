import { createButton, generateRandomId, hideElement } from "../core/dom-utils";
import { copyToClipboardByElementId } from "../core/clipboard";
import { QRCodeGenerator } from "../core/qr-generator";
import { I18nKeys } from "../core/i18n";

/**
 * Options for success result display
 */
interface SuccessDisplayOptions {
  separateKeyMode?: boolean;
  container: HTMLElement;
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

  createUrlSection(container, url, options.separateKeyMode);

  createQRCodeSection(container, url);

  createNoteSection(container);

  ensureQRCodeGeneratorCleanup();
}

function ensureQRCodeGeneratorCleanup() {
  if (cleanupListenerAdded) {
    return;
  }

  window.addEventListener("beforeunload", () => {
    QRCodeGenerator.cleanup();
  });
  cleanupListenerAdded = true;
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
  separateKeyMode?: boolean,
): void {
  const urlContainer = document.createElement("div");
  urlContainer.className = "url-container";

  if (separateKeyMode) {
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

  container.appendChild(inputContainer);
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

  // URL section
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
 * Create QR code section (with graceful degradation)
 */
async function createQRCodeSection(
  container: HTMLElement,
  url: string,
): Promise<void> {
  try {
    await QRCodeGenerator.ensureWasmLoaded();
    const qrSvg = QRCodeGenerator.generateQRCode(url);

    if (qrSvg) {
      const qrSection = createQRDisplayElement(qrSvg);
      container.appendChild(qrSection);
    }
  } catch (error) {
    // showing QR codes is optional, so we can ignore errors
  }
}

/**
 * Create QR code display element
 */
function createQRDisplayElement(qrSvg: string): HTMLElement {
  const qrSection = document.createElement("div");
  hideElement(qrSection);
  qrSection.className = "qr-code-section";

  const qrLabel = document.createElement("label");
  qrLabel.textContent = window.i18n.t(I18nKeys.Label.QrCode);
  qrSection.appendChild(qrLabel);

  const qrContainer = document.createElement("div");
  qrContainer.className = "qr-code-container";
  qrContainer.innerHTML = qrSvg;
  qrContainer.title = "Click to view full screen";

  // Add click handler for fullscreen
  qrContainer.addEventListener("click", () => {
    showQRFullscreen(qrSvg);
  });

  qrSection.appendChild(qrContainer);

  return qrSection;
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
