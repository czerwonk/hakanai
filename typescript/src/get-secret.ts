// SPDX-License-Identifier: Apache-2.0

import { HakanaiClient, HakanaiErrorCodes, PayloadData } from "./hakanai-client";
import { initI18n, I18nKeys } from "./core/i18n";
import { KeyboardShortcuts } from "./core/keyboard-shortcuts";
import { debounce, hideElement, secureInputClear, showElement, unexpandView } from "./core/dom-utils";
import { displayErrorMessage } from "./components/error-display";
import { ProgressBar } from "./components/progress-bar";
import { initTheme } from "./core/theme";
import { ErrorHandler, handleAPIError, isHakanaiError } from "./core/error";
import { initFeatures } from "./core/app-config";
import { showSecret } from "./components/get-result";

const TIMEOUTS = {
  DEBOUNCE: 300,
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

function showSuccess(payload: PayloadData) {
  clearResult();

  const { resultDiv } = getElements();
  showSecret(payload, resultDiv, resetForm);
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
export { normalizeUrl, hasUrlFragment, validateInputs };
