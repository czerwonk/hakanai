// SPDX-License-Identifier: Apache-2.0

import { HakanaiClient, type PayloadData } from "./hakanai-client";
import { initI18n, I18nKeys } from "./core/i18n";
import { announceToScreenReader, secureInputClear, showElement, hideElement } from "./core/dom-utils";
import { initTheme } from "./core/theme";
import { saveAuthTokenToStorage, getAuthTokenFromStorage, clearAuthTokenStorage } from "./core/auth-storage";
import { formatFileSize, sanitizeFileName } from "./core/formatters";
import { displaySuccessResult } from "./components/create-result";
import { displayErrorMessage } from "./components/error-display";
import { ErrorHandler, handleAPIError } from "./core/error";
import { initFeatures, fetchAppConfig } from "./core/app-config";
import { ProgressBar } from "./components/progress-bar";
import { TTLSelector } from "./components/ttl-selector";
import { KeyboardShortcuts } from "./core/keyboard-shortcuts";
import { FileDropzone } from "./components/file-dropzone";
import { RestrictionsTabs } from "./components/restrictions-tabs";
import { RestrictionData, toSecretRestrictions } from "./core/restriction-data";
import { SizeLimitIndicator } from "./components/size-limit";

let ttlSelector: TTLSelector | null = null;
let fileDropzone: FileDropzone | null = null;
let restrictionsTabs: RestrictionsTabs | null = null;
let sizeLimitIndicator: SizeLimitIndicator | null = null;

interface Elements {
  button: HTMLButtonElement;
  secretInput: HTMLInputElement;
  fileInput: HTMLInputElement;
  authTokenInput: HTMLInputElement;
  textRadio: HTMLInputElement;
  fileRadio: HTMLInputElement;
  resultDiv: HTMLElement;
}

interface FileElements {
  fileInput: HTMLInputElement;
  fileInfoDiv: HTMLElement;
  fileNameSpan: HTMLElement;
  fileSizeSpan: HTMLElement;
  radioGroup: HTMLElement;
  textInputGroup: HTMLElement;
  fileInputGroup: HTMLElement;
  fileRadio: HTMLInputElement;
  textRadio: HTMLInputElement;
}

interface FormValues {
  authToken: string;
  ttl: number;
  isFileMode: boolean;
  restrictionData?: RestrictionData;
}

const baseUrl = window.location.origin.includes("file://") ? "http://localhost:8080" : window.location.origin;

const client = new HakanaiClient(baseUrl);

function getElements(): Elements | null {
  const button = document.getElementById("createBtn") as HTMLButtonElement;
  const secretInput = document.getElementById("secretText") as HTMLInputElement;
  const fileInput = document.getElementById("secretFile") as HTMLInputElement;
  const authTokenInput = document.getElementById("authToken") as HTMLInputElement;
  const textRadio = document.getElementById("textRadio") as HTMLInputElement;
  const fileRadio = document.getElementById("fileRadio") as HTMLInputElement;
  const resultDiv = document.getElementById("result");

  if (!button || !secretInput || !fileInput || !authTokenInput || !textRadio || !fileRadio || !resultDiv) {
    return null;
  }

  return {
    button,
    secretInput,
    fileInput,
    authTokenInput,
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

async function validateAndProcessFileInput(fileInput: HTMLInputElement): Promise<PayloadData | null> {
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
    showError("Your browser doesn't support text encoding. Please use a modern browser.");
    return null;
  }

  const encoder = new TextEncoder();
  const textBytes = encoder.encode(secret);
  const payload = client.createPayload();
  payload.setFromBytes(textBytes);
  return payload;
}

function setElementsState(elements: Elements, disabled: boolean): void {
  const { button, secretInput, fileInput, authTokenInput, textRadio, fileRadio, resultDiv } = elements;

  const fileInputButton = document.getElementById("fileInputButton") as HTMLButtonElement;
  const saveTokenCheckbox = document.getElementById("saveTokenCookie") as HTMLInputElement;

  button.disabled = disabled;
  secretInput.disabled = disabled;
  fileInput.disabled = disabled;
  authTokenInput.disabled = disabled;
  textRadio.disabled = disabled;
  fileRadio.disabled = disabled;

  if (fileInputButton) fileInputButton.disabled = disabled;
  if (saveTokenCheckbox) saveTokenCheckbox.disabled = disabled;

  ttlSelector?.setEnabled(!disabled);
  fileDropzone?.setEnabled(!disabled);
  restrictionsTabs?.setEnabled(!disabled);

  if (disabled) {
    resultDiv.innerHTML = "";
  }
}

function clearInputs(secretInput: HTMLInputElement, fileInput: HTMLInputElement): void {
  secureInputClear(secretInput);
  fileInput.value = "";
  updateFileInfo();
}

function areRestrictionsEnabled(): boolean {
  const restrictAccessCheckbox = document.getElementById("restrictAccess") as HTMLInputElement;
  return restrictAccessCheckbox?.checked ?? false;
}

function getFormValues(elements: Elements): FormValues {
  const restrictionData = areRestrictionsEnabled() ? restrictionsTabs?.getRestrictions() : undefined;

  return {
    authToken: elements.authTokenInput.value.trim(),
    ttl: ttlSelector?.getValue() || 3600,
    isFileMode: elements.fileRadio.checked,
    restrictionData,
  };
}

function showError(message: string): void {
  // Page-specific behavior: show form after error
  const form = document.getElementById("create-secret-form");
  if (form) {
    showElement(form);
  }

  const resultContainer = document.getElementById("result")!;
  displayErrorMessage(message, resultContainer);
}

// Error handler implementation for create-secret page
class CreateSecretErrorHandler implements ErrorHandler {
  displayError(message: string): void {
    showError(message);
  }

  onAuthenticationError(): void {
    const authTokenInput = document.getElementById("authToken") as HTMLInputElement;
    if (authTokenInput) {
      authTokenInput.focus();
      authTokenInput.select();
    }
  }
}

// Create a singleton instance
const errorHandler = new CreateSecretErrorHandler();

function handleCreateError(error: unknown): void {
  handleAPIError(error, window.i18n.t(I18nKeys.Msg.CreateFailed), errorHandler);
}

async function createSecret(): Promise<void> {
  const elements = getElements();
  if (!elements) {
    showError("Page not fully loaded. Please refresh and try again.");
    return;
  }

  if (areRestrictionsEnabled() && !restrictionsTabs?.validateUserInput()) {
    return;
  }

  const { authToken, ttl, isFileMode, restrictionData } = getFormValues(elements);

  const payload = isFileMode
    ? await validateAndProcessFileInput(elements.fileInput)
    : validateTextInput(elements.secretInput);

  if (!payload) {
    return;
  }

  setElementsState(elements, true);

  // Create and show progress bar
  const progressBar = new ProgressBar();
  progressBar.show(window.i18n.t(I18nKeys.Msg.Creating));

  try {
    // Convert restrictionData to API format
    const restrictions = restrictionData ? await toSecretRestrictions(restrictionData) : undefined;

    // Pass the ProgressBar directly as it implements DataTransferObserver
    const secretUrl = await client.sendPayload(payload, ttl, authToken, progressBar, restrictions);

    // Handle auth token cookie saving
    const saveTokenCookie = document.getElementById("saveTokenCookie") as HTMLInputElement;
    if (saveTokenCookie) {
      handleAuthTokenSave(authToken, saveTokenCookie.checked);
    }

    showSuccess(secretUrl, restrictionData);
    clearInputs(elements.secretInput, elements.fileInput);
  } catch (error: unknown) {
    handleCreateError(error);
  } finally {
    progressBar.hide();
    setElementsState(elements, false);
    // Clear auth token from memory for security (unless saving to cookie)
    const saveTokenCookie = document.getElementById("saveTokenCookie") as HTMLInputElement;
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

function showSuccess(secretUrl: string, restrictionData?: RestrictionData): void {
  const resultContainer = document.getElementById("result");
  if (!resultContainer) {
    console.error("Result container not found");
    return;
  }

  hideForm();
  displaySuccessResult(secretUrl, {
    container: resultContainer,
    restrictionData,
  });
  announceToScreenReader(window.i18n.t(I18nKeys.Msg.SuccessTitle));
}

function getFileElements(): FileElements {
  return {
    fileInput: document.getElementById("secretFile") as HTMLInputElement,
    fileInfoDiv: document.getElementById("fileInfo") as HTMLElement,
    fileNameSpan: document.getElementById("fileName") as HTMLElement,
    fileSizeSpan: document.getElementById("fileSize") as HTMLElement,
    radioGroup: document.querySelector(".input-group:first-child") as HTMLElement,
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

async function validateFileSize(file: File): Promise<boolean> {
  const authTokenInput = document.getElementById("authToken") as HTMLInputElement;
  const hasToken = authTokenInput?.value?.trim();

  // for authenticated users we rely on server-side validation, client side validation is just a UX feature
  if (hasToken) {
    return true;
  }

  const config = await fetchAppConfig();
  const limit = config?.secretSizeLimit || 0;

  if (limit > 0 && file.size > limit) {
    const limitInKB = Math.round(limit / 1024);
    const fileSizeInKB = Math.round(file.size / 1024);
    showError(
      window.i18n.t(I18nKeys.Msg.FileSizeExceeded, {
        fileSize: fileSizeInKB,
        limit: limitInKB,
      }),
    );
    return false;
  }

  return true;
}

async function updateFileInfo(): Promise<void> {
  const elements = getFileElements();
  const { fileInput } = elements;

  if (fileInput.files?.length) {
    const file = fileInput.files[0];

    // Validate file size
    const isValid = await validateFileSize(file);
    if (!isValid) {
      fileInput.value = "";
      hideFileInfo(elements);
      return;
    }

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
  const fileInputButton = document.getElementById("fileInputButton") as HTMLButtonElement;
  const dropzoneContainer = document.getElementById("fileDropzone");

  if (!fileInput || !fileInputButton || !dropzoneContainer) {
    return;
  }

  // Always listen for file input changes
  fileInput.addEventListener("change", updateFileInfo);

  if (FileDropzone.isDragAndDropSupported()) {
    fileDropzone = new FileDropzone({
      fileInput: fileInput,
      container: dropzoneContainer,
    });
    hideElement(fileInputButton);
  } else {
    // No drag and drop support - hide dropzone and show the button as fallback
    hideElement(dropzoneContainer);
    showFileInputButton(fileInputButton, fileInput);
  }
}

function showFileInputButton(fileInputButton: HTMLButtonElement, fileInput: HTMLInputElement): void {
  showElement(fileInputButton);
  fileInputButton.addEventListener("click", () => {
    fileInput.click();
  });
}

function initializeAuthToken(): void {
  const savedToken = getAuthTokenFromStorage();
  if (savedToken) {
    const authTokenInput = document.getElementById("authToken") as HTMLInputElement;
    const saveTokenCheckbox = document.getElementById("saveTokenCookie") as HTMLInputElement;

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

function initTTLSelector(): void {
  const ttlContainer = document.getElementById("ttl-selector") as HTMLElement;
  if (!ttlContainer) {
    console.error("TTL container not found!");
    throw new Error("TTL container not found");
  }

  ttlSelector = new TTLSelector(ttlContainer);
}

function initKeyboardShortcuts(): void {
  const shortcuts = new KeyboardShortcuts();

  // Ctrl + Enter to create secret
  shortcuts.register({
    key: "Enter",
    ctrl: true,
    handler: () => {
      createSecret();
    },
    description: "Create secret",
  });
}

async function initTokenInputVisibility(): Promise<void> {
  const tokenInputGroup = document.querySelector(".input-group:has(#authToken)") as HTMLElement;

  if (!tokenInputGroup) {
    return;
  }

  const showTokenInput = await shouldShowTokenInput();
  if (!showTokenInput) {
    hideElement(tokenInputGroup);
    return;
  }

  showElement(tokenInputGroup);

  // add listener to update size limit visibility on token input changes
  const authTokenInput = document.getElementById("authToken") as HTMLInputElement;
  if (authTokenInput) {
    authTokenInput.addEventListener("input", updateSizeLimitVisibility);
  }
}

async function shouldShowTokenInput(): Promise<boolean> {
  const urlParams = new URLSearchParams(window.location.search);
  const showTokenParam = urlParams.get("show_token");

  if (showTokenParam === "1" || showTokenParam === "true") {
    return true;
  }

  const config = await fetchAppConfig();
  return config?.showTokenInput ?? false;
}

async function initSizeLimitIndicator(): Promise<void> {
  try {
    const config = await fetchAppConfig();
    const limit = config?.secretSizeLimit || 0;

    sizeLimitIndicator = new SizeLimitIndicator();
    sizeLimitIndicator.initialize(limit);

    if (limit > 0) {
      sizeLimitIndicator.show();
    }

    const secretInput = document.getElementById("secretText") as HTMLInputElement;
    if (secretInput) {
      secretInput.addEventListener("input", () => {
        if (sizeLimitIndicator) {
          sizeLimitIndicator.update(secretInput.value);
        }
      });
    }
  } catch (error) {
    console.warn("Failed to initialize size limit indicator:", error);
  }
}

function updateSizeLimitVisibility(): void {
  if (!sizeLimitIndicator) return;

  const authTokenInput = document.getElementById("authToken") as HTMLInputElement;
  const hasToken = authTokenInput?.value?.trim();

  if (hasToken) {
    sizeLimitIndicator.hide();
    return;
  }

  sizeLimitIndicator.show();

  const secretInput = document.getElementById("secretText") as HTMLInputElement;
  sizeLimitIndicator.update(secretInput?.value);
}

function initRestrictionsComponent(): void {
  const restrictionsTabsContainer = document.getElementById("restrictionsTabs");
  if (restrictionsTabsContainer) {
    restrictionsTabs = new RestrictionsTabs({
      container: restrictionsTabsContainer,
    });
  }
}

function initRestrictionsCheckbox(): void {
  const restrictAccessCheckbox = document.getElementById("restrictAccess") as HTMLInputElement;
  const restrictionsInputGroup = document.getElementById("restrictionsInputGroup") as HTMLElement;

  if (!restrictAccessCheckbox || !restrictionsInputGroup) {
    return;
  }

  // Initially hide restrictions (checkbox is unchecked by default)
  hideElement(restrictionsInputGroup);

  // Set up event handler for checkbox
  restrictAccessCheckbox.addEventListener("change", () => {
    if (restrictAccessCheckbox.checked) {
      showElement(restrictionsInputGroup);
      if (!restrictionsTabs) {
        initRestrictionsComponent();
      }
      restrictionsTabs?.focusActiveTab();
    } else {
      hideElement(restrictionsInputGroup);
    }
  });
}

document.addEventListener("DOMContentLoaded", async () => {
  initI18n();
  initTTLSelector();
  initTheme();
  focusSecretInput();
  setupFormHandler();
  setupRadioHandlers();
  setupFileInputHandler();
  initializeAuthToken();
  await initFeatures();
  await initTokenInputVisibility();
  await initSizeLimitIndicator();
  initRestrictionsCheckbox();
  initKeyboardShortcuts();
});
