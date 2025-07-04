// Listen for language changes to update dynamic content
document.addEventListener("languageChanged", function (e) {
  updateUIStrings();
});

function updateUIStrings() {
  UI_STRINGS.EMPTY_SECRET = i18n.t("msg.emptySecret");
  UI_STRINGS.CREATE_FAILED = i18n.t("msg.createFailed");
  UI_STRINGS.SUCCESS_TITLE = i18n.t("msg.successTitle");
  UI_STRINGS.ERROR_TITLE = i18n.t("msg.errorTitle");
  UI_STRINGS.COPY_TEXT = i18n.t("button.copy");
  UI_STRINGS.COPIED_TEXT = i18n.t("button.copied");
  UI_STRINGS.COPY_FAILED = i18n.t("msg.copyFailed");
  UI_STRINGS.NOTE_TEXT = i18n.t("msg.createNote");
  UI_STRINGS.SHARE_INSTRUCTIONS = i18n.t("msg.shareInstructions");
}

const UI_STRINGS = {
  EMPTY_SECRET: "Please enter a secret to share",
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
};

const TIMEOUTS = {
  COPY_FEEDBACK: 2000,
};

// Extract base URL from current location or use a default
const baseUrl = window.location.origin.includes("file://")
  ? "http://localhost:8080"
  : window.location.origin;

const client = new HakanaiClient(baseUrl);

async function createSecret() {
  const secretInput = document.getElementById("secretText");
  const authTokenInput = document.getElementById("authToken");
  const ttlSelect = document.getElementById("ttlSelect");
  const resultDiv = document.getElementById("result");
  const loadingDiv = document.getElementById("loading");
  const button = document.getElementById("createBtn");

  const secret = secretInput.value.trim();
  const authToken = authTokenInput.value.trim();
  const ttl = parseInt(ttlSelect.value);

  if (!secret) {
    showError(UI_STRINGS.EMPTY_SECRET);
    secretInput.focus();
    return;
  }

  // Show loading state
  loadingDiv.style.display = "block";
  button.disabled = true;
  secretInput.disabled = true;
  authTokenInput.disabled = true;
  ttlSelect.disabled = true;
  resultDiv.innerHTML = "";

  try {
    const secretUrl = await client.sendSecret(secret, ttl, authToken);

    showSuccess(secretUrl);

    // Clear the input
    secretInput.value = "";
  } catch (error) {
    showError(error.message || UI_STRINGS.CREATE_FAILED);
  } finally {
    loadingDiv.style.display = "none";
    button.disabled = false;
    secretInput.disabled = false;
    authTokenInput.disabled = false;
    ttlSelect.disabled = false;
  }
}

function showSuccess(secretUrl) {
  const resultDiv = document.getElementById("result");
  resultDiv.className = "result success";
  const urlId = "url-" + Date.now();

  resultDiv.innerHTML = "";

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

// Set up form submission handler
document.addEventListener("DOMContentLoaded", function () {
  const form = document.querySelector("form");
  if (form) {
    form.addEventListener("submit", function (event) {
      event.preventDefault();
      createSecret();
    });
  }
});

