// SPDX-License-Identifier: Apache-2.0

import { HakanaiClient, ContentAnalysis, type PayloadData } from "./hakanai-client";
import { initI18n, I18nKeys } from "./core/i18n";
import { KeyboardShortcuts } from "./core/keyboard-shortcuts";
import {
  announceToScreenReader,
  createButton,
  createButtonContainer,
  debounce,
  generateRandomId,
  hideElement,
  secureInputClear,
  showElement,
  expandView,
  unexpandView,
} from "./core/dom-utils";
import { copyToClipboard } from "./core/clipboard";
import { displayErrorMessage } from "./components/error-display";
import { formatFileSize } from "./core/formatters";
import { ProgressBar } from "./components/progress-bar";
import { initTheme } from "./core/theme";
import { ErrorHandler, handleAPIError, isHakanaiError } from "./core/error";
import { initFeatures } from "./core/app-config";
import { HakanaiErrorCodes } from "./hakanai-client";

const TIMEOUTS = {
  DEBOUNCE: 300,
  CLEANUP_DELAY: 100,
} as const;

const baseUrl = window.location.origin.includes("file://") ? "http://localhost:8080" : window.location.origin;

const client = new HakanaiClient(baseUrl);

function getElements() {
  return {
    urlInput: document.getElementById("secretUrl") as HTMLInputElement,
    keyInput: document.getElementById("secretKey") as HTMLInputElement,
    keyInputGroup: document.getElementById("keyInputGroup") as HTMLElement,
    passphraseInput: document.getElementById("passphraseInput") as HTMLInputElement,
    passphraseInputGroup: document.getElementById("passphraseInputGroup") as HTMLElement,
    resultDiv: document.getElementById("result") as HTMLElement,
    button: document.getElementById("retrieveBtn") as HTMLButtonElement,
  };
}

function hasUrlFragment(url: string): boolean {
  try {
    const urlObj = new URL(url);
    return (urlObj.hash?.length ?? 0) > 1;
  } catch {
    return false;
  }
}

function validateInputs(url: string, key: string, hasFragment: boolean): string | null {
  if (!url) return window.i18n.t(I18nKeys.Msg.EmptyUrl);
  if (!hasFragment && !key) return window.i18n.t(I18nKeys.Msg.MissingKey);
  return null;
}

function setElementsState(disabled: boolean): void {
  const { urlInput, keyInput, passphraseInput, button } = getElements();
  button.disabled = disabled;
  urlInput.disabled = disabled;
  keyInput.disabled = disabled;
  passphraseInput.disabled = disabled;
}

function clearInputs(): void {
  const { urlInput, keyInput, passphraseInput } = getElements();
  secureInputClear(urlInput);
  secureInputClear(keyInput);
  secureInputClear(passphraseInput);
}

function clearResult(): void {
  const { resultDiv } = getElements();
  if (!resultDiv) return;

  resultDiv.innerHTML = "";
  hideElement(resultDiv);
}

function resetView() {
  clearResult();
  unexpandView();
}

function showLoadingState(): void {
  setElementsState(true);
}

function hideLoadingState(): void {
  setElementsState(false);
}

async function processRetrieveRequest(): Promise<void> {
  const { urlInput, keyInput, passphraseInput } = getElements();

  const url = urlInput.value.trim();
  const key = keyInput?.value?.trim() ?? "";
  const passphrase = passphraseInput?.value?.trim() ?? "";

  const processedUrl = normalizeUrl(url);
  const hasFragment = hasUrlFragment(processedUrl);

  try {
    new URL(processedUrl);
  } catch {
    showError(window.i18n.t(I18nKeys.Msg.InvalidUrl));
    urlInput.focus();
    return;
  }

  const validationError = validateInputs(url, key, hasFragment);
  if (validationError) {
    showError(validationError);
    (validationError.includes("emptyUrl") ? urlInput : keyInput).focus();
    return;
  }

  const finalUrl = hasFragment ? processedUrl : `${processedUrl}#${key}`;

  await performRetrieval(finalUrl, passphrase || undefined);
}

function normalizeUrl(url: string): string {
  if (!url.startsWith("http://") && !url.startsWith("https://")) {
    return window.location.protocol + "//" + url;
  }
  return url;
}

async function performRetrieval(url: string, passphrase?: string): Promise<void> {
  const progressBar = new ProgressBar();
  progressBar.show(window.i18n.t(I18nKeys.Msg.Retrieving));
  resetView();
  showLoadingState();

  try {
    const payload = await client.receivePayload(url, progressBar, passphrase);
    showSuccess(payload);
    clearInputs();
    updateKeyInputVisibility();
    hidePassphraseInput();
    hideForm();
  } catch (error: unknown) {
    handleRetrieveError(error, url);
  } finally {
    progressBar.hide();
    hideLoadingState();
  }
}

function showError(message: string): void {
  unexpandView();

  const resultContainer = document.getElementById("result")!;
  displayErrorMessage(message, resultContainer);
}

class GetSecretErrorHandler implements ErrorHandler {
  displayError(message: string): void {
    showError(message);
  }
}

const errorHandler = new GetSecretErrorHandler();

function handleRetrieveError(error: unknown, url?: string): void {
  if (url && isHakanaiError(error) && error.code === HakanaiErrorCodes.PASSPHRASE_REQUIRED) {
    showPassphraseInput();
    return;
  }

  handleAPIError(error, window.i18n.t(I18nKeys.Msg.RetrieveFailed), errorHandler);
}

function showPassphraseInput(): void {
  const { passphraseInputGroup, passphraseInput } = getElements();
  showElement(passphraseInputGroup);
  passphraseInput.required = true;
  // Small delay to ensure element is fully rendered before focusing
  setTimeout(() => {
    passphraseInput.focus();
  }, 100);
}

function hidePassphraseInput(): void {
  const { passphraseInputGroup, passphraseInput } = getElements();
  hideElement(passphraseInputGroup);
  passphraseInput.required = false;
}

function hideForm(): void {
  const form = document.getElementById("secretForm");
  if (!form) return;

  hideElement(form);
}

function resetForm(): void {
  const form = document.getElementById("secretForm");
  if (!form) return;

  resetView();
  showElement(form);

  // Reset focus to URL input
  const { urlInput } = getElements();
  setTimeout(() => {
    urlInput.focus();
  }, 100);
}

const retrieveSecretDebounced = debounce(processRetrieveRequest, TIMEOUTS.DEBOUNCE);

function retrieveSecret(): void {
  retrieveSecretDebounced();
}

function updateKeyInputVisibility(): void {
  const { urlInput, keyInputGroup, keyInput } = getElements();

  const url = urlInput.value.trim();

  if (!url) {
    hideKeyInput(keyInputGroup, keyInput);
    return;
  }

  const processedUrl = normalizeUrl(url);
  const hasFragment = hasUrlFragment(processedUrl);

  if (hasFragment) {
    hideKeyInput(keyInputGroup, keyInput);
  } else {
    showKeyInput(keyInputGroup, keyInput);
  }
}

function hideKeyInput(keyInputGroup: HTMLElement, keyInput: HTMLInputElement): void {
  hideElement(keyInputGroup);
  keyInput.required = false;
  secureInputClear(keyInput);
}

function showKeyInput(keyInputGroup: HTMLElement, keyInput: HTMLInputElement): void {
  showElement(keyInputGroup);
  keyInput.required = true;
}

function createTextSecret(payload: PayloadData, decodedBytes: Uint8Array): HTMLElement {
  const secretId = "secret-" + generateRandomId();
  const container = document.createElement("div");
  container.className = "secret-container";

  const textarea = createSecretTextarea(secretId, decodedBytes);
  container.appendChild(textarea);

  const buttonsContainer = createButtonContainer();
  buttonsContainer.appendChild(createCopyButton(secretId));
  buttonsContainer.appendChild(createDownloadButton(payload, decodedBytes));
  container.appendChild(buttonsContainer);

  expandView();
  if (window.innerWidth > 640) {
    resizeTextarea(textarea);
  }

  return container;
}

function createSecretTextarea(secretId: string, decodedBytes: Uint8Array): HTMLTextAreaElement {
  const textarea = document.createElement("textarea");
  textarea.id = secretId;
  textarea.className = "secret-display";
  textarea.readOnly = true;
  textarea.setAttribute("aria-label", "Retrieved secret content");

  // Use TextDecoder with pre-decoded bytes for better performance
  const decoder = new TextDecoder();
  textarea.value = decoder.decode(decodedBytes);

  textarea.addEventListener("click", () => textarea.select());
  return textarea;
}

function resizeTextarea(textarea: HTMLTextAreaElement): void {
  // Use CSS custom properties to set height without inline styles
  const styles = window.getComputedStyle(textarea);
  const minHeight = parseInt(styles.minHeight);
  const maxHeight = parseInt(styles.maxHeight);
  const scrollHeight = textarea.scrollHeight;
  const height = Math.min(Math.max(scrollHeight, minHeight), maxHeight);

  // Set CSS custom property instead of inline style
  textarea.style.setProperty("--textarea-height", height + "px");
  textarea.classList.add("auto-height");
}

function createBinarySecret(payload: PayloadData, decodedBytes: Uint8Array): HTMLElement {
  const container = document.createElement("div");
  container.className = "secret-container";

  const message = document.createElement("p");
  message.className = "binary-message";
  message.textContent = window.i18n.t(I18nKeys.Msg.BinaryDetected);
  container.appendChild(message);

  const buttonsContainer = createButtonContainer();
  buttonsContainer.appendChild(createDownloadButton(payload, decodedBytes, true));
  container.appendChild(buttonsContainer);

  return container;
}

function createCopyButton(secretId: string): HTMLButtonElement {
  return createButton(
    "btn copy-btn",
    window.i18n.t(I18nKeys.Button.Copy),
    window.i18n.t(I18nKeys.Aria.CopySecret),
    function (this: HTMLButtonElement) {
      copySecret(secretId, this);
    },
  );
}

function createDownloadButton(
  payload: PayloadData,
  decodedBytes: Uint8Array,
  isBinary: boolean = false,
): HTMLButtonElement {
  return createButton(
    "btn download-btn",
    window.i18n.t(I18nKeys.Button.Download),
    window.i18n.t(I18nKeys.Aria.DownloadSecret),
    () => downloadSecret(payload, decodedBytes, isBinary),
  );
}

function createFilenameInfo(filename: string, size: number): HTMLElement {
  const fileInfo = document.createElement("p");
  fileInfo.className = "file-info";

  const fileLabel = document.createElement("strong");
  fileLabel.textContent = window.i18n.t(I18nKeys.Label.Filename) + " ";
  fileInfo.appendChild(fileLabel);
  fileInfo.appendChild(document.createTextNode(filename));

  // Add size information
  const sizeSpan = document.createElement("span");
  sizeSpan.textContent = ` (${formatFileSize(size)})`;
  fileInfo.appendChild(sizeSpan);

  return fileInfo;
}

function createNoteElement(): HTMLElement {
  const container = document.createElement("div");
  container.className = "note-container";

  const note = document.createElement("p");
  note.className = "note-element";
  note.appendChild(document.createTextNode("⚠️ " + window.i18n.t(I18nKeys.Msg.RetrieveNote)));
  container.appendChild(note);

  // Add CTA below the destruction note
  const cta = document.createElement("p");
  cta.className = "retrieve-cta";

  const ctaLink = document.createElement("a");
  ctaLink.href = "/";
  ctaLink.textContent = window.i18n.t(I18nKeys.Msg.RetrieveCTA) + " →";
  ctaLink.setAttribute("aria-label", "Learn more about Hakanai and create your own secrets");
  cta.appendChild(ctaLink);

  container.appendChild(cta);

  // Add separator line
  const separator = document.createElement("hr");
  separator.className = "section-separator";
  container.appendChild(separator);

  // Add "Retrieve Another Secret" button with proper spacing and centering
  const buttonContainer = document.createElement("div");
  buttonContainer.className = "retrieve-another-container";

  const retrieveAnotherButton = createButton(
    "btn secondary",
    window.i18n.t(I18nKeys.Button.RetrieveAnother),
    "Show the form again to retrieve another secret",
    () => resetForm(),
  );
  buttonContainer.appendChild(retrieveAnotherButton);
  container.appendChild(buttonContainer);

  return container;
}

function showSuccess(payload: PayloadData): void {
  clearResult();

  const { resultDiv } = getElements();
  resultDiv.className = "result success";

  const title = document.createElement("h3");
  title.textContent = window.i18n.t(I18nKeys.Msg.SuccessTitle);
  resultDiv.appendChild(title);

  const decodedBytes = payload.decodeBytes();
  const isBinaryFile = payload.filename != null || ContentAnalysis.isBinary(decodedBytes);

  const container = isBinaryFile ? createBinarySecret(payload, decodedBytes) : createTextSecret(payload, decodedBytes);
  resultDiv.appendChild(container);

  if (payload.filename) {
    resultDiv.appendChild(createFilenameInfo(payload.filename, decodedBytes.length));
  }

  resultDiv.appendChild(createNoteElement());
  announceToScreenReader(window.i18n.t(I18nKeys.Msg.SuccessTitle));
}

function copySecret(secretId: string, button: HTMLButtonElement): void {
  const secretElement = document.getElementById(secretId) as HTMLTextAreaElement;
  if (!secretElement) {
    showError(window.i18n.t(I18nKeys.Msg.CopyFailed));
    return;
  }

  copyToClipboard(secretElement.value, button);
}

function generateFilename(payload: PayloadData, isBinary: boolean): string {
  if (payload.filename) {
    return payload.filename;
  }

  const timestamp = new Date().toISOString().replace(/[:.]/g, "-");

  const extension = isBinary ? ".bin" : ".txt";

  return `hakanai-secret-${timestamp}${extension}`;
}

function downloadSecret(payload: PayloadData, decodedBytes: Uint8Array, isBinary: boolean): void {
  const filename = generateFilename(payload, isBinary);
  const mimeType = payload.filename ? "application/octet-stream" : "text/plain;charset=utf-8";

  const blob = new Blob([decodedBytes], { type: mimeType });
  const url = window.URL.createObjectURL(blob);

  const anchor = document.createElement("a");
  hideElement(anchor);
  anchor.href = url;
  anchor.download = filename;

  document.body.appendChild(anchor);
  anchor.click();

  setTimeout(() => {
    document.body.removeChild(anchor);
    window.URL.revokeObjectURL(url);
  }, TIMEOUTS.CLEANUP_DELAY);

  announceToScreenReader(window.i18n.t(I18nKeys.Msg.Downloaded));
}

function setupUrlInput(): void {
  const urlInput = document.getElementById("secretUrl") as HTMLInputElement;
  const keyInput = document.getElementById("secretKey") as HTMLInputElement;

  urlInput.placeholder = `${baseUrl}/s/uuid#key`;

  if (window.location.pathname.match(/^\/s\/[^\/]+$/)) {
    urlInput.value = window.location.href;
  }

  urlInput.addEventListener("input", updateKeyInputVisibility);
  urlInput.addEventListener("paste", () =>
    setTimeout(() => {
      updateKeyInputVisibility();
      focusNextLogicalElement();
    }, 0),
  );
  urlInput.addEventListener("blur", () => {
    urlInput.value = urlInput.value.trim();
  });

  keyInput.addEventListener("paste", () => setTimeout(focusNextLogicalElement, 0));
  keyInput.addEventListener("blur", () => {
    keyInput.value = keyInput.value.trim();
  });

  updateKeyInputVisibility();
}

function setupForm(): void {
  const form = document.querySelector("form");
  if (form) {
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      retrieveSecret();
    });
  }
}

function focusNextLogicalElement(): void {
  const { urlInput, keyInput, button } = getElements();

  const url = urlInput.value.trim();
  const key = keyInput.value.trim();

  if (!url) {
    urlInput.focus();
    return;
  }

  if (hasUrlFragment(url)) {
    button.focus();
  } else if (!key) {
    keyInput.focus();
  } else {
    button.focus();
  }
}

function setupSmartFocus(): void {
  const { urlInput } = getElements();

  // ensure DOM is ready before focusing
  setTimeout(() => {
    if (urlInput.value.trim()) {
      focusNextLogicalElement();
    } else {
      urlInput.focus();
    }
  }, 100);
}

function initKeyboardShortcuts(): void {
  const shortcuts = new KeyboardShortcuts();

  // Ctrl + Enter to retrieve secret
  shortcuts.register({
    key: "Enter",
    ctrl: true,
    handler: () => {
      retrieveSecret();
    },
    description: "Retrieve secret",
  });
}

document.addEventListener("DOMContentLoaded", () => {
  initI18n();
  initTheme();
  setupForm();
  setupUrlInput();
  initFeatures();
  initKeyboardShortcuts();
  setupSmartFocus();
});

// Export functions for testing
export { normalizeUrl, hasUrlFragment, validateInputs, generateFilename };
