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

// Theme management
const THEME_KEY = "hakanai-theme";

function isValidTheme(theme) {
  return theme && (theme === "light" || theme === "dark");
}

function currentThemeIsDark() {
  const current = document.body.getAttribute("data-theme");
  const systemPrefersDark = window.matchMedia(
    "(prefers-color-scheme: dark)",
  ).matches;

  return current === "dark" || (!current && systemPrefersDark);
}

/**
 * Get saved theme or use system preference
 */
export function getTheme() {
  try {
    const saved = localStorage.getItem(THEME_KEY);

    if (isValidTheme(saved)) {
      return saved;
    }
  } catch (error) {
    // localStorage might be disabled or throw errors in some environments
    console.warn("Failed to read theme preference:", error);
  }

  // Return null to use system preference
  return null;
}

/**
 * Apply theme to body
 */
export function applyTheme(theme) {
  if (isValidTheme(theme)) {
    document.body.setAttribute("data-theme", theme);
  } else {
    document.body.removeAttribute("data-theme");
  }
}

/**
 * Toggle between light and dark themes
 */
export function toggleTheme() {
  const newTheme = currentThemeIsDark() ? "light" : "dark";

  try {
    localStorage.setItem(THEME_KEY, newTheme);
  } catch (error) {
    // localStorage might be disabled or full - continue anyway
    console.warn("Failed to save theme preference:", error);
  }

  applyTheme(newTheme);
  updateThemeToggleButton();
}

/**
 * Update theme toggle button icon
 */
export function updateThemeToggleButton() {
  const button = document.getElementById("theme-toggle");
  if (!button) return;

  const isDark = currentThemeIsDark();

  // Show moon for dark mode, sun for light mode
  button.textContent = isDark ? "🌙" : "☀️";
  
  // Use translations if i18n is available, fallback to English
  if (typeof window.i18n !== 'undefined') {
    button.setAttribute(
      "aria-label",
      isDark ? window.i18n.t("aria.switchToLight") : window.i18n.t("aria.switchToDark"),
    );
  } else {
    button.setAttribute(
      "aria-label",
      isDark ? "Switch to light mode" : "Switch to dark mode",
    );
  }
}

/**
 * Initialize theme on page load
 */
export function initTheme() {
  const theme = getTheme();
  applyTheme(theme);

  // Create theme toggle button if it doesn't exist
  if (!document.getElementById("theme-toggle")) {
    const button = document.createElement("button");
    button.id = "theme-toggle";
    button.type = "button";
    button.addEventListener("click", toggleTheme);

    // Insert after language switcher or in header
    const languageSwitcher = document.getElementById("language-switcher");
    if (languageSwitcher && languageSwitcher.parentNode) {
      languageSwitcher.parentNode.insertBefore(button, languageSwitcher);
    } else {
      // Fallback: add to body
      document.body.appendChild(button);
    }
  } else {
    // Button exists, just add listener
    document
      .getElementById("theme-toggle")
      .addEventListener("click", toggleTheme);
  }

  updateThemeToggleButton();

  // Listen for system theme changes
  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", function () {
      // Only update if using system preference (no saved theme)
      try {
        if (!localStorage.getItem(THEME_KEY)) {
          updateThemeToggleButton();
        }
      } catch (error) {
        // localStorage access failed, update anyway
        updateThemeToggleButton();
      }
    });
}
