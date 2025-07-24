import {
  createButton,
  generateRandomId,
  hideElement,
} from "../core/dom-utils.js";
import { copyToClipboardByElementId } from "../core/clipboard.js";
import { QRCodeGenerator } from "../core/qr-generator.js";

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
  title.textContent =
    window.i18n?.t("msg.successTitle") ?? "Secret Created Successfully";
  container.appendChild(title);

  const instructions = document.createElement("p");
  instructions.className = "share-instructions";
  instructions.textContent =
    window.i18n?.t("msg.shareInstructions") ??
    "Share this URL with the intended recipient. The secret is encrypted and can only be accessed once.";
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
 * Create combined URL display (traditional mode)
 */
function createCombinedUrlDisplay(container: HTMLElement, url: string): void {
  const urlId = generateRandomId();

  const label = document.createElement("label");
  label.textContent = window.i18n?.t("label.url") ?? "Secret URL:";
  label.setAttribute("for", urlId);
  container.appendChild(label);

  const inputContainer = document.createElement("div");
  inputContainer.className = "input-group";

  const urlInput = document.createElement("input");
  urlInput.type = "text";
  urlInput.id = urlId;
  urlInput.value = url;
  urlInput.readOnly = true;
  urlInput.className = "url-input";
  inputContainer.appendChild(urlInput);

  const copyButton = createButton(
    "copy-button",
    window.i18n?.t("button.copy") ?? "Copy URL",
    "Copy secret URL to clipboard",
    () => copyToClipboardByElementId(urlId, copyButton as HTMLButtonElement),
  );
  inputContainer.appendChild(copyButton);

  container.appendChild(inputContainer);
}

/**
 * Create separate URL and key display (enhanced security mode)
 */
function createSeparateUrlDisplay(
  container: HTMLElement,
  fullUrl: string,
): void {
  const [url, key] = fullUrl.split("#");
  const id = generateRandomId();
  const urlId = id;
  const keyId = id + "-key";

  // URL section
  const urlLabel = document.createElement("label");
  urlLabel.textContent = window.i18n?.t("label.url") ?? "Secret URL:";
  urlLabel.setAttribute("for", urlId);
  container.appendChild(urlLabel);

  const urlInputContainer = document.createElement("div");
  urlInputContainer.className = "input-group";

  const urlInput = document.createElement("input");
  urlInput.type = "text";
  urlInput.id = urlId;
  urlInput.value = url;
  urlInput.readOnly = true;
  urlInput.className = "url-input";
  urlInputContainer.appendChild(urlInput);

  const urlCopyButton = createButton(
    "copy-button",
    window.i18n?.t("button.copy") ?? "Copy URL",
    "Copy secret URL to clipboard",
    () =>
      copyToClipboardByElementId(
        urlInput.id,
        urlCopyButton as HTMLButtonElement,
      ),
  );
  urlInputContainer.appendChild(urlCopyButton);
  container.appendChild(urlInputContainer);

  // Key section
  const keyLabel = document.createElement("label");
  keyLabel.textContent = window.i18n?.t("label.key") ?? "Decryption Key:";
  keyLabel.setAttribute("for", keyId);
  container.appendChild(keyLabel);

  const keyInputContainer = document.createElement("div");
  keyInputContainer.className = "input-group";

  const keyInput = document.createElement("input");
  keyInput.type = "text";
  keyInput.id = keyId;
  keyInput.value = key;
  keyInput.readOnly = true;
  keyInput.className = "url-input";
  keyInputContainer.appendChild(keyInput);

  const keyCopyButton = createButton(
    "copy-button",
    window.i18n?.t("button.copy") ?? "Copy Key",
    "Copy decryption key to clipboard",
    () =>
      copyToClipboardByElementId(
        keyInput.id,
        keyCopyButton as HTMLButtonElement,
      ),
  );
  keyInputContainer.appendChild(keyCopyButton);
  container.appendChild(keyInputContainer);
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
  qrLabel.textContent = window.i18n?.t("label.qrCode") ?? "QR Code:";
  qrSection.appendChild(qrLabel);

  const qrContainer = document.createElement("div");
  qrContainer.className = "qr-code-container";
  qrContainer.innerHTML = qrSvg;
  qrSection.appendChild(qrContainer);

  return qrSection;
}

/**
 * Create security note section
 */
function createNoteSection(container: HTMLElement): void {
  const note = document.createElement("p");
  note.className = "secret-note";

  const noteText =
    window.i18n?.t("msg.createNote") ??
    "Note: Share this URL carefully. The secret will be deleted after the first access or when it expires.";
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
