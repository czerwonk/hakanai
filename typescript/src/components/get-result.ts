// SPDX-License-Identifier: Apache-2.0

import { ContentAnalysis, type PayloadData, PayloadDataType } from "../hakanai-client";
import { I18nKeys } from "../core/i18n";
import {
  announceToScreenReader,
  createButton,
  createButtonContainer,
  generateRandomId,
  hideElement,
  expandView,
} from "../core/dom-utils";
import { copyToClipboard } from "../core/clipboard";
import { formatFileSize } from "../core/formatters";

const TIMEOUTS = {
  CLEANUP_DELAY: 100,
} as const;

export function showSecret(payload: PayloadData, resultDiv: HTMLElement, resetCallback: () => void): void {
  if (!payload || !resultDiv) return;

  resultDiv.className = "result success";

  const title = document.createElement("h3");
  title.textContent = window.i18n.t(I18nKeys.Msg.SuccessTitle);
  resultDiv.appendChild(title);

  const decodedBytes = payload.decodeBytes();
  const isBinaryFile = payload.filename != null || ContentAnalysis.isBinary(decodedBytes);

  const container = isBinaryFile
    ? createBinarySecret(payload, decodedBytes.buffer as ArrayBuffer)
    : createTextSecret(payload, decodedBytes.buffer as ArrayBuffer);
  resultDiv.appendChild(container);

  if (payload.filename) {
    resultDiv.appendChild(createFilenameInfo(payload.filename, decodedBytes.length));
  }

  const noteElement = createNoteElement();
  addResetElement(noteElement, resetCallback);
  resultDiv.appendChild(noteElement);

  announceToScreenReader(window.i18n.t(I18nKeys.Msg.SuccessTitle));
}

function createTextSecret(payload: PayloadData, decodedBytes: ArrayBuffer): HTMLElement {
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

function createSecretTextarea(secretId: string, decodedBytes: ArrayBuffer): HTMLTextAreaElement {
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

function createBinarySecret(payload: PayloadData, decodedBytes: ArrayBuffer): HTMLElement {
  const container = document.createElement("div");
  container.className = "secret-container";

  const message = document.createElement("p");
  message.className = "binary-message";
  message.textContent = window.i18n.t(I18nKeys.Msg.BinaryDetected);
  container.appendChild(message);

  const buttonsContainer = createButtonContainer();

  buttonsContainer.appendChild(createDownloadButton(payload, decodedBytes, true));

  if (payload.data_type === PayloadDataType.Image) {
    buttonsContainer.appendChild(createPreviewButton(payload, decodedBytes));
  }

  container.appendChild(buttonsContainer);

  return container;
}

function createPreviewButton(payload: PayloadData, decodedBytes: ArrayBuffer): HTMLButtonElement {
  return createButton(
    "btn preview-btn",
    window.i18n.t(I18nKeys.Button.Preview) || "Preview",
    window.i18n.t(I18nKeys.Aria.PreviewContent) || "Preview content",
    () => showImagePreview(payload, decodedBytes),
  );
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
  decodedBytes: ArrayBuffer,
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

  return container;
}

function addResetElement(container: HTMLElement, resetCallback: () => void) {
  if (!container) return;

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
    () => resetCallback(),
  );
  buttonContainer.appendChild(retrieveAnotherButton);
  container.appendChild(buttonContainer);
}

function copySecret(secretId: string, button: HTMLButtonElement): void {
  const secretElement = document.getElementById(secretId) as HTMLTextAreaElement;
  if (!secretElement) return;

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

function downloadSecret(payload: PayloadData, decodedBytes: ArrayBuffer, isBinary: boolean): void {
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

function showImagePreview(payload: PayloadData, decodedBytes: ArrayBuffer): void {
  const overlay = document.createElement("div");
  overlay.className = "image-preview-overlay";
  overlay.setAttribute("aria-modal", "true");
  overlay.setAttribute("role", "dialog");
  overlay.setAttribute("aria-label", window.i18n.t(I18nKeys.Aria.PreviewContent) || "Content preview");

  const modal = document.createElement("div");
  modal.className = "image-preview-modal";

  const img = document.createElement("img");
  const blob = new Blob([decodedBytes], { type: "image/*" });
  const url = URL.createObjectURL(blob);
  img.src = url;
  img.alt = payload.filename || "Secret image";
  img.className = "preview-image";

  const buttonsContainer = createButtonContainer();
  buttonsContainer.appendChild(createDownloadButton(payload, decodedBytes, true));

  const closeButton = createButton(
    "btn close-btn",
    window.i18n.t(I18nKeys.Button.Close) || "Close",
    window.i18n.t(I18nKeys.Aria.ClosePreview) || "Close preview",
    () => {
      URL.revokeObjectURL(url);
      overlay.remove();
    },
  );
  buttonsContainer.appendChild(closeButton);

  // Assemble modal
  modal.appendChild(img);
  modal.appendChild(buttonsContainer);
  overlay.appendChild(modal);

  // Close on overlay click
  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) {
      URL.revokeObjectURL(url);
      overlay.remove();
    }
  });

  // Close on Escape key
  document.addEventListener("keydown", function escapeHandler(e) {
    if (e.key === "Escape") {
      URL.revokeObjectURL(url);
      overlay.remove();
      document.removeEventListener("keydown", escapeHandler);
    }
  });

  document.body.appendChild(overlay);
}

// Export functions for testing
export { generateFilename };
