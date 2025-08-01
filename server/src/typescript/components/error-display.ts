/**
 * Error display component for consistent error UI across pages
 */

import { I18nKeys } from "../core/i18n";
import { announceToScreenReader } from "../core/dom-utils";

/**
 * Display an error message in a consistent format
 * @param message - The error message to display
 * @param container - The HTML element to display the error in
 */
export function displayErrorMessage(
  message: string,
  container: HTMLElement,
): void {
  container.className = "result error";
  container.innerHTML = "";

  const title = document.createElement("h3");
  title.textContent = (window as any).i18n.t(I18nKeys.Msg.ErrorTitle);
  container.appendChild(title);

  const errorDiv = document.createElement("div");
  errorDiv.textContent = message;
  container.appendChild(errorDiv);

  announceToScreenReader(
    `${(window as any).i18n.t(I18nKeys.Msg.ErrorTitle)}: ${message}`,
  );
}
