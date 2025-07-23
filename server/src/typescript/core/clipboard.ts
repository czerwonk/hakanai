import { announceToScreenReader } from "./dom-utils.js";

// Constants
const COPY_FEEDBACK_TIMEOUT = 2000;

declare global {
  interface Window {
    i18n: {
      t(key: string): string;
    };
  }
}

/**
 * Copy text to clipboard with visual feedback
 * @param text - Text to copy to clipboard
 * @param button - Button element to show feedback on
 */
export function copyToClipboard(text: string, button: HTMLButtonElement): void {
  const originalText = button.textContent || "Copy";
  navigator.clipboard
    .writeText(text)
    .then(() =>
      showCopySuccess(
        button,
        originalText,
        window.i18n?.t("button.copied") || "Copied!",
      ),
    )
    .catch(() =>
      showCopyFailure(
        button,
        originalText,
        window.i18n?.t("msg.copyFailed") || "Copy Failed",
      ),
    );
}

function showCopySuccess(
  button: HTMLButtonElement,
  originalText: string,
  successMessage: string,
): void {
  button.textContent = successMessage;
  button.classList.add("copied");
  announceToScreenReader(successMessage);

  setTimeout(() => {
    button.textContent = originalText;
    button.classList.remove("copied");
  }, COPY_FEEDBACK_TIMEOUT);
}

function showCopyFailure(
  button: HTMLButtonElement,
  originalText: string,
  failedMessage: string,
): void {
  // Show failure state visually without disruptive alerts
  button.textContent = failedMessage || "Copy Failed";
  button.classList.add("copy-failed");
  announceToScreenReader(failedMessage || "Copy Failed");

  setTimeout(() => {
    button.textContent = originalText;
    button.classList.remove("copy-failed");
  }, COPY_FEEDBACK_TIMEOUT);
}

export function copyToClipboardByElementId(
  elementId: string,
  button: HTMLButtonElement,
): void {
  const input = document.getElementById(elementId) as HTMLInputElement;
  if (input) {
    copyToClipboard(input.value, button);
  } else {
    showCopyFailure(
      button,
      button.textContent || "Copy",
      window.i18n?.t("msg.copyFailed") || "Copy Failed",
    );
  }
}
