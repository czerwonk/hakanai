// SPDX-License-Identifier: Apache-2.0

/**
 * Common functionality for static pages (homepage, impressum, privacy)
 * Contains shared theme and i18n components
 */
import { initI18n } from "./core/i18n";
import { initTheme } from "./core/theme";
import { initFeatures } from "./core/app-config";
import { registerServiceWorker } from "./service-worker";

document.addEventListener("DOMContentLoaded", async () => {
  initI18n();
  initTheme();
  initFeatures();

  await registerServiceWorker();
});
