// SPDX-License-Identifier: Apache-2.0

import { ContentAnalysis, type PayloadData } from "../hakanai-client";
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
import { isFileShareSupported, isWebShareSupported, createShareableFile, shareContent } from "../core/web-share";
import { getFileIcon, sanitizeFileName, getExt, isImageExt } from "../core/file-utils";

const TIMEOUTS = {
  CLEANUP_DELAY: 100,
} as const;

export function showSecret(
  payload: PayloadData,
  resultDiv: HTMLElement,
  resetCallback: () => void,
  secretId: string,
): void {
  if (!payload || !resultDiv) return;

  resultDiv.className = "result success";

  const title = document.createElement("h3");
  title.textContent = window.i18n.t(I18nKeys.Msg.SuccessTitle);
  resultDiv.appendChild(title);

  const decodedBytes = payload.decodeBytes();
  const isBinaryFile = payload.filename != null || ContentAnalysis.isBinary(decodedBytes);

  if (payload.filename) {
    const sanitized = sanitizeFileName(payload.filename);
    payload.setFilename(sanitized!);
  } else {
    const extension = isBinaryFile ? ".bin" : ".txt";
    const filename = generateFilename(secretId, extension);
    payload.setFilename(filename);
  }

  const container = isBinaryFile
    ? createBinarySecret(payload, decodedBytes.buffer as ArrayBuffer, decodedBytes.length)
    : createTextSecret(payload, secretId, decodedBytes.buffer as ArrayBuffer);
  resultDiv.appendChild(container);

  const noteElement = createNoteElement();
  addResetElement(noteElement, resetCallback);
  resultDiv.appendChild(noteElement);

  announceToScreenReader(window.i18n.t(I18nKeys.Msg.SuccessTitle));
}

function createTextSecret(payload: PayloadData, secretId: string, decodedBytes: ArrayBuffer): HTMLElement {
  const elementId = `secret-${secretId}`;
  const container = document.createElement("div");
  container.className = "secret-container";

  const textarea = createSecretTextarea(elementId, decodedBytes);
  container.appendChild(textarea);

  const buttonsContainer = createButtonContainer();
  buttonsContainer.appendChild(createCopyButton(elementId));
  buttonsContainer.appendChild(createDownloadButton(payload, decodedBytes));

  if (isWebShareSupported() && (!payload.filename || isFileShareSupported())) {
    buttonsContainer.appendChild(createShareButton(payload, decodedBytes));
  }

  container.appendChild(buttonsContainer);

  expandView();
  if (window.innerWidth > 640) {
    resizeTextarea(textarea);
  }

  return container;
}

function createSecretTextarea(elementId: string, decodedBytes: ArrayBuffer): HTMLTextAreaElement {
  const textarea = document.createElement("textarea");
  textarea.id = elementId;
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

function createBinarySecret(payload: PayloadData, decodedBytes: ArrayBuffer, size: number): HTMLElement {
  const container = document.createElement("div");
  container.className = "secret-container file-secret-container";

  const fileInfoSection = document.createElement("div");
  fileInfoSection.className = "file-info-section";

  const filenameWrapper = document.createElement("div");
  filenameWrapper.className = "file-name-wrapper";

  const icon = getFileIcon(payload.filename || "");
  const filenameElement = document.createElement("h4");
  filenameElement.className = "file-name";
  filenameElement.textContent = `${icon} ${payload.filename}`;

  filenameWrapper.appendChild(filenameElement);
  fileInfoSection.appendChild(filenameWrapper);

  const sizeElement = document.createElement("p");
  sizeElement.className = "file-size";
  sizeElement.innerHTML = `<strong>${window.i18n.t(I18nKeys.Label.Size)}</strong> ${formatFileSize(size)}`;
  fileInfoSection.appendChild(sizeElement);

  container.appendChild(fileInfoSection);

  const buttonsContainer = createButtonContainer();

  buttonsContainer.appendChild(createDownloadButton(payload, decodedBytes));

  if (isFileShareSupported()) {
    buttonsContainer.appendChild(createShareButton(payload, decodedBytes));
  }

  if (hasPreviewSupport(payload)) {
    buttonsContainer.appendChild(createPreviewButton(payload, decodedBytes));
  }

  container.appendChild(buttonsContainer);

  return container;
}

function hasPreviewSupport(payload: PayloadData) {
  const filename = payload.filename;
  if (!filename) return false;

  const ext = getExt(filename);
  return isImageExt(ext);
}

function createPreviewButton(payload: PayloadData, decodedBytes: ArrayBuffer): HTMLButtonElement {
  return createButton(
    "btn preview-btn",
    window.i18n.t(I18nKeys.Button.Preview) || "Preview",
    window.i18n.t(I18nKeys.Aria.PreviewContent) || "Preview content",
    () => showImagePreview(payload, decodedBytes),
  );
}

function createCopyButton(elementId: string): HTMLButtonElement {
  return createButton(
    "btn copy-btn",
    window.i18n.t(I18nKeys.Button.Copy),
    window.i18n.t(I18nKeys.Aria.CopySecret),
    function (this: HTMLButtonElement) {
      copySecret(elementId, this);
    },
  );
}

function createDownloadButton(payload: PayloadData, decodedBytes: ArrayBuffer): HTMLButtonElement {
  return createButton(
    "btn download-btn",
    window.i18n.t(I18nKeys.Button.Download),
    window.i18n.t(I18nKeys.Aria.DownloadSecret),
    () => downloadSecret(payload, decodedBytes),
  );
}

function createShareButton(payload: PayloadData, decodedBytes: ArrayBuffer): HTMLButtonElement {
  const clickHandler = payload.filename ? () => shareFileSecret(payload, decodedBytes) : () => shareTextSecret(payload);

  return createButton(
    "btn share-btn",
    window.i18n.t(I18nKeys.Button.Share),
    window.i18n.t(I18nKeys.Aria.ShareSecret),
    clickHandler,
  );
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

function copySecret(elementId: string, button: HTMLButtonElement): void {
  const secretElement = document.getElementById(elementId) as HTMLTextAreaElement;
  if (!secretElement) return;

  copyToClipboard(secretElement.value, button);
}

function generateFilename(secretId: string, extension: string): string {
  return `secret-${secretId}${extension}`;
}

function downloadSecret(payload: PayloadData, decodedBytes: ArrayBuffer): void {
  const mimeType = payload.filename ? "application/octet-stream" : "text/plain;charset=utf-8";

  const blob = new Blob([decodedBytes], { type: mimeType });
  const url = window.URL.createObjectURL(blob);

  const anchor = document.createElement("a");
  hideElement(anchor);
  anchor.href = url;
  anchor.download = payload.filename || "";

  document.body.appendChild(anchor);
  anchor.click();

  setTimeout(() => {
    document.body.removeChild(anchor);
    window.URL.revokeObjectURL(url);
  }, TIMEOUTS.CLEANUP_DELAY);

  announceToScreenReader(window.i18n.t(I18nKeys.Msg.Downloaded));
}

async function shareTextSecret(payload: PayloadData): Promise<void> {
  try {
    const text = payload.decode();

    await shareContent({
      text: text,
      title: "Hakanai Secret",
    });

    announceToScreenReader(window.i18n.t(I18nKeys.Msg.Shared));
  } catch (error) {
    if (error instanceof Error && error.name !== "AbortError") {
      console.error("Failed to share:", error);
    }
  }
}

async function shareFileSecret(payload: PayloadData, decodedBytes: ArrayBuffer): Promise<void> {
  try {
    const file = createShareableFile(decodedBytes, payload.filename || "");

    await shareContent({
      files: [file],
      title: "Hakanai Secret",
    });

    announceToScreenReader(window.i18n.t(I18nKeys.Msg.Shared));
  } catch (error) {
    if (error instanceof Error && error.name !== "AbortError") {
      console.error("Failed to share:", error);
    }
  }
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
  buttonsContainer.appendChild(createDownloadButton(payload, decodedBytes));

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
