import {
  HakanaiClient,
  ContentAnalysis,
  type PayloadData,
} from "./hakanai-client.js";
import {
  announceToScreenReader,
  copyToClipboard,
  createButton,
  createButtonContainer,
  debounce,
  formatFileSize,
  generateRandomId,
  initTheme,
  secureInputClear,
  updateThemeToggleButton,
} from "./common-utils.js";
import { isHakanaiError, isStandardError, isErrorLike } from "./types.js";

interface UIStrings {
  BINARY_DETECTED: string;
  COPY_ARIA: string;
  COPY_FAILED: string;
  COPY_TEXT: string;
  DOWNLOAD_ARIA: string;
  DOWNLOAD_TEXT: string;
  EMPTY_URL: string;
  ERROR_TITLE: string;
  FILENAME_LABEL: string;
  INVALID_URL: string;
  MISSING_KEY: string;
  NOTE_TEXT: string;
  RETRIEVE_FAILED: string;
  SUCCESS_TITLE: string;
}

const TIMEOUTS = {
  DEBOUNCE: 300,
  CLEANUP_DELAY: 100,
} as const;

const UI_STRINGS: UIStrings = {
  BINARY_DETECTED:
    "Binary file detected. Content hidden for security. Use download button to save the file.",
  COPY_ARIA: "Copy secret to clipboard",
  COPY_FAILED: "Failed to copy. Please select and copy manually.",
  COPY_TEXT: "Copy Secret",
  DOWNLOAD_ARIA: "Download secret as file",
  DOWNLOAD_TEXT: "Download",
  EMPTY_URL: "Please enter a valid secret URL",
  ERROR_TITLE: "Error",
  FILENAME_LABEL: "Filename:",
  INVALID_URL:
    "Invalid URL format. Please include the full URL with the secret key after #",
  MISSING_KEY: "Please enter the decryption key",
  NOTE_TEXT:
    "Note: This secret has been deleted from the server and cannot be accessed again.",
  RETRIEVE_FAILED: "Failed to retrieve secret",
  SUCCESS_TITLE: "Secret Retrieved Successfully",
};

const baseUrl = window.location.origin.includes("file://")
  ? "http://localhost:8080"
  : window.location.origin;

const client = new HakanaiClient(baseUrl);

declare global {
  interface Window {
    i18n: {
      t(key: string): string;
    };
  }
}

function updateUIStrings(): void {
  // Check if i18n is available before using it
  if (!window.i18n?.t) {
    return;
  }

  UI_STRINGS.BINARY_DETECTED = window.i18n.t("msg.binaryDetected");
  UI_STRINGS.COPY_ARIA = window.i18n.t("aria.copySecret");
  UI_STRINGS.COPY_FAILED = window.i18n.t("msg.copyFailed");
  UI_STRINGS.COPY_TEXT = window.i18n.t("button.copySecret");
  UI_STRINGS.DOWNLOAD_ARIA = window.i18n.t("aria.downloadSecret");
  UI_STRINGS.DOWNLOAD_TEXT = window.i18n.t("button.download");
  UI_STRINGS.EMPTY_URL = window.i18n.t("msg.emptyUrl");
  UI_STRINGS.ERROR_TITLE = window.i18n.t("msg.errorTitle");
  UI_STRINGS.FILENAME_LABEL = window.i18n.t("label.filename");
  UI_STRINGS.INVALID_URL = window.i18n.t("msg.invalidUrl");
  UI_STRINGS.MISSING_KEY = window.i18n.t("msg.missingKey");
  UI_STRINGS.NOTE_TEXT = window.i18n.t("msg.retrieveNote");
  UI_STRINGS.RETRIEVE_FAILED = window.i18n.t("msg.retrieveFailed");
  UI_STRINGS.SUCCESS_TITLE = window.i18n.t("msg.successTitle");
}

function getElements() {
  return {
    urlInput: document.getElementById("secretUrl") as HTMLInputElement,
    keyInput: document.getElementById("secretKey") as HTMLInputElement,
    keyInputGroup: document.getElementById("keyInputGroup") as HTMLElement,
    resultDiv: document.getElementById("result") as HTMLElement,
    loadingDiv: document.getElementById("loading") as HTMLElement,
    button: document.getElementById("retrieveBtn") as HTMLButtonElement,
  };
}

function normalizeUrl(url: string): string {
  if (!url.match(/^[a-zA-Z][a-zA-Z0-9+.-]*:\/\//)) {
    return window.location.protocol + "//" + url;
  }
  return url;
}

function hasUrlFragment(url: string): boolean {
  try {
    const urlObj = new URL(url);
    return !!(urlObj.hash && urlObj.hash.length > 1);
  } catch {
    return false;
  }
}

function validateInputs(
  url: string,
  key: string,
  hasFragment: boolean,
): string | null {
  if (!url) return UI_STRINGS.EMPTY_URL;
  if (!hasFragment && !key) return UI_STRINGS.MISSING_KEY;
  return null;
}

function setElementsState(disabled: boolean): void {
  const { urlInput, keyInput, button } = getElements();
  button.disabled = disabled;
  urlInput.disabled = disabled;
  keyInput.disabled = disabled;
}

function clearInputs(): void {
  const { urlInput, keyInput } = getElements();
  secureInputClear(urlInput);
  secureInputClear(keyInput);
}

function showLoadingState(): void {
  const { loadingDiv, resultDiv } = getElements();
  loadingDiv.classList.add("visible");
  loadingDiv.classList.remove("hidden");
  resultDiv.innerHTML = "";
  document.body.classList.remove("expanded-view");
  setElementsState(true);
}

function hideLoadingState(): void {
  const { loadingDiv } = getElements();
  loadingDiv.classList.add("hidden");
  loadingDiv.classList.remove("visible");
  setElementsState(false);
}

async function processRetrieveRequest(): Promise<void> {
  const { urlInput, keyInput } = getElements();
  const url = urlInput.value.trim();
  const key = keyInput.value.trim();

  const processedUrl = normalizeUrl(url);
  const hasFragment = hasUrlFragment(processedUrl);

  try {
    new URL(processedUrl);
  } catch {
    showError(UI_STRINGS.INVALID_URL);
    urlInput.focus();
    return;
  }

  const validationError = validateInputs(url, key, hasFragment);
  if (validationError) {
    showError(validationError);
    (validationError === UI_STRINGS.EMPTY_URL ? urlInput : keyInput).focus();
    return;
  }

  const finalUrl = hasFragment ? processedUrl : `${processedUrl}#${key}`;

  showLoadingState();

  try {
    const payload = await client.receivePayload(finalUrl);
    showSuccess(payload);
    clearInputs();
    toggleKeyInputVisibility();
  } catch (error: unknown) {
    handleRetrieveError(error);
  } finally {
    hideLoadingState();
  }
}

function handleRetrieveError(error: unknown): void {
  if (isHakanaiError(error)) {
    const errorKey = `error.${error.code}`;
    const localizedMessage = window.i18n.t(errorKey);
    const finalMessage =
      localizedMessage !== errorKey ? localizedMessage : error.message;
    showError(finalMessage);
  } else if (isStandardError(error)) {
    showError(error.message);
  } else if (isErrorLike(error)) {
    showError(error.message ?? UI_STRINGS.RETRIEVE_FAILED);
  } else {
    showError(UI_STRINGS.RETRIEVE_FAILED);
  }
}

const retrieveSecretDebounced = debounce(
  processRetrieveRequest,
  TIMEOUTS.DEBOUNCE,
);

function retrieveSecret(): void {
  retrieveSecretDebounced();
}

function toggleKeyInputVisibility(): void {
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

function hideKeyInput(
  keyInputGroup: HTMLElement,
  keyInput: HTMLInputElement,
): void {
  keyInputGroup.classList.remove("visible");
  keyInputGroup.classList.add("hidden");
  keyInput.required = false;
  secureInputClear(keyInput);
}

function showKeyInput(
  keyInputGroup: HTMLElement,
  keyInput: HTMLInputElement,
): void {
  keyInputGroup.classList.remove("hidden");
  keyInputGroup.classList.add("visible");
  keyInput.required = true;
}

function createTextSecret(
  payload: PayloadData,
  decodedBytes: Uint8Array,
): HTMLElement {
  const secretId = "secret-" + generateRandomId();
  const container = document.createElement("div");
  container.className = "secret-container";

  const textarea = createSecretTextarea(secretId, payload, decodedBytes);
  container.appendChild(textarea);

  const buttonsContainer = createButtonContainer();
  buttonsContainer.appendChild(createCopyButton(secretId));
  buttonsContainer.appendChild(createDownloadButton(payload, decodedBytes));
  container.appendChild(buttonsContainer);

  if (window.innerWidth > 640) {
    document.body.classList.add("expanded-view");
    resizeTextarea(textarea);
  }

  return container;
}

function createSecretTextarea(
  secretId: string,
  payload: PayloadData,
  decodedBytes: Uint8Array,
): HTMLTextAreaElement {
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
  textarea.style.height = "auto";
  const styles = window.getComputedStyle(textarea);
  const minHeight = parseInt(styles.minHeight);
  const maxHeight = parseInt(styles.maxHeight);
  const scrollHeight = textarea.scrollHeight;
  textarea.style.height =
    Math.min(Math.max(scrollHeight, minHeight), maxHeight) + "px";
}

function createBinarySecret(
  payload: PayloadData,
  decodedBytes: Uint8Array,
): HTMLElement {
  const container = document.createElement("div");
  container.className = "secret-container";

  const message = document.createElement("p");
  message.style.marginBottom = "var(--spacing-md, 1rem)";
  message.style.color = "var(--color-text, #000000)";
  message.textContent = UI_STRINGS.BINARY_DETECTED;
  container.appendChild(message);

  const buttonsContainer = createButtonContainer();
  buttonsContainer.appendChild(
    createDownloadButton(payload, decodedBytes, true),
  );
  container.appendChild(buttonsContainer);

  return container;
}

function createCopyButton(secretId: string): HTMLButtonElement {
  return createButton(
    "copy-button",
    UI_STRINGS.COPY_TEXT,
    UI_STRINGS.COPY_ARIA,
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
    "download-button",
    UI_STRINGS.DOWNLOAD_TEXT,
    UI_STRINGS.DOWNLOAD_ARIA,
    () => downloadSecret(payload, decodedBytes, isBinary),
  );
}

function createFilenameInfo(filename: string, size: number): HTMLElement {
  const fileInfo = document.createElement("p");
  fileInfo.style.marginTop = "var(--spacing-sm, 0.75rem)";
  fileInfo.style.fontSize = "0.875rem";
  fileInfo.style.color = "var(--color-text-muted, #888888)";

  const fileLabel = document.createElement("strong");
  fileLabel.textContent = UI_STRINGS.FILENAME_LABEL + " ";
  fileInfo.appendChild(fileLabel);
  fileInfo.appendChild(document.createTextNode(filename));

  // Add size information
  const sizeSpan = document.createElement("span");
  sizeSpan.textContent = ` (${formatFileSize(size)})`;
  fileInfo.appendChild(sizeSpan);

  return fileInfo;
}

function createNoteElement(): HTMLElement {
  const note = document.createElement("p");
  note.style.marginTop = "var(--spacing-sm, 0.75rem)";
  note.style.fontSize = "0.875rem";
  note.style.color = "var(--color-text-muted, #888888)";

  const strong = document.createElement("strong");
  strong.textContent = window.i18n.t("msg.retrieveNote").split(":")[0] + ":";
  note.appendChild(strong);
  note.appendChild(
    document.createTextNode(" " + window.i18n.t("msg.retrieveNoteText")),
  );

  return note;
}

function showSuccess(payload: PayloadData): void {
  const { resultDiv } = getElements();
  resultDiv.className = "result success";
  resultDiv.innerHTML = "";

  const title = document.createElement("h3");
  title.textContent = UI_STRINGS.SUCCESS_TITLE;
  resultDiv.appendChild(title);

  const decodedBytes = payload.decodeBytes();
  const isBinaryFile =
    payload.filename != null || ContentAnalysis.isBinary(decodedBytes);

  const container = isBinaryFile
    ? createBinarySecret(payload, decodedBytes)
    : createTextSecret(payload, decodedBytes);
  resultDiv.appendChild(container);

  if (payload.filename) {
    resultDiv.appendChild(
      createFilenameInfo(payload.filename, decodedBytes.length),
    );
  }

  resultDiv.appendChild(createNoteElement());
  announceToScreenReader(UI_STRINGS.SUCCESS_TITLE);
}

function showError(message: string): void {
  const { resultDiv } = getElements();
  resultDiv.className = "result error";
  resultDiv.innerHTML = "";
  document.body.classList.remove("expanded-view");

  const title = document.createElement("h3");
  title.textContent = UI_STRINGS.ERROR_TITLE;
  resultDiv.appendChild(title);

  const errorDiv = document.createElement("div");
  errorDiv.textContent = message;
  resultDiv.appendChild(errorDiv);

  announceToScreenReader(`${UI_STRINGS.ERROR_TITLE}: ${message}`);
}

function copySecret(secretId: string, button: HTMLButtonElement): void {
  const secretElement = document.getElementById(
    secretId,
  ) as HTMLTextAreaElement;
  if (!secretElement) {
    showError(UI_STRINGS.COPY_FAILED);
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

function downloadSecret(
  payload: PayloadData,
  decodedBytes: Uint8Array,
  isBinary: boolean,
): void {
  const filename = generateFilename(payload, isBinary);
  const mimeType = payload.filename
    ? "application/octet-stream"
    : "text/plain;charset=utf-8";

  const blob = new Blob([decodedBytes], { type: mimeType });
  const url = window.URL.createObjectURL(blob);

  const anchor = document.createElement("a");
  anchor.classList.add("hidden");
  anchor.href = url;
  anchor.download = filename;

  document.body.appendChild(anchor);
  anchor.click();

  setTimeout(() => {
    document.body.removeChild(anchor);
    window.URL.revokeObjectURL(url);
  }, TIMEOUTS.CLEANUP_DELAY);

  announceToScreenReader(window.i18n.t("msg.downloaded"));
}

function setupUrlInput(): void {
  const urlInput = document.getElementById("secretUrl") as HTMLInputElement;
  if (!urlInput) return;

  urlInput.placeholder = `${baseUrl}/s/uuid#key`;

  if (window.location.pathname.match(/^\/s\/[^\/]+$/)) {
    urlInput.value = window.location.href;
  }

  urlInput.addEventListener("input", toggleKeyInputVisibility);
  urlInput.addEventListener("paste", () =>
    setTimeout(toggleKeyInputVisibility, 0),
  );
  toggleKeyInputVisibility();
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

document.addEventListener("languageChanged", () => {
  updateUIStrings();
  updateThemeToggleButton();
});

document.addEventListener("i18nInitialized", () => {
  updateUIStrings();
  updateThemeToggleButton();
});

document.addEventListener("DOMContentLoaded", () => {
  initTheme();
  updateUIStrings();
  setupForm();
  setupUrlInput();
});

// Export functions for testing
export {
  normalizeUrl,
  hasUrlFragment,
  validateInputs,
  generateFilename,
  updateUIStrings,
  UI_STRINGS,
};
