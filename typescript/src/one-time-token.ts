// SPDX-License-Identifier: Apache-2.0

/**
 * Common functionality for static pages (homepage, impressum, privacy)
 * Contains shared theme and i18n components
 */
import { initI18n, I18nKeys } from "./core/i18n";
import { initTheme } from "./core/theme";
import { initFeatures } from "./core/app-config";
import { registerServiceWorker } from "./core/service-worker";
import { ErrorHandler, handleAPIError } from "./core/error";
import { generateRandomId, showElement } from "./core/dom-utils";
import { displayErrorMessage } from "./components/error-display";
import { createLabeledInputWithCopy } from "./core/result-utils";

class CreateOneTimeTokenErrorHandler implements ErrorHandler {
  displayError(message: string): void {
    showError(message);
  }
}

const errorHandler = new CreateOneTimeTokenErrorHandler();

function setupFormHandler(): void {
  const form = document.querySelector("form");
  if (form) {
    form.addEventListener("submit", (event) => {
      event.preventDefault();
      createToken();
    });
  }
}

async function createToken(): Promise<void> {
  try {
    const token = "test1234"; // TODO: implement API call

    showSuccess(token);
  } catch (error: unknown) {
    handleAPIError(error, window.i18n.t(I18nKeys.Msg.CreateOneTimeTokenFailed), errorHandler);
  }
}

function showError(message: string): void {
  // Page-specific behavior: show form after error
  const form = document.getElementById("one-time-token-form");
  if (form) {
    showElement(form);
  }

  const resultContainer = document.getElementById("result")!;
  displayErrorMessage(message, resultContainer);
}

function showSuccess(token: string): void {
  const container = document.getElementById("result");
  if (!container) {
    console.error("Result container not found");
    return;
  }

  container.className = "result success";
  container.innerHTML = "";

  const title = document.createElement("h3");
  title.textContent = window.i18n.t(I18nKeys.Msg.SuccessTitle);
  container.appendChild(title);

  const div = document.createElement("div");
  div.className = "url-container";

  const id = generateRandomId();
  const baseUrl = window.location.origin;
  const tokenUrl = `${baseUrl}/create#${token}`;
  createLabeledInputWithCopy(div, "Token URL", id, tokenUrl, "Copy URL to clipboard");

  container.appendChild(div);
}

document.addEventListener("DOMContentLoaded", async () => {
  initI18n();
  initTheme();
  initFeatures();

  setupFormHandler();

  await registerServiceWorker();
});
