/**
 * Common functionality for static pages (homepage, impressum, privacy)
 * Contains shared theme and i18n components
 */
import { initI18n } from "./core/i18n";
import { initTheme } from "./core/theme";

/**
 * Initialize common functionality for static pages
 * This function should be called on pages that need basic theme and i18n support
 */
export function initCommon(): void {
  // Initialize theme system
  initTheme();
}

// Auto-initialize for direct script inclusion
document.addEventListener("DOMContentLoaded", () => {
  initI18n();
  initCommon();
});
