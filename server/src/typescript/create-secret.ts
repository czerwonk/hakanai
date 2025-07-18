import {
  HakanaiClient,
  HakanaiErrorCodes,
  type PayloadData,
} from "./hakanai-client.js";
import {
  createButton,
  createButtonContainer,
  copyToClipboard,
  announceToScreenReader,
  secureInputClear,
  initTheme,
  updateThemeToggleButton,
  saveAuthTokenToStorage,
  getAuthTokenFromStorage,
  clearAuthTokenStorage,
} from "./common-utils.js";
import {
  type RequiredElements,
  type FileElements,
  type FormValues,
  isHakanaiError,
  isStandardError,
  isErrorLike,
} from "./types.js";

interface UIStrings {
  EMPTY_SECRET: string;
  EMPTY_FILE: string;
  CREATE_FAILED: string;
  SUCCESS_TITLE: string;
  ERROR_TITLE: string;
  COPY_TEXT: string;
  COPIED_TEXT: string;
  COPY_FAILED: string;
  NOTE_TEXT: string;
  SHARE_INSTRUCTIONS: string;
  FILE_TOO_LARGE: string;
  FILE_READ_ERROR: string;
  INVALID_FILENAME: string;
}

// Use the RequiredElements type from types.ts
type Elements = RequiredElements;

const UI_STRINGS: UIStrings = {
  EMPTY_SECRET: "Please enter a secret to share",
  EMPTY_FILE: "Please select a file to share",
  CREATE_FAILED: "Failed to create secret",
  SUCCESS_TITLE: "Secret Created Successfully",
  ERROR_TITLE: "Error",
  COPY_TEXT: "Copy URL",
  COPIED_TEXT: "Copied!",
  COPY_FAILED: "Failed to copy. Please select and copy manually.",
  NOTE_TEXT:
    "Note: Share this URL carefully. The secret will be deleted after the first access or when it expires.",
  SHARE_INSTRUCTIONS:
    "Share this URL with the intended recipient. The secret is encrypted and can only be accessed once.",
  FILE_TOO_LARGE: "File size exceeds 10MB limit",
  FILE_READ_ERROR: "Error reading file",
  INVALID_FILENAME: "Invalid filename. Please select a file with a valid name.",
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

  UI_STRINGS.EMPTY_SECRET = window.i18n.t("msg.emptySecret");
  UI_STRINGS.EMPTY_FILE = window.i18n.t("msg.emptyFile");
  UI_STRINGS.CREATE_FAILED = window.i18n.t("msg.createFailed");
  UI_STRINGS.SUCCESS_TITLE = window.i18n.t("msg.successTitle");
  UI_STRINGS.ERROR_TITLE = window.i18n.t("msg.errorTitle");
  UI_STRINGS.COPY_TEXT = window.i18n.t("button.copy");
  UI_STRINGS.COPIED_TEXT = window.i18n.t("button.copied");
  UI_STRINGS.COPY_FAILED = window.i18n.t("msg.copyFailed");
  UI_STRINGS.NOTE_TEXT = window.i18n.t("msg.createNote");
  UI_STRINGS.SHARE_INSTRUCTIONS = window.i18n.t("msg.shareInstructions");
  UI_STRINGS.FILE_TOO_LARGE = window.i18n.t("msg.fileTooLarge");
  UI_STRINGS.FILE_READ_ERROR = window.i18n.t("msg.fileReadError");
  UI_STRINGS.INVALID_FILENAME = window.i18n.t("msg.invalidFilename");
}

function getElements(): Elements | null {
  const loadingDiv = document.getElementById("loading");
  const button = document.getElementById("createBtn") as HTMLButtonElement;
  const secretInput = document.getElementById("secretText") as HTMLInputElement;
  const fileInput = document.getElementById("secretFile") as HTMLInputElement;
  const authTokenInput = document.getElementById(
    "authToken",
  ) as HTMLInputElement;
  const ttlSelect = document.getElementById("ttlSelect") as HTMLSelectElement;
  const textRadio = document.getElementById("textRadio") as HTMLInputElement;
  const fileRadio = document.getElementById("fileRadio") as HTMLInputElement;
  const resultDiv = document.getElementById("result");

  if (
    !loadingDiv ||
    !button ||
    !secretInput ||
    !fileInput ||
    !authTokenInput ||
    !ttlSelect ||
    !textRadio ||
    !fileRadio ||
    !resultDiv
  ) {
    return null;
  }

  return {
    loadingDiv,
    button,
    secretInput,
    fileInput,
    authTokenInput,
    ttlSelect,
    textRadio,
    fileRadio,
    resultDiv,
  };
}

function sanitizeFileName(fileName: string): string | null {
  const sanitized = fileName
    .replace(/[<>:"/\\|?*\x00-\x1f]/g, "_")
    .replace(/^\.+/, "")
    .substring(0, 255);

  return sanitized.length > 0 ? sanitized : null;
}

function readFileAsBytes(file: File): Promise<Uint8Array> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const arrayBuffer = e.target?.result as ArrayBuffer;
        const bytes = new Uint8Array(arrayBuffer);
        resolve(bytes);
      } catch (error) {
        reject(error);
      }
    };
    reader.onerror = () => reject(new Error("Failed to read file"));
    reader.readAsArrayBuffer(file);
  });
}

function validateFilename(fileName: string): boolean {
  return sanitizeFileName(fileName) !== null;
}

async function validateAndProcessFileInput(
  fileInput: HTMLInputElement,
): Promise<PayloadData | null> {
  const file = fileInput.files?.[0];
  if (!file) {
    showError(UI_STRINGS.EMPTY_FILE);
    fileInput.focus();
    return null;
  }

  const fileName = sanitizeFileName(file.name);
  if (!validateFilename(file.name)) {
    showError(UI_STRINGS.INVALID_FILENAME);
    fileInput.focus();
    return null;
  }

  try {
    const fileBytes = await readFileAsBytes(file);
    const payload = client.createPayload(fileName!);
    payload.setFromBytes?.(fileBytes);
    return payload;
  } catch {
    showError(UI_STRINGS.FILE_READ_ERROR);
    return null;
  }
}

function validateTextInput(secretInput: HTMLInputElement): PayloadData | null {
  const secret = secretInput.value.trim();
  if (!secret) {
    showError(UI_STRINGS.EMPTY_SECRET);
    secretInput.focus();
    return null;
  }

  if (typeof TextEncoder === "undefined") {
    showError(
      "Your browser doesn't support text encoding. Please use a modern browser.",
    );
    return null;
  }

  const encoder = new TextEncoder();
  const textBytes = encoder.encode(secret);
  const payload = client.createPayload();
  payload.setFromBytes?.(textBytes);
  return payload;
}

function setElementsState(elements: Elements, disabled: boolean): void {
  const {
    loadingDiv,
    button,
    secretInput,
    fileInput,
    authTokenInput,
    ttlSelect,
    textRadio,
    fileRadio,
    resultDiv,
  } = elements;

  if (disabled) {
    loadingDiv.classList.add("visible");
    loadingDiv.classList.remove("hidden");
  } else {
    loadingDiv.classList.add("hidden");
    loadingDiv.classList.remove("visible");
  }
  button.disabled = disabled;
  secretInput.disabled = disabled;
  fileInput.disabled = disabled;
  authTokenInput.disabled = disabled;
  ttlSelect.disabled = disabled;
  textRadio.disabled = disabled;
  fileRadio.disabled = disabled;

  if (disabled) {
    resultDiv.innerHTML = "";
  }
}

function clearInputs(
  secretInput: HTMLInputElement,
  fileInput: HTMLInputElement,
): void {
  secureInputClear(secretInput);
  fileInput.value = "";
  updateFileInfo();
}

function getFormValues(elements: Elements): FormValues {
  return {
    authToken: elements.authTokenInput.value.trim(),
    ttl: parseInt(elements.ttlSelect.value),
    isFileMode: elements.fileRadio.checked,
  };
}

function handleCreateError(error: unknown): void {
  if (isHakanaiError(error)) {
    const errorKey = `error.${error.code}`;
    const localizedMessage = window.i18n?.t(errorKey) ?? error.code;
    const finalMessage =
      localizedMessage !== errorKey ? localizedMessage : error.message;
    showError(finalMessage);

    // Focus auth token input for authentication errors
    if (
      error.code === HakanaiErrorCodes.AUTHENTICATION_REQUIRED ||
      error.code === HakanaiErrorCodes.INVALID_TOKEN
    ) {
      const authTokenInput = document.getElementById(
        "authToken",
      ) as HTMLInputElement;
      if (authTokenInput) {
        authTokenInput.focus();
        authTokenInput.select();
      }
    }
  } else if (isStandardError(error)) {
    showError(error.message);
  } else if (isErrorLike(error)) {
    showError(error.message ?? UI_STRINGS.CREATE_FAILED);
  } else {
    showError(UI_STRINGS.CREATE_FAILED);
  }
}

async function createSecret(): Promise<void> {
  const elements = getElements();
  if (!elements) {
    showError("Page not fully loaded. Please refresh and try again.");
    return;
  }

  const { authToken, ttl, isFileMode } = getFormValues(elements);

  const payload = isFileMode
    ? await validateAndProcessFileInput(elements.fileInput)
    : validateTextInput(elements.secretInput);

  if (!payload) {
    return;
  }

  setElementsState(elements, true);

  try {
    const secretUrl = await client.sendPayload(payload, ttl, authToken);

    // Handle auth token cookie saving
    const saveTokenCookie = document.getElementById(
      "saveTokenCookie",
    ) as HTMLInputElement;
    if (saveTokenCookie) {
      handleAuthTokenSave(authToken, saveTokenCookie.checked);
    }

    showSuccess(secretUrl);
    clearInputs(elements.secretInput, elements.fileInput);
  } catch (error: unknown) {
    handleCreateError(error);
  } finally {
    setElementsState(elements, false);
    // Clear auth token from memory for security (unless saving to cookie)
    const saveTokenCookie = document.getElementById(
      "saveTokenCookie",
    ) as HTMLInputElement;
    if (elements.authTokenInput.value && !saveTokenCookie?.checked) {
      secureInputClear(elements.authTokenInput);
    }
  }
}

function generateUrlId(): string {
  return crypto?.randomUUID && typeof crypto.randomUUID === "function"
    ? `url-${crypto.randomUUID()}`
    : `url-${Date.now()}-${Math.random()}`;
}

function hideForm(): void {
  const form = document.getElementById("create-secret-form");
  if (form) {
    form.classList.add("hidden");
  }
}

function createSuccessHeader(container: HTMLElement): void {
  const title = document.createElement("h3");
  title.textContent = UI_STRINGS.SUCCESS_TITLE;
  container.appendChild(title);

  const instructions = document.createElement("p");
  instructions.className = "share-instructions";
  instructions.textContent = UI_STRINGS.SHARE_INSTRUCTIONS;
  container.appendChild(instructions);
}

function createLabel(
  text: string,
  styles?: Partial<CSSStyleDeclaration>,
): HTMLLabelElement {
  const label = document.createElement("label");
  label.textContent = text;
  label.style.fontWeight = "bold";
  label.style.marginBottom = "0.5rem";
  label.style.display = "block";

  if (styles) {
    Object.assign(label.style, styles);
  }

  return label;
}

function createUrlInput(
  id: string,
  value: string,
  ariaLabel: string,
): HTMLInputElement {
  const input = document.createElement("input");
  input.type = "text";
  input.id = id;
  input.readOnly = true;
  input.setAttribute("aria-label", ariaLabel);
  input.value = value;
  input.className = "url-display";
  input.addEventListener("click", () => input.select());
  return input;
}

function createCopyButton(
  text: string,
  ariaLabel: string,
  clickHandler: () => void,
): HTMLButtonElement {
  return createButton("secondary-button", text, ariaLabel, clickHandler);
}

function splitSecretUrl(secretUrl: string) {
  const url = new URL(secretUrl);
  const key = url.hash.substring(1);
  url.hash = "";
  const urlWithoutFragment = url.toString();
  return { urlWithoutFragment, key };
}

function createSeparateUrlDisplay(
  secretUrl: string,
  urlId: string,
): HTMLElement {
  const container = document.createElement("div");
  container.className = "secret-container";

  const { urlWithoutFragment, key } = splitSecretUrl(secretUrl);

  const urlLabel = createLabel(window.i18n.t("label.url"));
  container.appendChild(urlLabel);

  const urlDisplay = createUrlInput(urlId, urlWithoutFragment, "Secret URL");
  container.appendChild(urlDisplay);

  const keyLabel = createLabel(window.i18n.t("label.key"), {
    marginTop: "1rem",
  });
  container.appendChild(keyLabel);

  const keyDisplay = createUrlInput(urlId + "-key", key, "Decryption Key");
  container.appendChild(keyDisplay);

  const buttonsContainer = createButtonContainer();

  const copyUrlBtn = createCopyButton(
    window.i18n.t("button.copyUrl"),
    "Copy secret URL to clipboard",
    function (this: HTMLButtonElement) {
      copyUrl(urlId, this);
    },
  );
  buttonsContainer.appendChild(copyUrlBtn);

  const copyKeyBtn = createCopyButton(
    window.i18n.t("button.copyKey"),
    "Copy decryption key to clipboard",
    function (this: HTMLButtonElement) {
      copyUrl(urlId + "-key", this);
    },
  );
  buttonsContainer.appendChild(copyKeyBtn);

  const createAnotherBtn = createCopyButton(
    window.i18n.t("button.createAnother"),
    "Create another secret",
    resetToCreateMode,
  );
  buttonsContainer.appendChild(createAnotherBtn);

  container.appendChild(buttonsContainer);
  return container;
}

function createCombinedUrlDisplay(
  secretUrl: string,
  urlId: string,
): HTMLElement {
  const container = document.createElement("div");
  container.className = "secret-container";

  const urlDisplay = createUrlInput(urlId, secretUrl, "Secret URL");
  container.appendChild(urlDisplay);

  const buttonsContainer = createButtonContainer();

  const copyBtn = createCopyButton(
    UI_STRINGS.COPY_TEXT,
    "Copy secret URL to clipboard",
    function (this: HTMLButtonElement) {
      copyUrl(urlId, this);
    },
  );
  buttonsContainer.appendChild(copyBtn);

  const createAnotherBtn = createCopyButton(
    window.i18n.t("button.createAnother"),
    "Create another secret",
    resetToCreateMode,
  );
  buttonsContainer.appendChild(createAnotherBtn);

  container.appendChild(buttonsContainer);
  return container;
}

function isSeparateKeyMode(): boolean {
  const separateKeyCheckbox = document.getElementById(
    "separateKey",
  ) as HTMLInputElement;
  return separateKeyCheckbox?.checked || false;
}

function createUrlDisplaySection(
  secretUrl: string,
  urlId: string,
): HTMLElement {
  return isSeparateKeyMode()
    ? createSeparateUrlDisplay(secretUrl, urlId)
    : createCombinedUrlDisplay(secretUrl, urlId);
}

function createNoteSection(container: HTMLElement): void {
  const note = document.createElement("p");
  note.className = "secret-note";

  const noteText = window.i18n.t("msg.createNote");
  const colonIndex = noteText.indexOf(":");

  if (colonIndex > 0) {
    const strong = document.createElement("strong");
    strong.textContent = noteText.substring(0, colonIndex + 1);
    note.appendChild(strong);
    note.appendChild(
      document.createTextNode(" " + noteText.substring(colonIndex + 1).trim()),
    );
  } else {
    const strong = document.createElement("strong");
    strong.textContent = "Note: ";
    note.appendChild(strong);
    note.appendChild(
      document.createTextNode(window.i18n.t("msg.createNoteText")),
    );
  }

  container.appendChild(note);
}

function showSuccess(secretUrl: string): void {
  const resultContainer = document.getElementById("result");
  if (!resultContainer) {
    console.error("Result container not found");
    return;
  }

  resultContainer.className = "result success";
  const urlId = generateUrlId();
  resultContainer.innerHTML = "";

  hideForm();
  createSuccessHeader(resultContainer);

  const container = createUrlDisplaySection(secretUrl, urlId);
  resultContainer.appendChild(container);

  createNoteSection(resultContainer);
  announceToScreenReader(UI_STRINGS.SUCCESS_TITLE);
}

function resetToCreateMode(): void {
  const form = document.getElementById("create-secret-form");
  const resultContainer = document.getElementById("result");

  if (form) {
    form.classList.remove("hidden");
  }
  if (resultContainer) {
    resultContainer.innerHTML = "";
    resultContainer.className = "";
  }

  focusTextMode();
}

function focusTextMode(): void {
  const textRadio = document.getElementById("textRadio") as HTMLInputElement;
  const secretText = document.getElementById("secretText") as HTMLInputElement;
  if (textRadio && secretText) {
    textRadio.checked = true;
    toggleSecretType();
    secretText.focus();
  }
}

function showError(message: string): void {
  const resultDiv = document.getElementById("result");
  if (!resultDiv) return;

  resultDiv.className = "result error";
  resultDiv.innerHTML = "";

  showForm();

  const title = document.createElement("h3");
  title.textContent = UI_STRINGS.ERROR_TITLE;
  resultDiv.appendChild(title);

  const errorDiv = document.createElement("div");
  errorDiv.textContent = message;
  resultDiv.appendChild(errorDiv);

  announceToScreenReader(`${UI_STRINGS.ERROR_TITLE}: ${message}`);
}

function showForm(): void {
  const form = document.getElementById("create-secret-form");
  if (form) {
    form.classList.remove("hidden");
  }
}

function copyUrl(urlId: string, button: HTMLButtonElement): void {
  const urlElement = document.getElementById(urlId) as HTMLInputElement;

  if (!urlElement || !button) {
    showError(UI_STRINGS.COPY_FAILED);
    return;
  }

  const originalText = button.textContent || "";
  const urlText = urlElement.value;

  copyToClipboard(
    urlText,
    button,
    originalText,
    UI_STRINGS.COPIED_TEXT,
    UI_STRINGS.COPY_FAILED,
  );
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function getFileElements(): FileElements {
  return {
    fileInput: document.getElementById("secretFile") as HTMLInputElement,
    fileInfoDiv: document.getElementById("fileInfo") as HTMLElement,
    fileNameSpan: document.getElementById("fileName") as HTMLElement,
    fileSizeSpan: document.getElementById("fileSize") as HTMLElement,
    radioGroup: document.querySelector(
      ".input-group:first-child",
    ) as HTMLElement,
    textInputGroup: document.getElementById("textInputGroup") as HTMLElement,
    fileInputGroup: document.getElementById("fileInputGroup") as HTMLElement,
    fileRadio: document.getElementById("fileRadio") as HTMLInputElement,
    textRadio: document.getElementById("textRadio") as HTMLInputElement,
  };
}

function showFileInfo(file: File, elements: FileElements): void {
  const { fileInfoDiv, fileNameSpan, fileSizeSpan } = elements;
  const sanitizedName = sanitizeFileName(file.name);

  fileNameSpan.textContent = sanitizedName || "Invalid filename";
  fileSizeSpan.textContent = formatFileSize(file.size);
  fileInfoDiv.classList.remove("hidden");
  fileInfoDiv.className = "file-info";
}

function hideFileInfo(elements: FileElements): void {
  const { fileInfoDiv } = elements;
  fileInfoDiv.classList.add("hidden");
}

function switchToFileMode(elements: FileElements): void {
  const { radioGroup, textInputGroup, fileInputGroup, fileRadio } = elements;

  radioGroup.classList.add("hidden");
  textInputGroup.classList.add("hidden");
  fileInputGroup.classList.remove("hidden");
  fileRadio.checked = true;
}

function switchToTextMode(elements: FileElements): void {
  const { radioGroup, textRadio } = elements;

  radioGroup.classList.remove("hidden");
  textRadio.checked = true;
  toggleSecretType();
}

function updateFileInfo(): void {
  const elements = getFileElements();
  const { fileInput } = elements;

  if (fileInput.files?.length && fileInput.files.length > 0) {
    const file = fileInput.files[0];
    showFileInfo(file, elements);
    switchToFileMode(elements);
  } else {
    hideFileInfo(elements);
    switchToTextMode(elements);
  }
}

function setupTextMode(): void {
  const textInputGroup = document.getElementById("textInputGroup");
  const fileInputGroup = document.getElementById("fileInputGroup");
  const secretText = document.getElementById("secretText") as HTMLInputElement;
  const secretFile = document.getElementById("secretFile") as HTMLInputElement;

  if (textInputGroup && fileInputGroup && secretText && secretFile) {
    textInputGroup.classList.remove("hidden");
    fileInputGroup.classList.add("hidden");
    secretText.required = true;
    secretFile.required = false;
    secretText.focus();
  }
}

function setupFileMode(): void {
  const textInputGroup = document.getElementById("textInputGroup");
  const fileInputGroup = document.getElementById("fileInputGroup");
  const secretText = document.getElementById("secretText") as HTMLInputElement;
  const secretFile = document.getElementById("secretFile") as HTMLInputElement;

  if (textInputGroup && fileInputGroup && secretText && secretFile) {
    textInputGroup.classList.add("hidden");
    fileInputGroup.classList.remove("hidden");
    secretText.required = false;
    secretFile.required = true;
    secretFile.focus();
  }
}

function toggleSecretType(): void {
  const textRadio = document.getElementById("textRadio") as HTMLInputElement;

  if (textRadio?.checked) {
    setupTextMode();
  } else {
    setupFileMode();
  }
}

function setupFormHandler(): void {
  const form = document.querySelector("form");
  if (form) {
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      createSecret();
    });
  }
}

function setupRadioHandlers(): void {
  const textRadio = document.getElementById("textRadio") as HTMLInputElement;
  const fileRadio = document.getElementById("fileRadio") as HTMLInputElement;

  if (textRadio && fileRadio) {
    textRadio.addEventListener("change", toggleSecretType);
    fileRadio.addEventListener("change", toggleSecretType);
    toggleSecretType();
  }
}

function setupFileInputHandler(): void {
  const fileInput = document.getElementById("secretFile") as HTMLInputElement;
  if (fileInput) {
    fileInput.addEventListener("change", updateFileInfo);
  }
}

function initializeAuthToken(): void {
  const savedToken = getAuthTokenFromStorage();
  if (savedToken) {
    const authTokenInput = document.getElementById(
      "authToken",
    ) as HTMLInputElement;
    const saveTokenCheckbox = document.getElementById(
      "saveTokenCookie",
    ) as HTMLInputElement;

    if (authTokenInput) {
      authTokenInput.value = savedToken;
    }

    // Check the checkbox since we have a saved token
    if (saveTokenCheckbox) {
      saveTokenCheckbox.checked = true;
    }
  }
}

function handleAuthTokenSave(token: string, shouldSave: boolean): void {
  if (shouldSave && token.trim()) {
    const saved = saveAuthTokenToStorage(token);
    if (!saved) {
      console.warn("Failed to save auth token to sessionStorage");
    }
  } else if (!shouldSave) {
    clearAuthTokenStorage();
  }
}

function focusSecretInput(): void {
  const secretText = document.getElementById("secretText") as HTMLInputElement;
  if (secretText) {
    secretText.focus();
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
  focusSecretInput();
  setupFormHandler();
  setupRadioHandlers();
  setupFileInputHandler();
  initializeAuthToken();
});

// Export functions for testing
export {
  sanitizeFileName,
  formatFileSize,
  createSecret,
  showError,
  showSuccess,
  updateFileInfo,
  toggleSecretType,
  resetToCreateMode,
  updateUIStrings,
  initializeAuthToken,
  handleAuthTokenSave,
  UI_STRINGS,
};
