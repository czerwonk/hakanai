// Constants
const SCREEN_READER_ANNOUNCEMENT_TIMEOUT = 1000;
const COPY_FEEDBACK_TIMEOUT = 2000;
const THEME_KEY = "hakanai-theme";

type Theme = "light" | "dark";
type ButtonClickHandler = (event: MouseEvent) => void;

/**
 * Format file size in human readable format
 * @param bytes - File size in bytes
 * @returns Formatted file size string (e.g., "1.5 MB")
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

/**
 * Create a button element with consistent styling and accessibility
 * @param className - CSS class for the button
 * @param text - Button text content
 * @param ariaLabel - Accessible label for screen readers
 * @param clickHandler - Click event handler
 * @returns Configured button element
 */
export function createButton(
  className: string,
  text: string,
  ariaLabel: string,
  clickHandler: ButtonClickHandler,
): HTMLButtonElement {
  const button = document.createElement("button");
  button.className = className;
  button.type = "button";
  button.textContent = text;
  button.setAttribute("aria-label", ariaLabel);
  button.addEventListener("click", clickHandler);
  return button;
}

export function createButtonContainer(): HTMLDivElement {
  const container = document.createElement("div");
  container.className = "buttons-container";
  return container;
}

/**
 * Copy text to clipboard with visual feedback
 * @param text - Text to copy to clipboard
 * @param button - Button element to show feedback on
 * @param originalText - Original button text to restore
 * @param successMessage - Message to show on successful copy
 * @param failedMessage - Message to announce on failure
 */
export function copyToClipboard(
  text: string,
  button: HTMLButtonElement,
  originalText: string,
  successMessage: string,
  failedMessage: string,
): void {
  navigator.clipboard
    .writeText(text)
    .then(() => showCopySuccess(button, originalText, successMessage))
    .catch(() => showCopyFailure(button, originalText, failedMessage));
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
  button.textContent = "Failed";
  button.classList.add("copy-failed");
  announceToScreenReader(failedMessage);

  setTimeout(() => {
    button.textContent = originalText;
    button.classList.remove("copy-failed");
  }, COPY_FEEDBACK_TIMEOUT);
}

/**
 * Securely clear sensitive input by overwriting with dummy data
 * @param input - HTML input element containing sensitive data
 */
export function secureInputClear(input: HTMLInputElement): void {
  if (input.value.length > 0) {
    const length = input.value.length;
    // Multiple overwrite passes
    for (let i = 0; i < 3; i++) {
      input.value = Array(length)
        .fill(0)
        .map(() => String.fromCharCode(Math.floor(Math.random() * 256)))
        .join("");
    }
    input.value = "";
  }
}

/**
 * Announce a message to screen readers using ARIA live regions
 * @param message - Message to announce
 */
export function announceToScreenReader(message: string): void {
  const announcement = createScreenReaderAnnouncement(message);
  document.body.appendChild(announcement);

  setTimeout(() => {
    document.body.removeChild(announcement);
  }, SCREEN_READER_ANNOUNCEMENT_TIMEOUT);
}

/**
 * Update banner images based on current theme
 * Light theme uses banner.svg (solid background)
 * Dark theme uses banner-transparent.svg (transparent background)
 */
export function updateBannerForTheme(): void {
  const bannerImages = document.querySelectorAll('.page-banner, .homepage-banner');
  const isDark = currentThemeIsDark();
  
  bannerImages.forEach((img) => {
    if (img instanceof HTMLImageElement) {
      if (isDark) {
        img.src = '/banner-transparent.svg';
      } else {
        img.src = '/banner.svg';
      }
    }
  });
}

function createScreenReaderAnnouncement(message: string): HTMLDivElement {
  const announcement = document.createElement("div");
  announcement.setAttribute("role", "status");
  announcement.setAttribute("aria-live", "polite");
  announcement.className = "sr-only";
  announcement.textContent = message;
  return announcement;
}

/**
 * Create a debounced version of a function
 * @template T - Function type to debounce
 * @param func - Function to debounce
 * @param wait - Milliseconds to wait before calling
 * @returns Debounced function
 */
export function debounce<T extends (...args: any[]) => void>(
  func: T,
  wait: number,
): (...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout> | null = null;

  return function executedFunction(...args: Parameters<T>): void {
    if (timeout) clearTimeout(timeout);
    timeout = setTimeout(() => func(...args), wait);
  };
}

// Theme management
function isValidTheme(theme: unknown): theme is Theme {
  return theme === "light" || theme === "dark";
}

function getSystemPrefersDark(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function getCurrentTheme(): Theme | null {
  const current = document.body.getAttribute("data-theme");
  return isValidTheme(current) ? current : null;
}

function currentThemeIsDark(): boolean {
  const current = getCurrentTheme();
  return current === "dark" || (!current && getSystemPrefersDark());
}

/**
 * Get the saved theme preference from localStorage
 * @returns Saved theme or null if not set/invalid
 */
export function getTheme(): Theme | null {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    return isValidTheme(saved) ? saved : null;
  } catch (error) {
    console.warn("Failed to read theme preference:", error);
    return null;
  }
}

/**
 * Apply theme to the document body
 * @param theme - Theme to apply or null for system default
 */
export function applyTheme(theme: Theme | null): void {
  if (isValidTheme(theme)) {
    document.body.setAttribute("data-theme", theme);
  } else {
    document.body.removeAttribute("data-theme");
  }
}

/**
 * Toggle between light and dark theme
 */
export function toggleTheme(): void {
  const newTheme: Theme = currentThemeIsDark() ? "light" : "dark";

  try {
    localStorage.setItem(THEME_KEY, newTheme);
  } catch (error) {
    console.warn("Failed to save theme preference:", error);
  }

  applyTheme(newTheme);
  updateThemeToggleButton();
  updateBannerForTheme();
}

function getThemeToggleButton(): HTMLButtonElement | null {
  return document.getElementById("theme-toggle") as HTMLButtonElement | null;
}

function getThemeToggleLabel(isDark: boolean): string {
  const i18n = (window as any).i18n;

  if (i18n?.t) {
    return isDark ? i18n.t("aria.switchToLight") : i18n.t("aria.switchToDark");
  }

  return isDark ? "Switch to light mode" : "Switch to dark mode";
}

/**
 * Update theme toggle button appearance and accessibility
 */
export function updateThemeToggleButton(): void {
  const button = getThemeToggleButton();
  if (!button) return;

  const isDark = currentThemeIsDark();
  button.textContent = isDark ? "🌙" : "☀️";
  button.setAttribute("aria-label", getThemeToggleLabel(isDark));
}

function createThemeToggleButton(): HTMLButtonElement {
  const button = document.createElement("button");
  button.id = "theme-toggle";
  button.type = "button";
  button.addEventListener("click", toggleTheme);
  return button;
}

function insertThemeToggleButton(button: HTMLButtonElement): void {
  const languageSwitcher = document.getElementById("language-switcher");

  if (languageSwitcher?.parentNode) {
    languageSwitcher.parentNode.insertBefore(button, languageSwitcher);
  } else {
    document.body.appendChild(button);
  }
}

function setupThemeToggleButton(): void {
  const existingButton = getThemeToggleButton();

  if (existingButton) {
    existingButton.addEventListener("click", toggleTheme);
  } else {
    const button = createThemeToggleButton();
    insertThemeToggleButton(button);
  }
}

function setupSystemThemeListener(): void {
  window
    .matchMedia("(prefers-color-scheme: dark)")
    .addEventListener("change", () => {
      try {
        if (!localStorage.getItem(THEME_KEY)) {
          updateThemeToggleButton();
          updateBannerForTheme();
        }
      } catch {
        updateThemeToggleButton();
        updateBannerForTheme();
      }
    });
}

/**
 * Initialize theme system with saved preference and listeners
 */
export function initTheme(): void {
  const theme = getTheme();
  applyTheme(theme);
  setupThemeToggleButton();
  updateThemeToggleButton();
  updateBannerForTheme();
  setupSystemThemeListener();
}

// SessionStorage management for auth tokens
const AUTH_TOKEN_KEY = "hakanai-auth-token";

/**
 * Save authentication token to session storage
 * @param token - Authentication token to save
 * @returns True if saved successfully, false otherwise
 * @description Token persists only for current browser session
 */
export function saveAuthTokenToStorage(token: string): boolean {
  if (!token.trim()) return false;

  try {
    sessionStorage.setItem(AUTH_TOKEN_KEY, token);
    return true;
  } catch (error) {
    console.warn("Failed to save auth token to sessionStorage:", error);
    return false;
  }
}

/**
 * Retrieve authentication token from session storage
 * @returns Stored token or null if not found/error
 */
export function getAuthTokenFromStorage(): string | null {
  try {
    return sessionStorage.getItem(AUTH_TOKEN_KEY);
  } catch (error) {
    console.warn("Failed to read auth token from sessionStorage:", error);
    return null;
  }
}

/**
 * Clear authentication token from session storage
 */
export function clearAuthTokenStorage(): void {
  try {
    sessionStorage.removeItem(AUTH_TOKEN_KEY);
  } catch (error) {
    console.warn("Failed to clear auth token from sessionStorage:", error);
  }
}
