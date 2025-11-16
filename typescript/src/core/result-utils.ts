// SPDX-License-Identifier: Apache-2.0

import { createButton, createButtonContainer } from "../core/dom-utils";
import { copyToClipboardByElementId } from "../core/clipboard";
import { QRCodeGenerator } from "../core/qr-generator";
import { I18nKeys } from "../core/i18n";
import { isWebShareSupported, shareContent } from "../core/web-share";

/**
 * Create a labeled input field with copy button
 */
export function createLabeledInputWithCopy(
  container: HTMLElement,
  labelKey: string,
  inputId: string,
  value: string,
  ariaLabel: string,
): void {
  const label = document.createElement("label");
  label.textContent = window.i18n.t(labelKey);
  label.setAttribute("for", inputId);
  container.appendChild(label);

  const inputContainer = document.createElement("div");
  inputContainer.className = "input-group";

  const input = document.createElement("input");
  input.type = "text";
  input.id = inputId;
  input.value = value;
  input.readOnly = true;
  input.className = "url-input";
  inputContainer.appendChild(input);

  const buttonContainer = createButtonContainer();

  const copyButton = createButton("btn copy-btn", window.i18n.t(I18nKeys.Button.Copy), ariaLabel, () =>
    copyToClipboardByElementId(inputId, copyButton as HTMLButtonElement),
  );
  buttonContainer.appendChild(copyButton);

  if (isWebShareSupported()) {
    const shareButton = createShareButton(value);
    buttonContainer.appendChild(shareButton);
  }

  const qrButton = createQrButton(value);
  buttonContainer.appendChild(qrButton);

  inputContainer.appendChild(buttonContainer);
  container.appendChild(inputContainer);
}

/**
 * Create a share button for native sharing
 */
function createShareButton(url: string): HTMLButtonElement {
  const button = createButton(
    "btn share-btn",
    window.i18n.t(I18nKeys.Button.Share),
    window.i18n.t(I18nKeys.Aria.ShareSecret),
    async () => {
      try {
        await shareContent({
          title: window.i18n.t(I18nKeys.Msg.ShareTitle),
          text: window.i18n.t(I18nKeys.Msg.ShareText),
          url: url,
        });
      } catch (error) {
        if (error instanceof Error && error.name !== "AbortError") {
          console.error("Share failed:", error);
        }
      }
    },
  );
  return button;
}

/**
 * Create a QR code button
 */
function createQrButton(url: string): HTMLButtonElement {
  return createButton("btn secondary-btn", "▦ QR", window.i18n.t(I18nKeys.Button.ShowQrCode), async () => {
    try {
      await QRCodeGenerator.ensureWasmLoaded();
      const qrSvg = QRCodeGenerator.generateQRCode(url);
      if (qrSvg) {
        showQRFullscreen(qrSvg);
      }
    } catch (error) {
      console.error("Failed to generate QR code:", error);
    }
  });
}

/**
 * Show QR code in fullscreen modal
 */
function showQRFullscreen(qrSvg: string): void {
  // Create fullscreen overlay
  const overlay = document.createElement("div");
  overlay.className = "qr-fullscreen-overlay";

  // Create container for the QR code
  const qrFullscreenContainer = document.createElement("div");
  qrFullscreenContainer.className = "qr-fullscreen-container";

  // Close handler (declare early for use in close button)
  const closeModal = () => {
    overlay.remove();
    document.removeEventListener("keydown", escapeHandler);
  };

  // Add close button
  const closeButton = createButton(
    "btn qr-close-btn",
    "✕",
    window.i18n.t(I18nKeys.Button.Close) || "Close",
    (e: Event) => {
      e.stopPropagation(); // Prevent closing the modal
      closeModal();
    },
  );
  qrFullscreenContainer.appendChild(closeButton);

  // Add QR code
  const qrContainer = document.createElement("div");
  qrContainer.innerHTML = qrSvg;
  qrFullscreenContainer.appendChild(qrContainer);

  // Add download button
  const downloadButton = createButton(
    "btn qr-download-btn",
    window.i18n.t(I18nKeys.Button.Download) || "Download",
    window.i18n.t(I18nKeys.Aria.DownloadQRCode),
    (e: Event) => {
      e.stopPropagation(); // Prevent closing the modal
      downloadQRCode(qrSvg);
    },
  );
  qrFullscreenContainer.appendChild(downloadButton);

  overlay.appendChild(qrFullscreenContainer);

  // Close on click
  overlay.addEventListener("click", (e) => {
    if (e.target === overlay) {
      closeModal();
    }
  });

  // Close on Escape key
  const escapeHandler = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      closeModal();
    }
  };
  document.addEventListener("keydown", escapeHandler);

  document.body.appendChild(overlay);
}

/**
 * Download QR code as SVG file
 */
function downloadQRCode(svgContent: string): void {
  // Create a blob from the SVG content
  const blob = new Blob([svgContent], { type: "image/svg+xml" });
  const url = URL.createObjectURL(blob);

  // Create a temporary link element and trigger download
  const link = document.createElement("a");
  link.href = url;
  link.download = `hakanai-qr-${Date.now()}.svg`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);

  // Clean up the blob URL
  setTimeout(() => URL.revokeObjectURL(url), 100);
}
