// Listen for language changes to update dynamic content
document.addEventListener("languageChanged", function (e) {
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

const TIMEOUTS = {
  COPY_FEEDBACK: 2000,
};

const FILE_LIMITS = {
  MAX_SIZE: 10 * 1024 * 1024, // 10MB in bytes
};

// Extract base URL from current location or use a default
const baseUrl = window.location.origin.includes("file://")
  ? "http://localhost:8080"
  : window.location.origin;

const client = new HakanaiClient(baseUrl);

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

  const authToken = authTokenInput.value.trim();
  const ttl = parseInt(ttlSelect.value);
  const isFileMode = fileRadio.checked;

  let payload;
  let validationError = null;

  if (isFileMode) {
    // File mode validation and processing
    const file = fileInput.files[0];
    if (!file) {
      showError(UI_STRINGS.EMPTY_FILE);
      fileInput.focus();
      return;
    }

    // Validate file size
    if (file.size > FILE_LIMITS.MAX_SIZE) {
      showError(UI_STRINGS.FILE_TOO_LARGE);
      fileInput.focus();
      return;
    }

    // Validate file name (basic sanitization)
    const fileName = sanitizeFileName(file.name);
    if (!fileName) {
      showError(UI_STRINGS.INVALID_FILENAME);
      fileInput.focus();
      return;
    }

    try {
      const fileContent = await readFileAsBase64(file);
      payload = {
        data: fileContent,
        filename: fileName,
      };
    } catch (error) {
      showError(UI_STRINGS.FILE_READ_ERROR);
      return;
    }
  } else {
    // Text mode validation
    const secret = secretInput.value.trim();
    if (!secret) {
      showError(UI_STRINGS.EMPTY_SECRET);
      secretInput.focus();
      return;
    }
    payload = { data: secret };
  }

  // Show loading state
  loadingDiv.style.display = "block";
  button.disabled = true;
  secretInput.disabled = true;
  fileInput.disabled = true;
  authTokenInput.disabled = true;
  ttlSelect.disabled = true;
  textRadio.disabled = true;
  fileRadio.disabled = true;
  resultDiv.innerHTML = "";

  try {
    const secretUrl = await client.sendPayload(payload, ttl, authToken);

    showSuccess(secretUrl);

    // Clear the inputs
    secretInput.value = "";
    fileInput.value = "";
    updateFileInfo();
  } catch (error) {
    showError(error.message || UI_STRINGS.CREATE_FAILED);
  } finally {
    loadingDiv.style.display = "none";
    button.disabled = false;
    secretInput.disabled = false;
    fileInput.disabled = false;
    authTokenInput.disabled = false;
    ttlSelect.disabled = false;
    textRadio.disabled = false;
    fileRadio.disabled = false;
  }
}

function showSuccess(secretUrl) {
  const resultDiv = document.getElementById("result");
  resultDiv.className = "result success";
  const urlId = "url-" + Date.now();

  resultDiv.innerHTML = "";

  // Hide the form elements after successful creation
  const form = document.getElementById("create-secret-form");
  if (form) {
    form.style.display = "none";
  }

  // Create elements programmatically to avoid XSS
  const title = document.createElement("h3");
  title.textContent = UI_STRINGS.SUCCESS_TITLE;
  resultDiv.appendChild(title);

  const instructions = document.createElement("p");
  instructions.className = "share-instructions";
  instructions.textContent = UI_STRINGS.SHARE_INSTRUCTIONS;
  resultDiv.appendChild(instructions);

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

  const buttonsContainer = document.createElement("div");
  buttonsContainer.className = "buttons-container";

  const copyBtn = document.createElement("button");
  copyBtn.className = "copy-button";
  copyBtn.type = "button";
  copyBtn.textContent = UI_STRINGS.COPY_TEXT;
  copyBtn.setAttribute("aria-label", "Copy secret URL to clipboard");
  copyBtn.addEventListener("click", function () {
    copyUrl(urlId, this);
  });
  buttonsContainer.appendChild(copyBtn);

  const createAnotherBtn = document.createElement("button");
  createAnotherBtn.className = "copy-button";
  createAnotherBtn.type = "button";
  createAnotherBtn.textContent =
    i18n.t("button.createAnother") || "Create Another";
  createAnotherBtn.setAttribute("aria-label", "Create another secret");
  createAnotherBtn.addEventListener("click", function () {
    // Show the form again and clear result
    const form = document.getElementById("create-secret-form");
    const resultDiv = document.getElementById("result");
    if (form) {
      form.style.display = "block";
    }
    resultDiv.innerHTML = "";
    resultDiv.className = "";

    // Focus on the first input
    const textRadio = document.getElementById("textRadio");
    const secretText = document.getElementById("secretText");
    if (textRadio && secretText) {
      textRadio.checked = true;
      toggleSecretType();
      secretText.focus();
    }
  });
  buttonsContainer.appendChild(createAnotherBtn);

  container.appendChild(buttonsContainer);

  resultDiv.appendChild(container);

  const note = document.createElement("p");
  note.className = "secret-note";

  // Create strong element for "Note:" text
  const strong = document.createElement("strong");
  strong.textContent = i18n.t("msg.createNote").split(":")[0] + ":";
  note.appendChild(strong);

  // Add the rest of the text
  note.appendChild(document.createTextNode(" " + i18n.t("msg.createNoteText")));
  resultDiv.appendChild(note);

  // Announce to screen readers
  announceToScreenReader(UI_STRINGS.SUCCESS_TITLE);
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
  const originalText = button.textContent;

  // Get the actual URL text from input value
  const urlText = urlElement.value;

  if (navigator.clipboard && navigator.clipboard.writeText) {
    // Modern clipboard API
    navigator.clipboard
      .writeText(urlText)
      .then(() => {
        button.textContent = UI_STRINGS.COPIED_TEXT;
        button.classList.add("copied");
        announceToScreenReader(UI_STRINGS.COPIED_TEXT);
        setTimeout(() => {
          button.textContent = originalText;
          button.classList.remove("copied");
        }, TIMEOUTS.COPY_FEEDBACK);
      })
      .catch((err) => {
        // Fallback to older method
        fallbackCopy(urlText, button, originalText);
      });
  } else {
    // Fallback for older browsers
    fallbackCopy(urlText, button, originalText);
  }
}

function fallbackCopy(text, button, originalText) {
  const textArea = document.createElement("textarea");
  textArea.value = text;
  textArea.style.position = "fixed";
  textArea.style.left = "-999999px";
  textArea.style.top = "-999999px";
  document.body.appendChild(textArea);
  textArea.focus();
  textArea.select();

  try {
    document.execCommand("copy");
    button.textContent = UI_STRINGS.COPIED_TEXT;
    button.classList.add("copied");
    announceToScreenReader(UI_STRINGS.COPIED_TEXT);
    setTimeout(() => {
      button.textContent = originalText;
      button.classList.remove("copied");
    }, TIMEOUTS.COPY_FEEDBACK);
  } catch (err) {
    alert(UI_STRINGS.COPY_FAILED);
  }

  document.body.removeChild(textArea);
}

// Focus on the secret input when page loads
document.addEventListener("DOMContentLoaded", function () {
  document.getElementById("secretText").focus();
  // Initialize UI strings after i18n is loaded
  updateUIStrings();
});

// Accessibility helper
function announceToScreenReader(message) {
  const announcement = document.createElement("div");
  announcement.setAttribute("role", "status");
  announcement.setAttribute("aria-live", "polite");
  announcement.className = "sr-only";
  announcement.textContent = message;
  document.body.appendChild(announcement);

  // Remove after announcement
  setTimeout(() => {
    document.body.removeChild(announcement);
  }, 1000);
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

function readFileAsBase64(file) {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = function (e) {
      try {
        // Get base64 content without the data URL prefix
        const base64Content = e.target.result.split(",")[1];
        resolve(base64Content);
      } catch (error) {
        reject(error);
      }
    };
    reader.onerror = function () {
      reject(new Error("Failed to read file"));
    };
    reader.readAsDataURL(file);
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
      fileSizeSpan.textContent += " (Too large!)";
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
  const fileRadio = document.getElementById("fileRadio");
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

// Set up form submission handler
document.addEventListener("DOMContentLoaded", function () {
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
