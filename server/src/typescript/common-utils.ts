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
 */
export function copyToClipboard(text: string, button: HTMLButtonElement): void {
  const originalText = button.textContent || "Copy";
  navigator.clipboard
    .writeText(text)
    .then(() =>
      showCopySuccess(button, originalText, window.i18n?.t("button.copied")),
    )
    .catch(() =>
      showCopyFailure(button, originalText, window.i18n?.t("msg.copyFailed")),
    );
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

// QR Code Generation Utilities

/**
 * QR Code generator using WebAssembly
 */
export class QRCodeGenerator {
  private static wasmModule: any = null;
  private static loadPromise: Promise<void> | null = null;

  /**
   * Ensure WASM module is loaded (loads once, cached for reuse)
   */
  static async ensureWasmLoaded(): Promise<void> {
    if (this.loadPromise) return this.loadPromise;

    this.loadPromise = this.loadWasm();
    return this.loadPromise;
  }

  /**
   * Load the WASM QR code module
   */
  private static async loadWasm(): Promise<void> {
    try {
      // TODO: Replace with actual WASM module loading when implemented
      // For now, we'll simulate the interface for development
      console.debug("QR code WASM module would be loaded here");

      // Simulate successful load for development
      this.wasmModule = {
        generate_qr_svg: (url: string) => {
          // Placeholder implementation - will be replaced with real WASM
          return `<svg width="100" height="100" xmlns="http://www.w3.org/2000/svg">
            <rect width="100" height="100" fill="white"/>
            <text x="50" y="50" text-anchor="middle" dy=".3em" font-size="8">QR CODE</text>
            <text x="50" y="65" text-anchor="middle" dy=".3em" font-size="6">(PLACEHOLDER)</text>
          </svg>`;
        },
      };
    } catch (error) {
      console.warn("Failed to load QR code WASM module:", error);
      throw error;
    }
  }

  /**
   * Generate QR code SVG for the given URL
   * @param url - URL to encode in QR code
   * @returns SVG string or null if generation failed
   */
  static generateQRCode(url: string): string | null {
    if (!this.wasmModule) return null;

    try {
      return this.wasmModule.generate_qr_svg(url);
    } catch (error) {
      console.warn("QR code generation failed:", error);
      return null;
    }
  }
}

// Unified Success Display Components

/**
 * Options for success result display
 */
interface SuccessDisplayOptions {
  separateKeyMode?: boolean;
  container: HTMLElement;
}

declare global {
  interface Window {
    i18n: {
      t(key: string): string;
    };
  }
}

/**
 * Display unified success result with URL, QR code, and security note
 * @param url - The secret URL to display
 * @param options - Configuration options
 */
export function displaySuccessResult(
  url: string,
  options: SuccessDisplayOptions,
): void {
  const { container } = options;

  container.className = "result success";
  container.innerHTML = "";

  // 1. Success header with instructions - ALWAYS shown
  createSuccessHeader(container);

  // 2. URL display (with separate key support)
  createUrlSection(container, url, options.separateKeyMode);

  // 3. QR Code section - ALWAYS attempted
  createQRCodeSection(container, url);

  // 4. Note section - ALWAYS shown for security
  createNoteSection(container);
}

/**
 * Create success header with title and instructions
 */
function createSuccessHeader(container: HTMLElement): void {
  const title = document.createElement("h3");
  title.textContent =
    window.i18n?.t("msg.successTitle") || "Secret Created Successfully";
  container.appendChild(title);

  const instructions = document.createElement("p");
  instructions.className = "share-instructions";
  instructions.textContent =
    window.i18n?.t("msg.shareInstructions") ||
    "Share this URL with the intended recipient. The secret is encrypted and can only be accessed once.";
  container.appendChild(instructions);
}

/**
 * Create URL display section with copy functionality
 */
function createUrlSection(
  container: HTMLElement,
  url: string,
  separateKeyMode?: boolean,
): void {
  const urlContainer = document.createElement("div");
  urlContainer.className = "url-container";

  if (separateKeyMode) {
    createSeparateUrlDisplay(urlContainer, url);
  } else {
    createCombinedUrlDisplay(urlContainer, url);
  }

  container.appendChild(urlContainer);
}

/**
 * Create combined URL display (traditional mode)
 */
function createCombinedUrlDisplay(container: HTMLElement, url: string): void {
  const urlId = generateRandomId();

  const label = document.createElement("label");
  label.textContent = window.i18n?.t("label.secretUrl") || "Secret URL:";
  label.setAttribute("for", urlId);
  container.appendChild(label);

  const inputContainer = document.createElement("div");
  inputContainer.className = "input-group";

  const urlInput = document.createElement("input");
  urlInput.type = "text";
  urlInput.id = urlId;
  urlInput.value = url;
  urlInput.readOnly = true;
  urlInput.className = "url-input";
  inputContainer.appendChild(urlInput);

  const copyButton = createButton(
    "copy-button",
    window.i18n?.t("button.copy") || "Copy URL",
    "Copy secret URL to clipboard",
    () => copyToClipboardByElementId(urlId, copyButton as HTMLButtonElement),
  );
  inputContainer.appendChild(copyButton);

  container.appendChild(inputContainer);
}

/*
 * Generate a unique ID to be used for dynamic elements like URL inputs
 */
export function generateRandomId(): string {
  return crypto?.randomUUID && typeof crypto.randomUUID === "function"
    ? `url-${crypto.randomUUID()}`
    : `url-${Date.now()}-${Math.random()}`;
}

/**
 * Create separate URL and key display (enhanced security mode)
 */
function createSeparateUrlDisplay(
  container: HTMLElement,
  fullUrl: string,
): void {
  const [url, key] = fullUrl.split("#");
  const id = generateRandomId();
  const urlId = id;
  const keyId = id + "-key";

  // URL section
  const urlLabel = document.createElement("label");
  urlLabel.textContent = window.i18n?.t("label.secretUrl") || "Secret URL:";
  urlLabel.setAttribute("for", urlId);
  container.appendChild(urlLabel);

  const urlInputContainer = document.createElement("div");
  urlInputContainer.className = "input-group";

  const urlInput = document.createElement("input");
  urlInput.type = "text";
  urlInput.id = urlId;
  urlInput.value = url;
  urlInput.readOnly = true;
  urlInput.className = "url-input";
  urlInputContainer.appendChild(urlInput);

  const urlCopyButton = createButton(
    "copy-button",
    window.i18n?.t("button.copy") || "Copy URL",
    "Copy secret URL to clipboard",
    () =>
      copyToClipboardByElementId(
        urlInput.id,
        urlCopyButton as HTMLButtonElement,
      ),
  );
  urlInputContainer.appendChild(urlCopyButton);
  container.appendChild(urlInputContainer);

  // Key section
  const keyLabel = document.createElement("label");
  keyLabel.textContent =
    window.i18n?.t("label.decryptionKey") || "Decryption Key:";
  keyLabel.setAttribute("for", keyId);
  container.appendChild(keyLabel);

  const keyInputContainer = document.createElement("div");
  keyInputContainer.className = "input-group";

  const keyInput = document.createElement("input");
  keyInput.type = "text";
  keyInput.id = keyId;
  keyInput.value = key;
  keyInput.readOnly = true;
  keyInput.className = "url-input";
  keyInputContainer.appendChild(keyInput);

  const keyCopyButton = createButton(
    "copy-button",
    window.i18n?.t("button.copy") || "Copy Key",
    "Copy decryption key to clipboard",
    () =>
      copyToClipboardByElementId(
        keyInput.id,
        keyCopyButton as HTMLButtonElement,
      ),
  );
  keyInputContainer.appendChild(keyCopyButton);
  container.appendChild(keyInputContainer);
}

function copyToClipboardByElementId(
  elementId: string,
  button: HTMLButtonElement,
): void {
  const input = document.getElementById(elementId) as HTMLInputElement;
  if (input) {
    copyToClipboard(input.value, button);
  } else {
    showCopyFailure(
      button,
      button.textContent || "Copy",
      window.i18n?.t("msg.copyFailed"),
    );
  }
}

/**
 * Create QR code section (with graceful degradation)
 */
async function createQRCodeSection(
  container: HTMLElement,
  url: string,
): Promise<void> {
  try {
    await QRCodeGenerator.ensureWasmLoaded();
    const qrSvg = QRCodeGenerator.generateQRCode(url);

    if (qrSvg) {
      const qrSection = createQRDisplayElement(qrSvg);
      container.appendChild(qrSection);
    }
  } catch (error) {
    // Silent graceful degradation - no UI indication needed
    // QR code simply doesn't appear if WASM fails
    console.debug("QR code not available:", error);
  }
}

/**
 * Create QR code display element
 */
function createQRDisplayElement(qrSvg: string): HTMLElement {
  const qrSection = document.createElement("div");
  qrSection.className = "qr-code-section";

  const qrLabel = document.createElement("label");
  qrLabel.textContent = window.i18n?.t("label.qrCode") || "QR Code:";
  qrSection.appendChild(qrLabel);

  const qrContainer = document.createElement("div");
  qrContainer.className = "qr-code-container";
  qrContainer.innerHTML = qrSvg;
  qrSection.appendChild(qrContainer);

  return qrSection;
}

/**
 * Create security note section
 */
function createNoteSection(container: HTMLElement): void {
  const note = document.createElement("p");
  note.className = "secret-note";

  const noteText =
    window.i18n?.t("msg.createNote") ||
    "Note: Share this URL carefully. The secret will be deleted after the first access or when it expires.";
  const colonIndex = noteText.indexOf(":");

  if (colonIndex > 0) {
    const strong = document.createElement("strong");
    strong.textContent = noteText.substring(0, colonIndex + 1);
    note.appendChild(strong);

    const remaining = document.createTextNode(
      noteText.substring(colonIndex + 1),
    );
    note.appendChild(remaining);
  } else {
    note.textContent = noteText;
  }

  container.appendChild(note);
}
