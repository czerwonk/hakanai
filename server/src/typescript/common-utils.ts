// Constants
const SCREEN_READER_ANNOUNCEMENT_TIMEOUT = 1000;
const COPY_FEEDBACK_TIMEOUT = 2000;
const THEME_KEY = "hakanai-theme";

type Theme = "light" | "dark";
type ButtonClickHandler = (event: MouseEvent) => void;

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

function fallbackCopyToClipboard(
  text: string,
  button: HTMLButtonElement,
  originalText: string,
  successMessage: string,
  failedMessage: string,
): void {
  // Create a temporary textarea element
  const textArea = document.createElement("textarea");
  textArea.value = text;

  // Make it invisible but functional
  textArea.style.position = "fixed";
  textArea.style.left = "-999999px";
  textArea.style.top = "-999999px";
  textArea.setAttribute("readonly", "");
  textArea.style.opacity = "0";

  document.body.appendChild(textArea);

  try {
    textArea.select();
    textArea.setSelectionRange(0, 99999); // For mobile devices

    const successful = document.execCommand("copy");
    document.body.removeChild(textArea);

    if (successful) {
      showCopySuccess(button, originalText, successMessage);
    } else {
      showCopyFailure(button, originalText, failedMessage);
    }
  } catch (err) {
    document.body.removeChild(textArea);
    showCopyFailure(button, originalText, failedMessage);
  }
}

function isModernCopySupported(): boolean {
  return (
    navigator.clipboard && typeof navigator.clipboard.writeText === "function"
  );
}

export function copyToClipboard(
  text: string,
  button: HTMLButtonElement,
  originalText: string,
  successMessage: string,
  failedMessage: string,
): void {
  if (!isModernCopySupported()) {
    fallbackCopyToClipboard(
      text,
      button,
      originalText,
      successMessage,
      failedMessage,
    );
    return;
  }

  navigator.clipboard
    .writeText(text)
    .then(() => showCopySuccess(button, originalText, successMessage))
    .catch(() => {
      // Fallback to legacy method
      fallbackCopyToClipboard(
        text,
        button,
        originalText,
        successMessage,
        failedMessage,
      );
    });
}

export function secureInputClear(input: HTMLInputElement): void {
  if (input.value.length > 0) {
    // Simple secure clear: overwrite then empty
    input.value = "x".repeat(input.value.length);
    input.value = "";
  }
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

export function announceToScreenReader(message: string): void {
  const announcement = createScreenReaderAnnouncement(message);
  document.body.appendChild(announcement);

  setTimeout(() => {
    document.body.removeChild(announcement);
  }, SCREEN_READER_ANNOUNCEMENT_TIMEOUT);
}

function createScreenReaderAnnouncement(message: string): HTMLDivElement {
  const announcement = document.createElement("div");
  announcement.setAttribute("role", "status");
  announcement.setAttribute("aria-live", "polite");
  announcement.className = "sr-only";
  announcement.textContent = message;
  return announcement;
}

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

export function getTheme(): Theme | null {
  try {
    const saved = localStorage.getItem(THEME_KEY);
    return isValidTheme(saved) ? saved : null;
  } catch (error) {
    console.warn("Failed to read theme preference:", error);
    return null;
  }
}

export function applyTheme(theme: Theme | null): void {
  if (isValidTheme(theme)) {
    document.body.setAttribute("data-theme", theme);
  } else {
    document.body.removeAttribute("data-theme");
  }
}

export function toggleTheme(): void {
  const newTheme: Theme = currentThemeIsDark() ? "light" : "dark";

  try {
    localStorage.setItem(THEME_KEY, newTheme);
  } catch (error) {
    console.warn("Failed to save theme preference:", error);
  }

  applyTheme(newTheme);
  updateThemeToggleButton();
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
        }
      } catch {
        updateThemeToggleButton();
      }
    });
}

export function initTheme(): void {
  const theme = getTheme();
  applyTheme(theme);
  setupThemeToggleButton();
  updateThemeToggleButton();
  setupSystemThemeListener();
}

// LocalStorage management for auth tokens
const AUTH_TOKEN_KEY = "hakanai-auth-token";
const TOKEN_EXPIRY_HOURS = 24;

interface StoredTokenData {
  token: string;
  expires: number;
}

export function saveAuthTokenToStorage(token: string): boolean {
  if (!token.trim()) return false;

  try {
    const expirationTime = Date.now() + TOKEN_EXPIRY_HOURS * 60 * 60 * 1000;
    const tokenData: StoredTokenData = {
      token: token,
      expires: expirationTime,
    };
    localStorage.setItem(AUTH_TOKEN_KEY, JSON.stringify(tokenData));
    return true;
  } catch (error) {
    console.warn("Failed to save auth token to localStorage:", error);
    return false;
  }
}

export function getAuthTokenFromStorage(): string | null {
  try {
    const stored = localStorage.getItem(AUTH_TOKEN_KEY);
    if (!stored) return null;

    const tokenData: StoredTokenData = JSON.parse(stored);

    // Check if token has expired
    if (Date.now() > tokenData.expires) {
      localStorage.removeItem(AUTH_TOKEN_KEY); // Clean up expired token
      return null;
    }

    return tokenData.token;
  } catch (error) {
    console.warn("Failed to read auth token from localStorage:", error);
    // Clean up corrupted data
    localStorage.removeItem(AUTH_TOKEN_KEY);
    return null;
  }
}

export function clearAuthTokenStorage(): void {
  try {
    localStorage.removeItem(AUTH_TOKEN_KEY);
  } catch (error) {
    console.warn("Failed to clear auth token from localStorage:", error);
  }
}
