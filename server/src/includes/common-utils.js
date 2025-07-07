// Common utility functions shared between create-secret.js and get-secret.js

/**
 * Helper function to create buttons with consistent styling and behavior
 */
export function createButton(className, text, ariaLabel, clickHandler) {
  const button = document.createElement("button");
  button.className = className;
  button.type = "button";
  button.textContent = text;
  button.setAttribute("aria-label", ariaLabel);
  button.addEventListener("click", clickHandler);
  return button;
}

/**
 * Helper function to create button container
 */
export function createButtonContainer() {
  const container = document.createElement("div");
  container.className = "buttons-container";
  return container;
}

/**
 * Copy text to clipboard with fallback for older browsers
 */
export function copyToClipboard(
  text,
  button,
  originalText,
  successMessage,
  failedMessage,
) {
  if (navigator.clipboard && navigator.clipboard.writeText) {
    // Modern clipboard API
    navigator.clipboard
      .writeText(text)
      .then(() => {
        button.textContent = successMessage;
        button.classList.add("copied");
        announceToScreenReader(successMessage);
        setTimeout(() => {
          button.textContent = originalText;
          button.classList.remove("copied");
        }, COPY_FEEDBACK_TIMEOUT);
      })
      .catch(() => {
        // Fallback to older method
        fallbackCopy(text, button, originalText, successMessage, failedMessage);
      });
  } else {
    // Fallback for older browsers
    fallbackCopy(text, button, originalText, successMessage, failedMessage);
  }
}

/**
 * Fallback copy method for older browsers
 */
function fallbackCopy(
  text,
  button,
  originalText,
  successMessage,
  failedMessage,
) {
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
    button.textContent = successMessage;
    button.classList.add("copied");
    announceToScreenReader(successMessage);
    setTimeout(() => {
      button.textContent = originalText;
      button.classList.remove("copied");
    }, COPY_FEEDBACK_TIMEOUT);
  } catch (error) {
    alert(failedMessage);
  }

  document.body.removeChild(textArea);
}

// Constants
const SCREEN_READER_ANNOUNCEMENT_TIMEOUT = 1000;
const COPY_FEEDBACK_TIMEOUT = 2000;

/**
 * Announce messages to screen readers for accessibility
 */
export function announceToScreenReader(message) {
  const announcement = document.createElement("div");
  announcement.setAttribute("role", "status");
  announcement.setAttribute("aria-live", "polite");
  announcement.className = "sr-only";
  announcement.textContent = message;
  document.body.appendChild(announcement);

  // Remove after announcement
  setTimeout(() => {
    document.body.removeChild(announcement);
  }, SCREEN_READER_ANNOUNCEMENT_TIMEOUT);
}

/**
 * Debounce function to limit the rate of function calls
 */
export function debounce(func, wait) {
  let timeout;
  return function executedFunction(...args) {
    const later = () => {
      clearTimeout(timeout);
      func(...args);
    };
    clearTimeout(timeout);
    timeout = setTimeout(later, wait);
  };
}
