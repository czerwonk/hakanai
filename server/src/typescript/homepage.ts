import { I18n } from "./i18n.js";
import { initTheme } from "./core/theme.js";

class HomePage {
  private readonly i18n: I18n;

  constructor() {
    this.i18n = new I18n();
  }

  async init(): Promise<void> {
    initTheme();
  }
}

document.addEventListener("DOMContentLoaded", async () => {
  const homePage = new HomePage();
  await homePage.init();
});
