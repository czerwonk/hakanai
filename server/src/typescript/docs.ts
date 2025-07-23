import { initTheme } from "./core/theme.js";

function initDocs(): void {
  initTheme();
}

document.addEventListener("DOMContentLoaded", initDocs);
