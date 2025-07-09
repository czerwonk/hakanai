import { HakanaiClient } from "/scripts/hakanai-client.js";
import {
  createButton,
  createButtonContainer,
  copyToClipboard,
  announceToScreenReader,
} from "/common-utils.js";

// Listen for language changes to update dynamic content
document.addEventListener("languageChanged", function () {
  updateUIStrings();
});

function updateUIStrings() {
  UI_STRINGS.EMPTY_SECRET = i18n.t("msg.emptySecret");
  UI_STRINGS.EMPTY_FILE = i18n.t("msg.emptyFile");
  UI_STRINGS.CREATE_FAILED = i18n.t("msg.createFailed");
  UI_STRINGS.SUCCESS_TITLE = i18n.t("msg.successTitle");
  UI_STRINGS.ERROR_TITLE = i18n.t("msg.errorTitle");
  UI_STRINGS.COPY_TEXT = i18n.t("button.copy");
  UI_STRINGS.COPIED_TEXT = i18n.t("button.copied");
  UI_STRINGS.COPY_FAILED = i18n.t("msg.copyFailed");
  UI_STRINGS.NOTE_TEXT = i18n.t("msg.createNote");
  UI_STRINGS.SHARE_INSTRUCTIONS = i18n.t("msg.shareInstructions");
  UI_STRINGS.FILE_TOO_LARGE = i18n.t("msg.fileTooLarge");
  UI_STRINGS.FILE_READ_ERROR = i18n.t("msg.fileReadError");
  UI_STRINGS.INVALID_FILENAME = i18n.t("msg.invalidFilename");
}

const UI_STRINGS = {
  EMPTY_SECRET: "Please enter a secret to share",
  EMPTY_FILE: "Please select a file to share",
  CREATE_FAILED: "Failed to create secret",
  SUCCESS_TITLE: "Secret Created Successfully",
  ERROR_TITLE: "Error",
  COPY_TEXT: "Copy URL",
  COPIED_TEXT: "Copied!",
  COPY_FAILED: "Failed to copy. Please select and copy manually.",
  NOTE_TEXT:
    "Note: Share this URL carefully. The secret will be deleted after the first access or when it expires.",
  SHARE_INSTRUCTIONS:
    "Share this URL with the intended recipient. The secret is encrypted and can only be accessed once.",
  FILE_TOO_LARGE: "File size exceeds 10MB limit",
  FILE_READ_ERROR: "Error reading file",
  INVALID_FILENAME: "Invalid filename. Please select a file with a valid name.",
};

const FILE_LIMITS = {
  MAX_SIZE: 10 * 1024 * 1024, // 10MB in bytes
};

// Extract base URL from current location or use a default
const baseUrl = window.location.origin.includes("file://")
  ? "http://localhost:8080"
  : window.location.origin;

const client = new HakanaiClient(baseUrl);

async function validateAndProcessFileInput(fileInput) {
  const file = fileInput.files[0];
  if (!file) {
    showError(UI_STRINGS.EMPTY_FILE);
    fileInput.focus();
    return null;
  }

  if (file.size > FILE_LIMITS.MAX_SIZE) {
    showError(UI_STRINGS.FILE_TOO_LARGE);
    fileInput.focus();
    return null;
  }

  const fileName = sanitizeFileName(file.name);
  if (!fileName) {
    showError(UI_STRINGS.INVALID_FILENAME);
    fileInput.focus();
    return null;
  }

  try {
    const fileBytes = await readFileAsBytes(file);
    const payload = client.createPayload(fileName);
    payload.setFromBytes(fileBytes);
    return payload;
  } catch (error) {
    showError(UI_STRINGS.FILE_READ_ERROR);
    return null;
  }
}

function validateTextInput(secretInput) {
  const secret = secretInput.value.trim();
  if (!secret) {
    showError(UI_STRINGS.EMPTY_SECRET);
    secretInput.focus();
    return null;
  }

  // Check for TextEncoder support
  if (typeof TextEncoder === "undefined") {
    showError(
      "Your browser doesn't support text encoding. Please use a modern browser.",
    );
    return null;
  }

  const encoder = new TextEncoder();
  const textBytes = encoder.encode(secret);
  const payload = client.createPayload();
  payload.setFromBytes(textBytes);
  return payload;
}

function setLoadingState(elements, isLoading) {
  const {
    loadingDiv,
    button,
    secretInput,
    fileInput,
    authTokenInput,
    ttlSelect,
    textRadio,
    fileRadio,
    resultDiv,
  } = elements;

  loadingDiv.style.display = isLoading ? "block" : "none";
  button.disabled = isLoading;
  secretInput.disabled = isLoading;
  fileInput.disabled = isLoading;
  authTokenInput.disabled = isLoading;
  ttlSelect.disabled = isLoading;
  textRadio.disabled = isLoading;
  fileRadio.disabled = isLoading;

  if (isLoading) {
    resultDiv.innerHTML = "";
  }
}

function clearInputs(secretInput, fileInput) {
  secretInput.value = "";
  fileInput.value = "";
  updateFileInfo();
}

async function createSecret() {
  const secretInput = document.getElementById("secretText");
  const fileInput = document.getElementById("secretFile");
  const authTokenInput = document.getElementById("authToken");
  const ttlSelect = document.getElementById("ttlSelect");
  const resultDiv = document.getElementById("result");
  const loadingDiv = document.getElementById("loading");
  const button = document.getElementById("createBtn");
  const textRadio = document.getElementById("textRadio");
  const fileRadio = document.getElementById("fileRadio");

  // Check if all required elements exist
  if (
    !secretInput ||
    !fileInput ||
    !authTokenInput ||
    !ttlSelect ||
    !resultDiv ||
    !loadingDiv ||
    !button ||
    !textRadio ||
    !fileRadio
  ) {
    showError("Page not fully loaded. Please refresh and try again.");
    return;
  }

  const authToken = authTokenInput.value.trim();
  const ttl = parseInt(ttlSelect.value);
  const isFileMode = fileRadio.checked;

  const elements = {
    loadingDiv,
    button,
    secretInput,
    fileInput,
    authTokenInput,
    ttlSelect,
    textRadio,
    fileRadio,
    resultDiv,
  };

  let payload;
  if (isFileMode) {
    payload = await validateAndProcessFileInput(fileInput);
  } else {
    payload = validateTextInput(secretInput);
  }

  if (!payload) {
    return;
  }

  setLoadingState(elements, true);

  try {
    const secretUrl = await client.sendPayload(payload, ttl, authToken);
    showSuccess(secretUrl);
    clearInputs(secretInput, fileInput);
  } catch (error) {
    // Check if it's a HakanaiError with a code for localization
    if (error.name === "HakanaiError" && error.code) {
      const errorKey = `error.${error.code}`;
      const localizedMessage = i18n.t(errorKey);
      // Fall back to the original message if translation is not found
      const finalMessage =
        localizedMessage !== errorKey ? localizedMessage : error.message;
      showError(finalMessage);
    } else {
      showError(error.message || UI_STRINGS.CREATE_FAILED);
    }
  } finally {
    setLoadingState(elements, false);
  }
}

function showSuccess(secretUrl) {
  const resultContainer = document.getElementById("result");
  if (!resultContainer) {
    console.error("Result container not found");
    return;
  }

  resultContainer.className = "result success";
  const urlId = crypto?.randomUUID
    ? `url-${crypto.randomUUID()}`
    : `url-${Date.now()}-${Math.random()}`;

  resultContainer.innerHTML = "";

  // Hide the form elements after successful creation
  const form = document.getElementById("create-secret-form");
  if (form) {
    form.style.display = "none";
  }

  // Create header section
  createSuccessHeader(resultContainer);

  // Create URL display section
  const container = createUrlDisplaySection(secretUrl, urlId);
  resultContainer.appendChild(container);

  // Create note section
  createNoteSection(resultContainer);

  // Announce to screen readers
  announceToScreenReader(UI_STRINGS.SUCCESS_TITLE);
}

function createSuccessHeader(container) {
  const title = document.createElement("h3");
  title.textContent = UI_STRINGS.SUCCESS_TITLE;
  container.appendChild(title);

  const instructions = document.createElement("p");
  instructions.className = "share-instructions";
  instructions.textContent = UI_STRINGS.SHARE_INSTRUCTIONS;
  container.appendChild(instructions);
}

function createUrlDisplaySection(secretUrl, urlId) {
  const container = document.createElement("div");
  container.className = "secret-container";

  const urlDisplay = document.createElement("input");
  urlDisplay.type = "text";
  urlDisplay.id = urlId;
  urlDisplay.readOnly = true;
  urlDisplay.setAttribute("aria-label", "Secret URL");
  urlDisplay.value = secretUrl;
  urlDisplay.className = "url-display";
  urlDisplay.addEventListener("click", function () {
    this.select();
  });
  container.appendChild(urlDisplay);

  const buttonsContainer = createButtonContainer();

  const copyBtn = createButton(
    "copy-button",
    UI_STRINGS.COPY_TEXT,
    "Copy secret URL to clipboard",
    function () {
      copyUrl(urlId, this);
    },
  );
  buttonsContainer.appendChild(copyBtn);

  const createAnotherBtn = createButton(
    "secondary-button",
    i18n.t("button.createAnother"),
    "Create another secret",
    function () {
      resetToCreateMode();
    },
  );
  buttonsContainer.appendChild(createAnotherBtn);

  container.appendChild(buttonsContainer);
  return container;
}

function createNoteSection(container) {
  const note = document.createElement("p");
  note.className = "secret-note";

  // Handle i18n note text more safely
  const noteText = i18n.t("msg.createNote");
  const colonIndex = noteText.indexOf(":");

  if (colonIndex > 0) {
    const strong = document.createElement("strong");
    strong.textContent = noteText.substring(0, colonIndex + 1);
    note.appendChild(strong);
    note.appendChild(
      document.createTextNode(" " + noteText.substring(colonIndex + 1).trim()),
    );
  } else {
    // Fallback if no colon found
    const strong = document.createElement("strong");
    strong.textContent = "Note: ";
    note.appendChild(strong);
    note.appendChild(document.createTextNode(i18n.t("msg.createNoteText")));
  }

  container.appendChild(note);
}

function resetToCreateMode() {
  const form = document.getElementById("create-secret-form");
  const resultContainer = document.getElementById("result");

  if (form) {
    form.style.display = "block";
  }
  if (resultContainer) {
    resultContainer.innerHTML = "";
    resultContainer.className = "";
  }

  // Focus on the first input
  const textRadio = document.getElementById("textRadio");
  const secretText = document.getElementById("secretText");
  if (textRadio && secretText) {
    textRadio.checked = true;
    toggleSecretType();
    secretText.focus();
  }
}

function showError(message) {
  const resultDiv = document.getElementById("result");
  resultDiv.className = "result error";

  // Clear existing content
  resultDiv.innerHTML = "";

  // Ensure form is visible for retry
  const form = document.getElementById("create-secret-form");
  if (form) {
    form.style.display = "block";
  }

  // Create elements programmatically to avoid XSS
  const title = document.createElement("h3");
  title.textContent = UI_STRINGS.ERROR_TITLE;
  resultDiv.appendChild(title);

  const errorDiv = document.createElement("div");
  errorDiv.textContent = message;
  resultDiv.appendChild(errorDiv);

  // Announce error to screen readers
  announceToScreenReader(`${UI_STRINGS.ERROR_TITLE}: ${message}`);
}

function copyUrl(urlId, button) {
  const urlElement = document.getElementById(urlId);
  if (!urlElement) {
    showError(UI_STRINGS.COPY_FAILED);
    return;
  }

  const originalText = button.textContent;
  const urlText = urlElement.value;

  copyToClipboard(
    urlText,
    button,
    originalText,
    UI_STRINGS.COPIED_TEXT,
    UI_STRINGS.COPY_FAILED,
  );
}

// File handling utilities
function sanitizeFileName(fileName) {
  // Remove potentially dangerous characters and limit length
  const sanitized = fileName
    .replace(/[<>:"/\\|?*\x00-\x1f]/g, "_") // Replace dangerous chars
    .replace(/^\.+/, "") // Remove leading dots
    .substring(0, 255); // Limit length

  return sanitized.length > 0 ? sanitized : null;
}

function readFileAsBytes(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = function (e) {
      try {
        const arrayBuffer = e.target.result;
        const bytes = new Uint8Array(arrayBuffer);
        resolve(bytes);
      } catch (error) {
        reject(error);
      }
    };
    reader.onerror = function () {
      reject(new Error("Failed to read file"));
    };
    reader.readAsArrayBuffer(file);
  });
}

function formatFileSize(bytes) {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function updateFileInfo() {
  const fileInput = document.getElementById("secretFile");
  const fileInfoDiv = document.getElementById("fileInfo");
  const fileNameSpan = document.getElementById("fileName");
  const fileSizeSpan = document.getElementById("fileSize");
  const radioGroup = document.querySelector(".input-group:first-child");
  const textInputGroup = document.getElementById("textInputGroup");
  const fileInputGroup = document.getElementById("fileInputGroup");
  const fileRadio = document.getElementById("fileRadio");

  if (fileInput.files.length > 0) {
    const file = fileInput.files[0];
    const sanitizedName = sanitizeFileName(file.name);

    fileNameSpan.textContent = sanitizedName || "Invalid filename";
    fileSizeSpan.textContent = formatFileSize(file.size);
    fileInfoDiv.style.display = "block";

    // Show warning if file is too large
    if (file.size > FILE_LIMITS.MAX_SIZE) {
      fileInfoDiv.className = "file-info error";
      fileSizeSpan.textContent = formatFileSize(file.size) + " (Too large!)";
    } else {
      fileInfoDiv.className = "file-info";
    }

    // Hide radio group and text input when file is selected
    radioGroup.style.display = "none";
    textInputGroup.style.display = "none";
    fileInputGroup.style.display = "block";

    // Select file radio button
    fileRadio.checked = true;
  } else {
    fileInfoDiv.style.display = "none";

    // Show radio group when no file is selected
    radioGroup.style.display = "block";

    // Reset to text mode when file is cleared
    const textRadio = document.getElementById("textRadio");
    textRadio.checked = true;
    toggleSecretType();
  }
}

function toggleSecretType() {
  const textRadio = document.getElementById("textRadio");
  const textInputGroup = document.getElementById("textInputGroup");
  const fileInputGroup = document.getElementById("fileInputGroup");
  const secretText = document.getElementById("secretText");
  const secretFile = document.getElementById("secretFile");

  if (textRadio.checked) {
    textInputGroup.style.display = "block";
    fileInputGroup.style.display = "none";
    secretText.required = true;
    secretFile.required = false;
    secretText.focus();
  } else {
    textInputGroup.style.display = "none";
    fileInputGroup.style.display = "block";
    secretText.required = false;
    secretFile.required = true;
    secretFile.focus();
  }
}

// Set up all event handlers when DOM is ready
document.addEventListener("DOMContentLoaded", function () {
  // Initialize UI strings after i18n is loaded
  updateUIStrings();

  // Focus on the secret input
  const secretText = document.getElementById("secretText");
  if (secretText) {
    secretText.focus();
  }

  // Set up form submission handler
  const form = document.querySelector("form");
  if (form) {
    form.addEventListener("submit", function (event) {
      event.preventDefault();
      createSecret();
    });
  }

  // Set up radio button handlers
  const textRadio = document.getElementById("textRadio");
  const fileRadio = document.getElementById("fileRadio");
  const fileInput = document.getElementById("secretFile");

  if (textRadio && fileRadio) {
    textRadio.addEventListener("change", toggleSecretType);
    fileRadio.addEventListener("change", toggleSecretType);

    // Initialize with correct state - text mode by default
    toggleSecretType();
  }

  // Set up file input handler
  if (fileInput) {
    fileInput.addEventListener("change", updateFileInfo);
  }
});
