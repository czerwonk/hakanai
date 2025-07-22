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
 * Format TTL seconds as human readable string
 * @param seconds - Duration in seconds
 * @returns Human readable duration string
 */
export function formatTTL(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const days = Math.floor(hours / 24);

  if (days > 0) {
    return `${days} day${days > 1 ? "s" : ""}`;
  } else if (hours > 0) {
    return `${hours} hour${hours > 1 ? "s" : ""}`;
  } else {
    const minutes = Math.floor(seconds / 60);
    return `${minutes} minute${minutes > 1 ? "s" : ""}`;
  }
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
        }
      } catch {
        updateThemeToggleButton();
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

/**
 * Application configuration interface
 */
interface AppConfig {
  features: {
    impressum: boolean;
    privacy: boolean;
  };
}

/**
 * Fetch application configuration from server
 */
async function fetchAppConfig(): Promise<AppConfig | null> {
  try {
    const response = await fetch("/config.json");
    if (!response.ok) {
      console.warn("Failed to fetch app config:", response.status);
      return null;
    }
    return await response.json();
  } catch (error) {
    console.warn("Failed to fetch app config:", error);
    return null;
  }
}

/**
 * Initialize UI based on application configuration
 */
async function initializeUI(): Promise<void> {
  const config = await fetchAppConfig();
  await initializeOptionalFeature(
    "impressum-link",
    config?.features.impressum ?? false,
  );
  await initializeOptionalFeature(
    "privacy-link",
    config?.features.privacy ?? false,
  );
}

async function initializeOptionalFeature(
  elementId: string,
  enabled: boolean,
): Promise<void> {
  const element = document.getElementById(elementId);
  if (element) {
    if (enabled) {
      element.style.display = "";
    } else {
      element.style.display = "none";
    }
  }
}

// Auto-initialize when DOM is ready
document.addEventListener("DOMContentLoaded", async () => {
  await initializeUI();
});

/**
 * Sanitize filename by removing invalid characters and limiting length
 * @param fileName - Original filename to sanitize
 * @returns Sanitized filename or null if invalid
 */
export function sanitizeFileName(fileName: string): string | null {
  const sanitized = fileName
    .replace(/[<>:"/\\|?*\x00-\x1f]/g, "_")
    .replace(/^\.+/, "")
    .substring(0, 255)
    .trim();

  return sanitized.length > 0 ? sanitized : null;
}

/**
 * Validation error codes for ShareData
 */
export enum ShareDataValidationError {
  MISSING_DATA = "MISSING_DATA",
  INVALID_FILENAME = "INVALID_FILENAME",
  INVALID_TOKEN = "INVALID_TOKEN",
  INVALID_TTL = "INVALID_TTL",
  EMPTY_JSON = "EMPTY_JSON",
  INVALID_JSON_FORMAT = "INVALID_JSON_FORMAT",
}

/**
 * Custom error class for ShareData validation
 */
export class ShareDataError extends Error {
  constructor(
    public readonly code: ShareDataValidationError,
    message: string,
  ) {
    super(message);
    this.name = "ShareDataError";
  }
}

/**
 * Share data structure for clipboard and fragment-based sharing
 */
export class ShareData {
  constructor(
    public readonly data: string, // base64-encoded content
    public readonly filename?: string,
    public readonly token?: string,
    public readonly ttl?: number,
  ) {
    this.validate();
  }

  /**
   * Validate the share data
   * @throws Error if validation fails
   */
  private validate(): void {
    // Validate required fields
    if (!this.data || typeof this.data !== "string") {
      throw new ShareDataError(
        ShareDataValidationError.MISSING_DATA,
        'Missing or invalid "data" field',
      );
    }

    // Validate optional fields
    if (this.filename !== undefined && typeof this.filename !== "string") {
      throw new ShareDataError(
        ShareDataValidationError.INVALID_FILENAME,
        'Invalid "filename" field - must be string',
      );
    }

    if (this.token !== undefined && typeof this.token !== "string") {
      throw new ShareDataError(
        ShareDataValidationError.INVALID_TOKEN,
        'Invalid "token" field - must be string',
      );
    }

    if (
      this.ttl !== undefined &&
      (typeof this.ttl !== "number" || this.ttl <= 0 || isNaN(this.ttl))
    ) {
      throw new ShareDataError(
        ShareDataValidationError.INVALID_TTL,
        'Invalid "ttl" field - must be positive number',
      );
    }
  }

  /**
   * Create ShareData from JSON string (clipboard content)
   * @param jsonString JSON string containing share data
   * @returns ShareData instance
   * @throws Error if JSON is invalid or validation fails
   */
  static fromJSON(jsonString: string): ShareData {
    if (!jsonString.trim()) {
      throw new ShareDataError(
        ShareDataValidationError.EMPTY_JSON,
        "JSON string is empty",
      );
    }

    let payload;
    try {
      payload = JSON.parse(jsonString);
    } catch (error) {
      throw new ShareDataError(
        ShareDataValidationError.INVALID_JSON_FORMAT,
        "Invalid JSON format",
      );
    }

    return new ShareData(
      payload.data,
      payload.filename,
      payload.token,
      payload.ttl,
    );
  }

  /**
   * Create ShareData from URL fragment parameters
   * @param fragment URL fragment (without #)
   * @returns ShareData instance or null if no data found
   * @throws Error if validation fails
   */
  static fromFragment(fragment: string): ShareData | null {
    if (!fragment) return null;

    const params = new URLSearchParams(fragment);
    const data = params.get("data");

    if (!data) return null;

    return new ShareData(
      data,
      params.get("filename") || undefined,
      params.get("token") || undefined,
      params.get("ttl") ? parseInt(params.get("ttl")!) : undefined,
    );
  }

  /**
   * Calculate content size in bytes from base64 data
   */
  getContentSize(): number {
    return Math.ceil((this.data.length * 3) / 4);
  }
}
