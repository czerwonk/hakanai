/**
 * Share page functionality for clipboard-based secret sharing
 * Reads JSON data from clipboard and creates secrets
 */

import { HakanaiClient } from "./hakanai-client.js";
import { formatFileSize, formatTTL } from "./common-utils.js";

// Declare window.i18n
declare global {
  interface Window {
    i18n: {
      t(key: string): string;
    };
  }
}

interface ClipboardData {
  data: string; // base64-encoded content
  filename?: string;
  token?: string;
  ttl?: number;
}

let clipboardPayload: ClipboardData | null = null;

/**
 * Show loading state
 */
function showLoading(message: string): void {
  document.getElementById("loading-text")!.textContent = message;
  document.getElementById("loading")!.style.display = "block";
  hideOtherSections("loading");
}

/**
 * Hide loading state
 */
function hideLoading(): void {
  document.getElementById("loading")!.style.display = "none";
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
      document.getElementById(section)!.style.display = "none";
    }
  });
}

/**
 * Show clipboard content preview
 */
function showClipboardContent(payload: ClipboardData): void {
  clipboardPayload = payload;

  // Calculate content size from base64
  const contentBytes = Math.ceil((payload.data.length * 3) / 4);

  // Update UI
  document.getElementById("content-size")!.textContent =
    formatFileSize(contentBytes);
  document.getElementById("content-ttl")!.textContent = formatTTL(
    payload.ttl || 86400,
  );

  // Show filename if present
  const filenameRow = document.getElementById("filename-row")!;
  if (payload.filename) {
    document.getElementById("content-filename")!.textContent = payload.filename;
    filenameRow.style.display = "block";
  } else {
    filenameRow.style.display = "none";
  }

  document.getElementById("clipboard-content")!.style.display = "block";
  hideOtherSections("clipboard-content");
}

/**
 * Show clipboard error
 */
function showClipboardError(message: string): void {
  document.getElementById("error-message")!.textContent = message;
  document.getElementById("clipboard-error")!.style.display = "block";
  hideOtherSections("clipboard-error");
}

/**
 * Show success result
 */
function showSuccess(url: string): void {
  document.getElementById("secret-url")!.textContent = url;
  document.getElementById("result-success")!.style.display = "block";
  hideOtherSections("result-success");
}

/**
 * Validate clipboard JSON payload
 * @throws Error if validation fails
 */
function validateClipboardPayload(payload: any): void {
  // Validate required fields
  if (!payload.data || typeof payload.data !== "string") {
    throw new Error('Missing or invalid "data" field in clipboard JSON');
  }

  // Validate optional fields
  if (payload.filename !== undefined && typeof payload.filename !== "string") {
    throw new Error('Invalid "filename" field - must be string');
  }

  if (payload.token !== undefined && typeof payload.token !== "string") {
    throw new Error('Invalid "token" field - must be string');
  }

  if (
    payload.ttl !== undefined &&
    (typeof payload.ttl !== "number" || payload.ttl <= 0)
  ) {
    throw new Error('Invalid "ttl" field - must be positive number');
  }
}

/**
 * Read clipboard content and validate it
 * @returns Parsed clipboard data
 * @throws Error if clipboard is empty or invalid JSON
 */
async function readClipboardContent(): Promise<ClipboardData> {
  const clipboardText = await navigator.clipboard.readText();
  if (!clipboardText.trim()) {
    throw new Error("Clipboard is empty");
  }

  let payload = JSON.parse(clipboardText);
  validateClipboardPayload(payload);

  return payload;
}

/**
 * Read and parse clipboard content
 */
async function readClipboard(): Promise<void> {
  try {
    showLoading(
      window.i18n?.t("msg.readingClipboard") || "Reading clipboard...",
    );
    const payload = await readClipboardContent();

    hideLoading();
    showClipboardContent(payload);
  } catch (error) {
    hideLoading();

    if (error instanceof Error && error.name === "NotAllowedError") {
      showClipboardError(
        window.i18n?.t("msg.clipboardPermissionDenied") ||
          "Clipboard access denied. Please grant permission and try again.",
      );
    } else if (error instanceof SyntaxError) {
      showClipboardError(
        window.i18n?.t("msg.clipboardInvalidJson") ||
          "Clipboard does not contain valid JSON",
      );
    } else {
      showClipboardError(
        `Error reading clipboard: ${error instanceof Error ? error.message : "Unknown error"}`,
      );
    }
  }
}

/**
 * Create and send the secret
 */
async function createSecret(): Promise<void> {
  if (!clipboardPayload) {
    showClipboardError("No clipboard data available");
    return;
  }

  try {
    showLoading(window.i18n?.t("msg.creatingSecret") || "Creating secret...");

    const client = new HakanaiClient(window.location.origin);

    const hakanaiPayload = client.createPayload(clipboardPayload.filename);
    hakanaiPayload.setFromBase64(clipboardPayload.data);

    const url = await client.sendPayload(
      hakanaiPayload,
      clipboardPayload.ttl || 86400,
      clipboardPayload.token,
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
 * Copy URL to clipboard
 */
async function copyUrl(): Promise<void> {
  const urlElement = document.getElementById("secret-url");
  const copyButton = document.getElementById("copy-url") as HTMLButtonElement;

  if (!urlElement || !copyButton) return;

  try {
    await navigator.clipboard.writeText(urlElement.textContent || "");

    // Show feedback
    const originalText = copyButton.textContent;
    copyButton.textContent = "Copied!";
    copyButton.classList.add("copied");

    setTimeout(() => {
      copyButton.textContent = originalText;
      copyButton.classList.remove("copied");
    }, 2000);
  } catch (error) {
    console.error("Failed to copy:", error);
    copyButton.textContent = "Copy Failed";
    copyButton.classList.add("copy-failed");

    setTimeout(() => {
      copyButton.textContent = "Copy URL";
      copyButton.classList.remove("copy-failed");
    }, 2000);
  }
}

/**
 * Initialize the page
 */
function init(): void {
  // Check autoInit flag at runtime
  if (!autoInit) {
    return;
  }

  // Show permission prompt initially
  document.getElementById("permission-prompt")!.style.display = "block";
  hideOtherSections("permission-prompt");

  // Add event listeners
  document
    .getElementById("read-clipboard")
    ?.addEventListener("click", readClipboard);
  document
    .getElementById("share-button")
    ?.addEventListener("click", createSecret);
  document.getElementById("copy-url")?.addEventListener("click", copyUrl);

  // Auto-read if auto parameter is present
  const urlParams = new URLSearchParams(window.location.search);
  if (urlParams.get("auto") === "true") {
    // Small delay to let the page render
    setTimeout(readClipboard, 100);
  }
}

// Auto-initialization flag (can be disabled for testing)
let autoInit = true;

// Export for testing (must be before auto-init to allow test setup)
(globalThis as any).sharePageExports = {
  validateClipboardPayload,
  setAutoInit: (value: boolean) => {
    autoInit = value;
  },
  init,
};

// Initialize when DOM is ready (but not in test environment)
if (typeof document !== "undefined" && typeof global === "undefined") {
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
}
