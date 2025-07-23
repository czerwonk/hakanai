// Constants
const THEME_KEY = "hakanai-theme";
// Theme management
function isValidTheme(theme) {
    return theme === "light" || theme === "dark";
}
function getSystemPrefersDark() {
    return window.matchMedia("(prefers-color-scheme: dark)").matches;
}
function getCurrentTheme() {
    const current = document.body.getAttribute("data-theme");
    return isValidTheme(current) ? current : null;
}
function currentThemeIsDark() {
    const current = getCurrentTheme();
    return current === "dark" || (!current && getSystemPrefersDark());
}
/**
 * Get the saved theme preference from localStorage
 * @returns Saved theme or null if not set/invalid
 */
export function getTheme() {
    try {
        const saved = localStorage.getItem(THEME_KEY);
        return isValidTheme(saved) ? saved : null;
    }
    catch (error) {
        console.warn("Failed to read theme preference:", error);
        return null;
    }
}
/**
 * Apply theme to the document body
 * @param theme - Theme to apply or null for system default
 */
export function applyTheme(theme) {
    if (isValidTheme(theme)) {
        document.body.setAttribute("data-theme", theme);
    }
    else {
        document.body.removeAttribute("data-theme");
    }
}
/**
 * Toggle between light and dark theme
 */
export function toggleTheme() {
    const newTheme = currentThemeIsDark() ? "light" : "dark";
    try {
        localStorage.setItem(THEME_KEY, newTheme);
    }
    catch (error) {
        console.warn("Failed to save theme preference:", error);
    }
    applyTheme(newTheme);
    updateThemeToggleButton();
}
function getThemeToggleButton() {
    return document.getElementById("theme-toggle");
}
function getThemeToggleLabel(isDark) {
    var _a;
    if ((_a = window.i18n) === null || _a === void 0 ? void 0 : _a.t) {
        return isDark
            ? window.i18n.t("aria.switchToLight")
            : window.i18n.t("aria.switchToDark");
    }
    return isDark ? "Switch to light mode" : "Switch to dark mode";
}
/**
 * Update theme toggle button appearance and accessibility
 */
export function updateThemeToggleButton() {
    const button = getThemeToggleButton();
    if (!button)
        return;
    const isDark = currentThemeIsDark();
    button.textContent = isDark ? "☀️" : "🌙";
    button.setAttribute("aria-label", getThemeToggleLabel(isDark));
}
function createThemeToggleButton() {
    const button = document.createElement("button");
    button.id = "theme-toggle";
    button.type = "button";
    button.addEventListener("click", toggleTheme);
    return button;
}
function insertThemeToggleButton(button) {
    const languageSwitcher = document.getElementById("language-switcher");
    if (languageSwitcher === null || languageSwitcher === void 0 ? void 0 : languageSwitcher.parentNode) {
        languageSwitcher.parentNode.insertBefore(button, languageSwitcher);
    }
    else {
        document.body.appendChild(button);
    }
}
function setupThemeToggleButton() {
    const existingButton = getThemeToggleButton();
    if (existingButton) {
        existingButton.addEventListener("click", toggleTheme);
    }
    else {
        const button = createThemeToggleButton();
        insertThemeToggleButton(button);
    }
}
function setupSystemThemeListener() {
    window
        .matchMedia("(prefers-color-scheme: dark)")
        .addEventListener("change", () => {
        try {
            if (getTheme() === null) {
                updateThemeToggleButton();
            }
        }
        catch (_a) {
            updateThemeToggleButton();
        }
    });
}
/**
 * Initialize theme system with saved preference and listeners
 */
export function initTheme() {
    const theme = getTheme();
    applyTheme(theme);
    setupThemeToggleButton();
    updateThemeToggleButton();
    setupSystemThemeListener();
}
