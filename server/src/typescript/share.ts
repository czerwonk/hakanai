/**
 * Share page functionality for clipboard-based secret sharing
 * Reads JSON data from clipboard and creates secrets
 */

import { HakanaiClient } from "./hakanai-client";
import { initI18n, I18nKeys } from "./core/i18n";
import { formatFileSize, formatTTL, sanitizeFileName } from "./core/formatters";
import { hideElement, showElement } from "./core/dom-utils";
import { displaySuccessResult } from "./components/success-display";
import { ShareData, ShareDataError } from "./core/types";
import { initFeatures } from "./core/app-config";

let sharePayload: ShareData | null = null;

const DEFAULT_TTL = 86400; // Default TTL in seconds (1 day)

/**
 * Show loading state
 */
function showLoading(message: string): void {
  document.getElementById("loading-text")!.textContent = message;
  const loading = document.getElementById("loading")!;
  showElement(loading);
  hideOtherSections("loading");
}

/**
 * Hide loading state
 */
function hideLoading(): void {
  const loading = document.getElementById("loading")!;
  hideElement(loading);
}

/**
 * Hide all sections except the specified one
 */
function hideOtherSections(except: string): void {
  const sections = [
    "clipboard-content",
    "clipboard-error",
    "permission-prompt",
    "loading",
    "result-success",
  ];
  sections.forEach((section) => {
    if (section !== except) {
      const element = document.getElementById(section)!;
      hideElement(element);
    }
  });
}

/**
 * Show share content preview
 */
function showShareContent(payload: ShareData): void {
  sharePayload = payload;

  // Update UI
  document.getElementById("content-size")!.textContent = formatFileSize(
    payload.getContentSize(),
  );
  document.getElementById("content-ttl")!.textContent = formatTTL(
    payload.ttl ?? DEFAULT_TTL,
  );

  // Show filename if present (sanitized for security)
  const filenameRow = document.getElementById("filename-row")!;
  if (payload.filename) {
    const sanitizedFilename = sanitizeFileName(payload.filename);
    document.getElementById("content-filename")!.textContent =
      sanitizedFilename ?? "Invalid filename";
    showElement(filenameRow);
  } else {
    hideElement(filenameRow);
  }

  const clipboardContent = document.getElementById("clipboard-content")!;
  showElement(clipboardContent);
  hideOtherSections("clipboard-content");
}

/**
 * Show clipboard error
 */
function showClipboardError(message: string): void {
  document.getElementById("error-message")!.textContent = message;
  const clipboardError = document.getElementById("clipboard-error")!;
  showElement(clipboardError);
  hideOtherSections("clipboard-error");
}

/**
 * Show success result
 */
function showSuccess(url: string): void {
  const resultContainer = document.getElementById("result-success");
  if (!resultContainer) {
    console.error("Result container not found");
    return;
  }

  const separateKeyCheckbox = document.getElementById(
    "separate-key-mode",
  ) as HTMLInputElement;
  const separateKeyMode = separateKeyCheckbox?.checked ?? false;

  displaySuccessResult(url, {
    container: resultContainer,
    separateKeyMode: separateKeyMode,
  });
  hideOtherSections("result-success");
}

/**
 * Handle share data reading errors
 */
function handleShareError(error: unknown, context: string): void {
  hideLoading();

  if (error instanceof Error && error.name === "NotAllowedError") {
    showClipboardError(window.i18n.t(I18nKeys.Msg.ClipboardPermissionDenied));
  } else if (error instanceof ShareDataError) {
    // Handle validation errors with translations
    const translationKey = `validation.${error.code}`;
    const translatedMessage = window.i18n.t(translationKey);

    showClipboardError(
      translatedMessage || error.message, // Fallback to original message
    );
  } else if (
    error instanceof Error &&
    error.message === "Invalid JSON format"
  ) {
    showClipboardError(window.i18n.t(I18nKeys.Msg.ClipboardInvalidJson));
  } else {
    showClipboardError(
      `Error ${context}: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}

/**
 * Read clipboard synchronously for iOS compatibility
 * Must be called directly from button click handler
 */
function readClipboard(): void {
  showLoading(window.i18n.t(I18nKeys.Msg.ReadingClipboard));

  navigator.clipboard
    .readText()
    .then((clipboardText) => {
      try {
        const payload = ShareData.fromJSON(clipboardText);
        hideLoading();
        showShareContent(payload);
      } catch (error) {
        handleShareError(error, "parsing clipboard data");
      }
    })
    .catch((error) => {
      handleShareError(error, "reading clipboard");
    });
}

/**
 * Create and send the secret
 */
async function createSecret(): Promise<void> {
  if (!sharePayload) {
    showClipboardError("No share data available");
    return;
  }

  try {
    showLoading(window.i18n.t(I18nKeys.Msg.CreatingSecret));

    const client = new HakanaiClient(window.location.origin);

    // Sanitize filename before creating payload
    const sanitizedFilename = sharePayload.filename
      ? sanitizeFileName(sharePayload.filename)
      : undefined;
    const hakanaiPayload = client.createPayload(sanitizedFilename || undefined);
    hakanaiPayload.setFromBase64(sharePayload.data);

    const url = await client.sendPayload(
      hakanaiPayload,
      sharePayload.ttl || 86400,
      sharePayload.token,
    );

    try {
      await navigator.clipboard.writeText(url);
    } catch (e) {
      console.warn("Could not copy URL to clipboard:", e);
    }

    hideLoading();
    showSuccess(url);
  } catch (error) {
    hideLoading();
    showClipboardError(
      `Failed to create secret: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}

function initShareData() {
  const fragment = window.location.hash.substring(1);
  if (fragment) {
    try {
      const fragmentData = ShareData.fromFragment(fragment);
      if (fragmentData) {
        showShareContent(fragmentData);
        return;
      }
    } catch (error) {
      showClipboardError(
        `Invalid share data: ${error instanceof Error ? error.message : "Unknown error"}`,
      );
      return;
    }
  }

  // Show permission prompt for clipboard access
  const permissionPrompt = document.getElementById("permission-prompt")!;
  showElement(permissionPrompt);
  hideOtherSections("permission-prompt");
}

document.addEventListener("DOMContentLoaded", () => {
  initI18n();
  initFeatures();

  document
    .getElementById("read-clipboard")
    ?.addEventListener("click", readClipboard);
  document
    .getElementById("share-button")
    ?.addEventListener("click", createSecret);

  initShareData();
});
