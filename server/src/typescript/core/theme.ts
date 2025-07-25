// Constants
const THEME_KEY = "hakanai-theme";

import { I18nKeys } from "./i18n";

type Theme = "light" | "dark";

declare global {
  interface Window {
    i18n: {
      t(key: string): string;
    };
  }
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
  return isDark
    ? window.i18n.t(I18nKeys.Aria.SwitchToLight)
    : window.i18n.t(I18nKeys.Aria.SwitchToDark);
}

/**
 * Update theme toggle button appearance and accessibility
 */
export function updateThemeToggleButton(): void {
  const button = getThemeToggleButton();
  if (!button) return;

  const isDark = currentThemeIsDark();
  button.textContent = isDark ? "☀️" : "🌙";
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
        if (getTheme() === null) {
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
