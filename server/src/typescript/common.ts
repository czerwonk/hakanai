/**
 * Common functionality for static pages (homepage, impressum, privacy)
 * Contains shared theme and i18n components
 */
import { initI18n } from "./core/i18n";
import { initTheme } from "./core/theme";
import { initFeatures } from "./core/app-config";

document.addEventListener("DOMContentLoaded", () => {
  initI18n();
  initTheme();
  initFeatures();
});
