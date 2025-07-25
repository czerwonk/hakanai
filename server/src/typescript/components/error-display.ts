/**
 * Error display component for consistent error UI across pages
 */

import { I18nKeys } from "../core/i18n";
import { announceToScreenReader } from "../core/dom-utils";

export interface ErrorDisplayOptions {
  containerId?: string;
}

/**
 * Display an error message in a consistent format
 * @param message - The error message to display
 * @param options - Display options for page-specific behavior
 */
export function displayErrorMessage(
  message: string,
  options: ErrorDisplayOptions = {},
): void {
  const { containerId = "result" } = options;

  const resultDiv = document.getElementById(containerId);
  if (!resultDiv) {
    console.error(`Error container '${containerId}' not found`);
    return;
  }

  resultDiv.className = "result error";
  resultDiv.innerHTML = "";

  const title = document.createElement("h3");
  title.textContent = window.i18n.t(I18nKeys.Msg.ErrorTitle);
  resultDiv.appendChild(title);

  const errorDiv = document.createElement("div");
  errorDiv.textContent = message;
  resultDiv.appendChild(errorDiv);

  announceToScreenReader(
    `${window.i18n.t(I18nKeys.Msg.ErrorTitle)}: ${message}`,
  );
}
