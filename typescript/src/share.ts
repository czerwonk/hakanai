// SPDX-License-Identifier: Apache-2.0

/**
 * Share page functionality for clipboard-based secret sharing
 * Reads JSON data from clipboard and creates secrets
 */

import { HakanaiClient } from "./hakanai-client";
import { initI18n, I18nKeys } from "./core/i18n";
import { formatFileSize, sanitizeFileName } from "./core/formatters";
import { hideElement, showElement } from "./core/dom-utils";
import { KeyboardShortcuts } from "./core/keyboard-shortcuts";
import { displaySuccessResult } from "./components/create-result";
import { displayErrorMessage } from "./components/error-display";
import { ShareData, ShareDataError } from "./core/share-data";
import { ErrorHandler, handleAPIError } from "./core/error";
import { initFeatures } from "./core/app-config";
import { TTLSelector } from "./components/ttl-selector";
import { ProgressBar } from "./components/progress-bar";
import { RestrictionData, toSecretRestrictions } from "./core/restriction-data";
import {
  registerServiceWorker,
  SHARE_CACHE_NAME,
  SHARE_DATA_KEY,
  type ShareTargetData,
  type ShareTargetFile,
} from "./service-worker";

const DEFAULT_TTL = 3600; // Default TTL in seconds (1 hour)

let sharePayload: ShareData | null = null;
let ttlSelector: TTLSelector | null = null;

function showLoading(message: string): void {
  const loadingDiv = document.getElementById("loading");
  if (loadingDiv) {
    const loadingText = loadingDiv.querySelector(".loading-text");
    if (loadingText) {
      loadingText.textContent = message;
    }
    showElement(loadingDiv);
    hideOtherSections("loading");
  }
}

function hideLoading(): void {
  const loadingDiv = document.getElementById("loading");
  if (loadingDiv) {
    hideElement(loadingDiv);
  }
}

/**
 * Hide all sections except the specified one
 */
function hideOtherSections(except: string): void {
  const sections = ["clipboard-content", "ttl-selector", "result", "permission-prompt", "loading", "result-success"];
  sections.forEach((section) => {
    if (section !== except) {
      const element = document.getElementById(section)!;
      hideElement(element);
    }
  });
}

function showTTLSelect(payload: ShareData): void {
  const ttlSelectDiv = document.getElementById("ttl-selector")!;
  showElement(ttlSelectDiv);

  const ttl = payload.ttl || DEFAULT_TTL;
  ttlSelector?.setValue(ttl);
}

/**
 * Show share content preview
 */
function showShareContent(payload: ShareData): void {
  sharePayload = payload;

  // Update UI
  document.getElementById("content-size")!.textContent = formatFileSize(payload.getContentSize());

  // Show filename if present (sanitized for security)
  const filenameRow = document.getElementById("filename-row")!;
  if (payload.filename) {
    const sanitizedFilename = sanitizeFileName(payload.filename);
    document.getElementById("content-filename")!.textContent = sanitizedFilename ?? "Invalid filename";
    showElement(filenameRow);
  } else {
    hideElement(filenameRow);
  }

  const clipboardContent = document.getElementById("clipboard-content")!;
  showElement(clipboardContent);
  hideOtherSections("clipboard-content");
  showTTLSelect(payload);
}

/**
 * Show clipboard error
 */
function showError(message: string): void {
  const result = document.getElementById("result")!;
  showElement(result);
  hideOtherSections("result");

  displayErrorMessage(message, result);
}

/**
 * Show success result
 */
function showSuccess(url: string, restrictions?: RestrictionData): void {
  const resultContainer = document.getElementById("result-success");
  if (!resultContainer) {
    console.error("Result container not found");
    return;
  }

  displaySuccessResult(url, {
    container: resultContainer,
    restrictionData: restrictions,
  });
  hideOtherSections("result-success");
}

// Error handler implementation for share page
class ShareErrorHandler implements ErrorHandler {
  displayError(message: string): void {
    showError(message);
  }
}

// Create a singleton instance
const errorHandler = new ShareErrorHandler();

/**
 * Handle share data reading errors
 */
function handleShareError(error: unknown, context: string): void {
  hideLoading();

  // Handle share-specific errors first
  if (error instanceof Error && error.name === "NotAllowedError") {
    showError(window.i18n.t(I18nKeys.Msg.ClipboardPermissionDenied));
  } else if (error instanceof ShareDataError) {
    // Handle validation errors with translations
    const translationKey = `validation.${error.code}`;
    const translatedMessage = window.i18n.t(translationKey);
    showError(translatedMessage || error.message);
  } else if (error instanceof Error && error.message === "Invalid JSON format") {
    showError(window.i18n.t(I18nKeys.Msg.ClipboardInvalidJson));
  } else {
    // For all other errors, use the generic handler
    const fallbackMessage = context
      ? `Error ${context}: ${error instanceof Error ? error.message : "Unknown error"}`
      : "Unknown error occurred";
    handleAPIError(error, fallbackMessage, errorHandler);
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
    showError("No share data available");
    return;
  }

  const progressBar = new ProgressBar();
  progressBar.show(window.i18n.t(I18nKeys.Msg.CreatingSecret));
  hideOtherSections("loading");

  try {
    const client = new HakanaiClient(window.location.origin);

    const sanitizedFilename = sharePayload.filename ? sanitizeFileName(sharePayload.filename) : undefined;
    const hakanaiPayload = client.createPayload(sanitizedFilename || undefined, sharePayload.data_type);
    hakanaiPayload.setFromBase64(sharePayload.data);

    if (!ttlSelector) {
      throw new Error("TTL selector not initialized");
    }
    const ttl = ttlSelector.getValue();

    // Convert RestrictionData to SecretRestrictions if present
    const restrictions = sharePayload.restrictions ? await toSecretRestrictions(sharePayload.restrictions) : undefined;

    const url = await client.sendPayload(hakanaiPayload, ttl, sharePayload.token, progressBar, restrictions);

    try {
      await navigator.clipboard.writeText(url);
    } catch (e) {
      console.warn("Could not copy URL to clipboard:", e);
    }

    progressBar.hide();
    showSuccess(url, sharePayload.restrictions);
  } catch (error) {
    progressBar.hide();
    showError(`Failed to create secret: ${error instanceof Error ? error.message : "Unknown error"}`);
  }
}

function showPermissionPrompt() {
  const permissionPrompt = document.getElementById("permission-prompt")!;
  showElement(permissionPrompt);
  hideOtherSections("permission-prompt");
}

function initTTLSelector(): void {
  const ttlContainer = document.getElementById("ttl-selector") as HTMLElement;
  if (!ttlContainer) {
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

/**
 * Convert Share Target data to ShareData format
 */
function convertShareTargetToShareData(data: ShareTargetData): ShareData | null {
  if (data.file && data.file.data) {
    const file = data.file as ShareTargetFile;
    return new ShareData(file.data, file.name);
  }

  const content = data.text || data.url || data.title || "";
  if (content) {
    const data = btoa(encodeURIComponent(content));
    return new ShareData(data);
  }

  return null;
}

/**
 * Check for and process Share Target data from service worker cache
 */
async function checkForShareTargetData(): Promise<boolean> {
  // Check if we have data from Share Target (via service worker)
  if (!("caches" in window)) {
    return false;
  }

  try {
    const cache = await caches.open(SHARE_CACHE_NAME);
    const response = await cache.match(SHARE_DATA_KEY);

    if (!response) {
      return false;
    }

    const data = (await response.json()) as ShareTargetData;

    // Clear the cache immediately after reading
    await cache.delete("/share-target-data");

    // Also tell service worker to clear (belt and suspenders)
    if (navigator.serviceWorker?.controller) {
      navigator.serviceWorker.controller.postMessage({ type: "CLEAR_SHARE_CACHE" });
    }

    const shareData = convertShareTargetToShareData(data);

    if (shareData) {
      showShareContent(shareData);
      return true;
    }
  } catch (error) {
    console.error("Error reading share target data:", error);
  }

  return false;
}

document.addEventListener("DOMContentLoaded", async () => {
  initI18n();
  initTTLSelector();
  initFeatures();
  initKeyboardShortcuts();

  document.getElementById("read-clipboard")?.addEventListener("click", readClipboard);
  document.getElementById("share-button")?.addEventListener("click", createSecret);

  await registerServiceWorker();

  const hasShareData = await checkForShareTargetData();
  if (!hasShareData) {
    showPermissionPrompt();
  }
});
