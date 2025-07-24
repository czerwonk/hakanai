/**
 * Share page functionality for clipboard-based secret sharing
 * Reads JSON data from clipboard and creates secrets
 */

import { HakanaiClient } from "./hakanai-client.js";
import { initI18n } from "./core/i18n.js";
import {
  formatFileSize,
  formatTTL,
  sanitizeFileName,
} from "./core/formatters.js";
import { hideElement, showElement } from "./core/dom-utils";
import { displaySuccessResult } from "./components/success-display.js";
import { ShareData, ShareDataError } from "./core/types.js";
import { initFeatures } from "./core/app-config";

let sharePayload: ShareData | null = null;

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
    payload.ttl ?? 86400,
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

  displaySuccessResult(url, {
    container: resultContainer,
    separateKeyMode: false,
  });
  hideOtherSections("result-success");
}

/**
 * Read share data from clipboard or URL fragment
 * @returns Parsed share data
 * @throws Error if no data found or validation fails
 */
async function readShareData(): Promise<ShareData> {
  // First try URL fragment
  const fragment = window.location.hash.substring(1);
  const fragmentData = ShareData.fromFragment(fragment);
  if (fragmentData) {
    return fragmentData;
  }

  // Fall back to clipboard
  const clipboardText = await navigator.clipboard.readText();
  return ShareData.fromJSON(clipboardText);
}

/**
 * Handle share data reading errors
 */
function handleShareError(error: unknown, context: string): void {
  hideLoading();

  if (error instanceof Error && error.name === "NotAllowedError") {
    showClipboardError(
      window.i18n?.t("msg.clipboardPermissionDenied") ||
        "Clipboard access denied. Please grant permission and try again.",
    );
  } else if (error instanceof ShareDataError) {
    // Handle validation errors with translations
    const translationKey = `validation.${error.code}`;
    const translatedMessage = window.i18n?.t(translationKey);

    showClipboardError(
      translatedMessage || error.message, // Fallback to original message
    );
  } else if (
    error instanceof Error &&
    error.message === "Invalid JSON format"
  ) {
    showClipboardError(
      window.i18n?.t("msg.clipboardInvalidJson") ||
        "Invalid JSON format in clipboard or URL",
    );
  } else {
    showClipboardError(
      `Error ${context}: ${error instanceof Error ? error.message : "Unknown error"}`,
    );
  }
}

/**
 * Read and parse share data from fragment or clipboard
 */
async function readShare(): Promise<void> {
  try {
    showLoading(window.i18n?.t("msg.readingClipboard") || "Reading data...");
    const payload = await readShareData();

    hideLoading();
    showShareContent(payload);
  } catch (error) {
    handleShareError(error, "reading data");
  }
}

/**
 * Read clipboard synchronously for iOS compatibility
 * Must be called directly from button click handler
 */
function readClipboard(): void {
  showLoading(window.i18n?.t("msg.readingClipboard") || "Reading data...");

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
    showLoading(window.i18n?.t("msg.creatingSecret") || "Creating secret...");

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

/**
 * Initialize the page
 */
function init(): void {
  initI18n();
  initFeatures();
  // Always add event listeners first
  document
    .getElementById("read-clipboard")
    ?.addEventListener("click", readClipboard);
  document
    .getElementById("share-button")
    ?.addEventListener("click", createSecret);

  // Check for fragment data first
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

  // Auto-read if auto parameter is present (clipboard only)
  const urlParams = new URLSearchParams(window.location.search);
  if (urlParams.get("auto") === "true") {
    // Small delay to let the page render
    setTimeout(readShare, 100);
  }
}

// Initialize when DOM is ready
if (typeof document !== "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
}
