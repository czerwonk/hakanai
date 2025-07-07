import { HakanaiClient } from "/scripts/hakanai-client.js";
import {
  createButton,
  createButtonContainer,
  copyToClipboard,
  announceToScreenReader,
  debounce,
} from "/common-utils.js";

// Listen for language changes to update dynamic content
document.addEventListener("languageChanged", function () {
  updateUIStrings();
});

function updateUIStrings() {
  UI_STRINGS.EMPTY_URL = i18n.t("msg.emptyUrl");
  UI_STRINGS.INVALID_URL = i18n.t("msg.invalidUrl");
  UI_STRINGS.RETRIEVE_FAILED = i18n.t("msg.retrieveFailed");
  UI_STRINGS.SUCCESS_TITLE = i18n.t("msg.successTitle");
  UI_STRINGS.ERROR_TITLE = i18n.t("msg.errorTitle");
  UI_STRINGS.COPY_TEXT = i18n.t("button.copy");
  UI_STRINGS.COPIED_TEXT = i18n.t("button.copied");
  UI_STRINGS.COPY_FAILED = i18n.t("msg.copyFailed");
  UI_STRINGS.DOWNLOAD_TEXT = i18n.t("button.download");
  UI_STRINGS.NOTE_TEXT = i18n.t("msg.retrieveNote");
  UI_STRINGS.BINARY_DETECTED = i18n.t("msg.binaryDetected");
  UI_STRINGS.COPY_ARIA = i18n.t("aria.copySecret");
  UI_STRINGS.DOWNLOAD_ARIA = i18n.t("aria.downloadSecret");
  UI_STRINGS.FILENAME_LABEL = i18n.t("label.filename");
}

const UI_STRINGS = {
  EMPTY_URL: "Please enter a valid secret URL",
  INVALID_URL:
    "Invalid URL format. Please include the full URL with the secret key after #",
  RETRIEVE_FAILED: "Failed to retrieve secret",
  SUCCESS_TITLE: "Secret Retrieved Successfully",
  ERROR_TITLE: "Error",
  COPY_TEXT: "Copy",
  COPIED_TEXT: "Copied!",
  COPY_FAILED: "Failed to copy. Please select and copy manually.",
  DOWNLOAD_TEXT: "Download",
  NOTE_TEXT:
    "Note: This secret has been deleted from the server and cannot be accessed again.",
  BINARY_DETECTED:
    "Binary file detected. Content hidden for security. Use download button to save the file.",
  COPY_ARIA: "Copy secret to clipboard",
  DOWNLOAD_ARIA: "Download secret as file",
  FILENAME_LABEL: "Filename:",
};

const TIMEOUTS = {
  DEBOUNCE: 300,
  CLEANUP_DELAY: 100,
};

// Extract base URL from current location or use a default
const baseUrl = window.location.origin.includes("file://")
  ? "http://localhost:8080"
  : window.location.origin;

const client = new HakanaiClient(baseUrl);

const retrieveSecretDebounced = debounce(async function retrieveSecret() {
  const urlInput = document.getElementById("secretUrl");
  const resultDiv = document.getElementById("result");
  const loadingDiv = document.getElementById("loading");
  const button = document.getElementById("retrieveBtn");

  const url = urlInput.value.trim();

  if (!url) {
    showError(UI_STRINGS.EMPTY_URL);
    urlInput.focus();
    return;
  }

  let processedUrl = url;

  if (!url.match(/^[a-zA-Z][a-zA-Z0-9+.-]*:\/\//)) {
    // If URL doesn't start with a scheme, prepend the current location's scheme
    const currentScheme = window.location.protocol;
    processedUrl = currentScheme + "//" + url;
  }

  try {
    const urlObj = new URL(processedUrl);
    if (!urlObj.hash || urlObj.hash.length <= 1) {
      throw new Error("Missing hash");
    }
  } catch (error) {
    // Since msg.invalidUrl already mentions missing hash, just use it
    showError(UI_STRINGS.INVALID_URL);
    urlInput.focus();
    return;
  }

  // Show loading state
  loadingDiv.style.display = "block";
  button.disabled = true;
  urlInput.disabled = true;
  resultDiv.innerHTML = "";

  try {
    const payload = await client.receivePayload(processedUrl);

    showSuccess(payload);

    // Clear the input
    urlInput.value = "";
  } catch (error) {
    showError(error.message || UI_STRINGS.RETRIEVE_FAILED);
  } finally {
    loadingDiv.style.display = "none";
    button.disabled = false;
    urlInput.disabled = false;
  }
}, TIMEOUTS.DEBOUNCE);

async function retrieveSecret() {
  retrieveSecretDebounced();
}

function showSuccess(payload) {
  const resultDiv = document.getElementById("result");
  resultDiv.className = "result success";
  const secretId = "secret-" + Date.now();

  resultDiv.innerHTML = "";

  // Create elements programmatically to avoid XSS
  const title = document.createElement("h3");
  title.textContent = UI_STRINGS.SUCCESS_TITLE;
  resultDiv.appendChild(title);

  const container = document.createElement("div");
  container.className = "secret-container";

  // Check if this is a binary file (has filename)
  const isBinaryFile =
    payload.filename !== null && payload.filename !== undefined;

  if (!isBinaryFile) {
    // Only show content for text secrets (no filename)
    const textarea = document.createElement("textarea");
    textarea.id = secretId;
    textarea.className = "secret-display";
    textarea.readOnly = true;
    textarea.setAttribute("aria-label", "Retrieved secret content");

    // Use the decode method from PayloadData
    textarea.value = payload.decode();

    textarea.addEventListener("click", function () {
      this.select();
    });
    container.appendChild(textarea);

    const buttonsContainer = createButtonContainer();

    const copyBtn = createButton(
      "copy-button",
      UI_STRINGS.COPY_TEXT,
      UI_STRINGS.COPY_ARIA,
      function () {
        copySecret(secretId, this);
      },
    );
    buttonsContainer.appendChild(copyBtn);

    const downloadBtn = createButton(
      "download-button",
      UI_STRINGS.DOWNLOAD_TEXT,
      UI_STRINGS.DOWNLOAD_ARIA,
      function () {
        downloadSecret(payload);
      },
    );
    buttonsContainer.appendChild(downloadBtn);

    container.appendChild(buttonsContainer);
  } else {
    // For binary files, only show download button and message
    const message = document.createElement("p");
    message.style.marginBottom = "var(--spacing-md, 1rem)";
    message.style.color = "var(--color-text, #000000)";
    message.textContent = UI_STRINGS.BINARY_DETECTED;
    container.appendChild(message);

    const buttonsContainer = createButtonContainer();

    const downloadBtn = createButton(
      "download-button",
      UI_STRINGS.DOWNLOAD_TEXT,
      UI_STRINGS.DOWNLOAD_ARIA,
      function () {
        downloadSecret(payload);
      },
    );
    buttonsContainer.appendChild(downloadBtn);

    container.appendChild(buttonsContainer);
  }

  resultDiv.appendChild(container);

  // Show filename if available
  if (payload.filename) {
    const fileInfo = document.createElement("p");
    fileInfo.style.marginTop = "var(--spacing-sm, 0.75rem)";
    fileInfo.style.fontSize = "0.875rem";
    fileInfo.style.color = "var(--color-text-muted, #888888)";

    const fileLabel = document.createElement("strong");
    fileLabel.textContent = UI_STRINGS.FILENAME_LABEL + " ";
    fileInfo.appendChild(fileLabel);
    fileInfo.appendChild(document.createTextNode(payload.filename));
    resultDiv.appendChild(fileInfo);
  }

  const note = document.createElement("p");
  note.style.marginTop = "var(--spacing-sm, 0.75rem)";
  note.style.fontSize = "0.875rem";
  note.style.color = "var(--color-text-muted, #888888)";

  // Create strong element for "Note:" text
  const strong = document.createElement("strong");
  strong.textContent = i18n.t("msg.retrieveNote").split(":")[0] + ":";
  note.appendChild(strong);

  // Add the rest of the text
  note.appendChild(
    document.createTextNode(" " + i18n.t("msg.retrieveNoteText")),
  );
  resultDiv.appendChild(note);

  announceToScreenReader(UI_STRINGS.SUCCESS_TITLE);
}

function showError(message) {
  const resultDiv = document.getElementById("result");
  resultDiv.className = "result error";

  // Clear existing content
  resultDiv.innerHTML = "";

  // Create elements programmatically to avoid XSS
  const title = document.createElement("h3");
  title.textContent = UI_STRINGS.ERROR_TITLE;
  resultDiv.appendChild(title);

  const errorDiv = document.createElement("div");
  errorDiv.textContent = message;
  resultDiv.appendChild(errorDiv);

  announceToScreenReader(`${UI_STRINGS.ERROR_TITLE}: ${message}`);
}

function copySecret(secretId, button) {
  const secretElement = document.getElementById(secretId);
  if (!secretElement) {
    alert(UI_STRINGS.COPY_FAILED);
    return;
  }

  const originalText = button.textContent;
  const secretText = secretElement.value;

  copyToClipboard(
    secretText,
    button,
    originalText,
    UI_STRINGS.COPIED_TEXT,
    UI_STRINGS.COPY_FAILED,
  );
}

function downloadSecret(payload) {
  // Determine the filename
  let filename;
  if (payload.filename) {
    filename = payload.filename;
  } else {
    filename = `hakanai-secret-${new Date().toISOString().replace(/[:.]/g, "-")}.txt`;
  }

  // Use the decodeBytes method from PayloadData
  const blobData = payload.decodeBytes();

  // Create a blob from the decoded data
  const blob = new Blob([blobData], {
    type: payload.filename
      ? "application/octet-stream"
      : "text/plain;charset=utf-8",
  });

  // Create a temporary URL for the blob
  const url = window.URL.createObjectURL(blob);

  // Create a temporary anchor element and trigger download
  const a = document.createElement("a");
  a.style.display = "none";
  a.href = url;
  a.download = filename;

  document.body.appendChild(a);
  a.click();

  // Clean up
  setTimeout(() => {
    document.body.removeChild(a);
    window.URL.revokeObjectURL(url);
  }, TIMEOUTS.CLEANUP_DELAY);

  announceToScreenReader(i18n.t("msg.downloaded"));
}

const urlInput = document.getElementById("secretUrl");
if (urlInput) {
  urlInput.placeholder = `${baseUrl}/s/uuid#key`;

  // Check if current URL is a short link and prefill the input
  if (window.location.pathname.match(/^\/s\/[^\/]+$/)) {
    urlInput.value = window.location.href;
  }
}

// Initialize UI strings after DOM is ready
document.addEventListener("DOMContentLoaded", function () {
  updateUIStrings();
});

// Add event listener for form submission
document.addEventListener("DOMContentLoaded", function () {
  const form = document.querySelector("form");
  if (form) {
    form.addEventListener("submit", function (event) {
      event.preventDefault();
      retrieveSecret();
    });
  }
});
