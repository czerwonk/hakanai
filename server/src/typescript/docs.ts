import { initTheme } from "./common-utils.js";

function initDocs(): void {
  initTheme();
}

document.addEventListener("DOMContentLoaded", initDocs);
