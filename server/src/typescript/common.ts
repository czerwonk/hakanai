/**
 * Common functionality for static pages (homepage, impressum, privacy)
 * Contains shared theme and i18n components
 */

import { I18n } from "./core/i18n";
import { initTheme } from "./core/theme";

// Re-export core functionality needed by static pages
export { I18n, initTheme };

/**
 * Initialize common functionality for static pages
 * This function should be called on pages that need basic theme and i18n support
 */
export function initCommon(): void {
  // Initialize theme system
  initTheme();

  // I18n is automatically initialized via its module-level code
  // No additional initialization needed here
}

// Auto-initialize for direct script inclusion
document.addEventListener("DOMContentLoaded", () => {
  initCommon();
});

