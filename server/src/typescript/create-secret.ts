import {
  HakanaiClient,
  HakanaiErrorCodes,
  type PayloadData,
} from "./hakanai-client";
import { initI18n, I18nKeys } from "./core/i18n";
import {
  announceToScreenReader,
  secureInputClear,
  showElement,
  hideElement,
} from "./core/dom-utils";
import { initTheme } from "./core/theme";
import {
  saveAuthTokenToStorage,
  getAuthTokenFromStorage,
  clearAuthTokenStorage,
} from "./core/auth-storage";
import { formatFileSize, sanitizeFileName } from "./core/formatters";
import { displaySuccessResult } from "./components/success-display";
import {
  type RequiredElements,
  type FileElements,
  type FormValues,
  isHakanaiError,
  isStandardError,
  isErrorLike,
} from "./core/types";
import { initFeatures } from "./core/app-config";

// Use the RequiredElements type from types.ts
type Elements = RequiredElements;

const baseUrl = window.location.origin.includes("file://")
  ? "http://localhost:8080"
  : window.location.origin;

const client = new HakanaiClient(baseUrl);

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
    showError(window.i18n.t(I18nKeys.Msg.EmptyFile));
    fileInput.focus();
    return null;
  }

  const fileName = sanitizeFileName(file.name);
  if (!validateFilename(file.name)) {
    showError(window.i18n.t(I18nKeys.Msg.InvalidFilename));
    fileInput.focus();
    return null;
  }

  try {
    const fileBytes = await readFileAsBytes(file);
    const payload = client.createPayload(fileName!);
    payload.setFromBytes(fileBytes);
    return payload;
  } catch {
    showError(window.i18n.t(I18nKeys.Msg.FileReadError));
    return null;
  }
}

function validateTextInput(secretInput: HTMLInputElement): PayloadData | null {
  const secret = secretInput.value.trim();
  if (!secret) {
    showError(window.i18n.t(I18nKeys.Msg.EmptySecret));
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
  payload.setFromBytes(textBytes);
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
    showElement(loadingDiv);
  } else {
    hideElement(loadingDiv);
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
    const localizedMessage = window.i18n.t(errorKey);
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
    showError(error.message ?? window.i18n.t(I18nKeys.Msg.CreateFailed));
  } else {
    showError(window.i18n.t(I18nKeys.Msg.CreateFailed));
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

function hideForm(): void {
  const form = document.getElementById("create-secret-form");
  if (form) {
    hideElement(form);
  }
}

function isSeparateKeyMode(): boolean {
  const separateKeyCheckbox = document.getElementById(
    "separateKey",
  ) as HTMLInputElement;
  return separateKeyCheckbox?.checked ?? false;
}

function showSuccess(secretUrl: string): void {
  const resultContainer = document.getElementById("result");
  if (!resultContainer) {
    console.error("Result container not found");
    return;
  }

  hideForm();
  displaySuccessResult(secretUrl, {
    container: resultContainer,
    separateKeyMode: isSeparateKeyMode(),
  });
  announceToScreenReader(window.i18n.t(I18nKeys.Msg.SuccessTitle));
}

function showError(message: string): void {
  const resultDiv = document.getElementById("result");
  if (!resultDiv) return;

  resultDiv.className = "result error";
  resultDiv.innerHTML = "";

  showForm();

  const title = document.createElement("h3");
  title.textContent = window.i18n.t(I18nKeys.Msg.ErrorTitle);
  resultDiv.appendChild(title);

  const errorDiv = document.createElement("div");
  errorDiv.textContent = message;
  resultDiv.appendChild(errorDiv);

  announceToScreenReader(
    `${window.i18n.t(I18nKeys.Msg.ErrorTitle)}: ${message}`,
  );
}

function showForm(): void {
  const form = document.getElementById("create-secret-form");
  if (form) {
    showElement(form);
  }
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

  fileNameSpan.textContent = sanitizedName ?? "Invalid filename";
  fileSizeSpan.textContent = `(${formatFileSize(file.size)})`;
  showElement(fileInfoDiv);
  fileInfoDiv.className = "file-info";
}

function hideFileInfo(elements: FileElements): void {
  const { fileInfoDiv } = elements;
  hideElement(fileInfoDiv);
}

function switchToFileMode(elements: FileElements): void {
  const { radioGroup, textInputGroup, fileInputGroup, fileRadio } = elements;

  hideElement(radioGroup);
  hideElement(textInputGroup);
  showElement(fileInputGroup);
  fileRadio.checked = true;
}

function switchToTextMode(elements: FileElements): void {
  const { radioGroup, textRadio } = elements;

  showElement(radioGroup);
  textRadio.checked = true;
  toggleSecretType();
}

function updateFileInfo(): void {
  const elements = getFileElements();
  const { fileInput } = elements;

  if (fileInput.files?.length) {
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
    showElement(textInputGroup);
    hideElement(fileInputGroup);
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
    hideElement(textInputGroup);
    showElement(fileInputGroup);
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
  const fileInputButton = document.getElementById(
    "fileInputButton",
  ) as HTMLButtonElement;

  if (fileInput) {
    fileInput.addEventListener("change", updateFileInfo);
  }

  if (fileInputButton && fileInput) {
    fileInputButton.addEventListener("click", () => {
      fileInput.click();
    });
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

document.addEventListener("DOMContentLoaded", () => {
  initI18n();
  initTheme();
  focusSecretInput();
  setupFormHandler();
  setupRadioHandlers();
  setupFileInputHandler();
  initializeAuthToken();
  initFeatures();
});
