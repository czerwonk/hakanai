type TranslationKey = string;
type LanguageCode = "en" | "de";

interface TranslationDictionary {
  [key: TranslationKey]: string;
}

type Translations = {
  [lang in LanguageCode]: TranslationDictionary;
};

const translations: Translations = {
  en: {
    // Page titles
    "page.create.title": "Hakanai - Create Secret",
    "page.get.title": "Hakanai - Retrieve Secret",
    "page.homepage.title": "Hakanai - One-Time Secret Sharing",
    "page.share.title": "Hakanai - Share from Clipboard",

    // Headers
    "header.create": "Create One-Time Secret",
    "header.get": "One-Time Secret",
    "header.homepage": "One-Time Secret Sharing",
    "header.share": "Share from Clipboard",

    // Labels
    "label.secret": "Secret message:",
    "label.secretType": "Secret Type:",
    "label.text": "Text Message",
    "label.file": "File",
    "label.fileSelect": "Select file to share:",
    "label.token": "Token:",
    "label.expires": "Expires after:",
    "label.url": "Secret URL:",
    "label.key": "Decryption Key:",
    "label.separateKey": "Show Key separately",
    "label.saveToken": "Remember authentication token",
    "label.filename": "Filename:",
    "label.size": "Size:",
    "label.expiresIn": "Expires in:",
    "label.contentPreview": "Content Preview",

    // Placeholders
    "placeholder.secret": "Enter your secret message here...",
    "placeholder.token": "Enter authentication token here...",

    // Helper texts
    "helper.url": "The decryption key after # is never sent to the server",
    "helper.secret":
      "Your message will be encrypted before leaving your browser",
    "helper.fileSelect": "File will be encrypted before upload.",
    "helper.token": "Leave empty if no authentication is required",
    "helper.key": "Base64-encoded decryption key (shared separately)",
    "helper.separateKey":
      "When enabled, the URL and decryption key are displayed separately, allowing you to share them through different channels for enhanced security.",
    "helper.saveToken":
      "Token will be stored securely in your browser for the current session only. You will need to re-enter it when you start a new browser session.",
    "helper.expires": "Secret will self-destruct after this time or first view",

    // Time options
    "time.5min": "5 minutes",
    "time.30min": "30 minutes",
    "time.1hour": "1 hour",
    "time.2hours": "2 hours",
    "time.12hours": "12 hours",
    "time.24hours": "24 hours",
    "time.7days": "7 days",

    // Buttons
    "button.create": "Create Secret",
    "button.retrieve": "Retrieve Secret",
    "button.copy": "Copy",
    "button.copied": "Copied!",
    "button.copyUrl": "Copy URL",
    "button.copyKey": "Copy Key",
    "button.download": "Download",
    "button.createAnother": "Create Another",
    "button.chooseFile": "📁 Choose File",
    "button.readClipboard": "Read Clipboard",
    "button.createSecret": "Create Secret",

    // Messages
    "msg.creating": "Creating secret...",
    "msg.retrieving": "Retrieving secret...",
    "msg.jsRequired": "JavaScript Required",
    "msg.jsRequiredDetail":
      "This application requires JavaScript to encrypt secrets securely in your browser.",
    "msg.emptySecret": "Please enter a secret to share",
    "msg.emptyFile": "Please select a file to share",
    "msg.createFailed": "Failed to create secret",
    "msg.fileReadError": "Error reading file",
    "msg.invalidFilename":
      "Invalid filename. Please select a file with a valid name.",
    "msg.emptyUrl": "Please enter a valid secret URL",
    "msg.invalidUrl":
      "Invalid URL format. Please include the full URL with the secret key after #",
    "msg.missingKey": "Please enter the decryption key",
    "msg.retrieveFailed": "Failed to retrieve secret",
    "msg.successTitle": "Success",
    "msg.errorTitle": "Error",
    "msg.copyFailed": "Failed to copy. Please select and copy manually.",
    "msg.createNote":
      "Note: Share this URL carefully. The secret will be deleted after the first access or when it expires.",
    "msg.createNoteText":
      "Share this URL carefully. The secret will be deleted after the first access or when it expires.",
    "msg.shareInstructions":
      "Share this URL with the intended recipient. The secret is encrypted and can only be accessed once.",
    "msg.clipboardError": "Clipboard Error",
    "msg.clipboardRequired": "Clipboard Access Required",
    "msg.clipboardRequiredDetail":
      "Click the button below to read the shared content from your clipboard.",
    "msg.clipboardPermissionDenied":
      "Clipboard access denied. Please grant permission and try again.",
    "msg.clipboardInvalidJson": "Clipboard does not contain valid JSON",
    "msg.clipboardEmpty": "Clipboard is empty",
    "msg.readingClipboard": "Reading clipboard...",
    "msg.creatingSecret": "Creating secret...",
    "msg.shareSuccess":
      "Your secret has been created and the URL copied to clipboard:",
    "msg.expectedJsonFormat": "Expected JSON format:",
    "msg.retrieveNote":
      "Note: This secret has been deleted from the server and cannot be accessed again.",
    "msg.retrieveNoteText":
      "This secret has been deleted from the server and cannot be accessed again.",
    "msg.downloaded": "Secret downloaded as text file",
    "msg.binaryDetected": "Use download button to save the file.",

    // Aria labels
    "aria.copySecret": "Copy secret to clipboard",
    "aria.downloadSecret": "Download secret as file",
    "aria.secretInput": "Enter the secret message you want to share securely",
    "aria.fileInput":
      "Select a file to share securely. The file will be encrypted before being sent.",
    "aria.tokenInput":
      "Enter the authentication token if required by the server",
    "aria.expiresSelect":
      "Select how long the secret should be available before it expires",
    "aria.urlInput":
      "Enter the full URL including the secret key after the hash",
    "aria.keyInput": "Enter the base64-encoded decryption key",
    "aria.themeToggle": "Switch between light and dark mode",
    "aria.switchToLight": "Switch to light mode",
    "aria.switchToDark": "Switch to dark mode",
    "aria.logoHome": "Go to home page",

    // Meta descriptions
    "meta.create":
      "Create and share one-time secrets securely with Hakanai - zero-knowledge secret sharing",
    "meta.get":
      "Retrieve your one-time secret securely with Hakanai - zero-knowledge secret sharing",
    "meta.homepage": "Hakanai - Zero-knowledge one-time secret sharing service",

    // Homepage content
    "homepage.tagline": "Share secrets securely with zero-knowledge encryption",
    "homepage.create.description":
      "Share text messages or files securely. All encryption happens in your browser.",
    "homepage.create.button": "Create Secret",
    "homepage.retrieve.description":
      "Have a secret URL? Enter it here to decrypt and view your one-time secret.",
    "homepage.retrieve.button": "Retrieve Secret",
    "homepage.how.feature1.title": "Zero-Knowledge",
    "homepage.how.feature1.description":
      "Your secrets are encrypted in your browser before being sent",
    "homepage.how.feature2.title": "One-Time",
    "homepage.how.feature2.description":
      "Secrets are destroyed after being viewed once",
    "homepage.how.feature3.title": "Secure",
    "homepage.how.feature3.description":
      "AES-256-GCM encryption with secure key generation",
    "homepage.how.feature4.title": "Private",
    "homepage.how.feature4.description":
      "The server never sees your unencrypted data",
    "homepage.docs.link": "View API Documentation",

    // Footer links
    "footer.privacy": "Privacy Policy",

    // Page privacy
    "page.privacy.title": "Privacy Policy",

    // Error codes
    "error.SEND_FAILED": "Failed to send secret",
    "error.AUTHENTICATION_REQUIRED":
      "Authentication required - Please enter your authentication token",
    "error.INVALID_TOKEN":
      "Invalid authentication token - Please check your token and try again",
    "error.SECRET_NOT_FOUND": "Secret not found or has expired",
    "error.SECRET_ALREADY_ACCESSED":
      "Secret has been accessed and is no longer available",
    "error.RETRIEVE_FAILED": "Failed to retrieve secret",
    "error.MISSING_DECRYPTION_KEY": "No decryption key found in URL",
  },
  de: {
    // Page titles
    "page.create.title": "Hakanai - Secret erstellen",
    "page.get.title": "Hakanai - Secret abrufen",
    "page.homepage.title": "Hakanai - Einmal-Secret-Sharing",
    "page.share.title": "Hakanai - Aus Zwischenablage teilen",

    // Headers
    "header.create": "Einmal-Secret erstellen",
    "header.get": "Einmal-Secret",
    "header.homepage": "Einmal-Secret-Sharing",
    "header.share": "Aus Zwischenablage teilen",

    // Labels
    "label.secret": "Text:",
    "label.secretType": "Secret-Typ:",
    "label.text": "Text-Nachricht",
    "label.file": "Datei",
    "label.fileSelect": "Datei zum Teilen auswählen:",
    "label.token": "Token:",
    "label.expires": "Läuft ab nach:",
    "label.url": "Secret-URL:",
    "label.key": "Geheimer Schlüssel:",
    "label.separateKey": "Schlüssel separat anzeigen",
    "label.saveToken": "Token merken",
    "label.filename": "Dateiname:",
    "label.size": "Größe:",
    "label.expiresIn": "Läuft ab in:",
    "label.contentPreview": "Inhaltsvorschau",

    // Placeholders
    "placeholder.secret": "Hier wird gen geheime Text eingegeben...",
    "placeholder.token": "Authentifizierungs-Token eingeben",

    // Helper texts
    "helper.url":
      "Der geheime Schlüssel nach dem # wird niemals an den Server gesendet",
    "helper.secret":
      "Die Nachricht wird verschlüsselt, bevor sie den Browser verlässt",
    "helper.fileSelect": "Die Datei wird vor dem Upload verschlüsselt.",
    "helper.token":
      "Kann leer gelassen werden, wenn keine Authentifizierung erforderlich ist",
    "helper.key": "Base64-kodierter geheimer Schlüssel (separat geteilt)",
    "helper.separateKey":
      "Wenn aktiviert, werden URL und geheimer Schlüssel separat angezeigt, so dass sie über verschiedene Kanäle für erweiterte Sicherheit geteilt werden können.",
    "helper.saveToken":
      "Token wird sicher für die Session im Browser gespeichert. Nach dem Schließen des Tabs muss dieses neu eigegeben werden.",
    "helper.expires":
      "Das Secret wird nach dieser Zeit oder beim ersten Zugriff selbst zerstört",

    // Time options
    "time.5min": "5 Minuten",
    "time.30min": "30 Minuten",
    "time.1hour": "1 Stunde",
    "time.2hours": "2 Stunden",
    "time.12hours": "12 Stunden",
    "time.24hours": "24 Stunden",
    "time.7days": "7 Tage",

    // Buttons
    "button.create": "Secret erstellen",
    "button.retrieve": "Secret abrufen",
    "button.copy": "Kopieren",
    "button.copied": "Kopiert!",
    "button.copyUrl": "URL kopieren",
    "button.copyKey": "Schlüssel kopieren",
    "button.download": "Herunterladen",
    "button.createAnother": "Neues Secret erstellen",
    "button.chooseFile": "📁 Datei auswählen",
    "button.readClipboard": "Zwischenablage lesen",
    "button.createSecret": "Secret erstellen",

    // Messages
    "msg.creating": "Secret wird erstellt...",
    "msg.retrieving": "Secret wird abgerufen...",
    "msg.jsRequired": "JavaScript erforderlich",
    "msg.jsRequiredDetail":
      "Diese Anwendung benötigt JavaScript, um Secrets sicher im Browser zu verschlüsseln.",
    "msg.emptySecret": "Bitte den Text für das Secret eingeben",
    "msg.emptyFile": "Bitte eine Datei zum Teilen auswählen",
    "msg.createFailed": "Fehler beim Erstellen des Secrets",
    "msg.fileReadError": "Fehler beim Lesen der Datei",
    "msg.invalidFilename":
      "Ungültiger Dateiname. Bitte eine Datei mit einem gültigen Namen auswählen.",
    "msg.emptyUrl": "Bitte eine gültige Secret-URL eingeben",
    "msg.invalidUrl":
      "Ungültiges URL-Format. Bitte vollständige URL einschließlich des Teils nach dem # eingeben",
    "msg.missingKey": "Bitte den geheimen Schlüssel eingeben",
    "msg.retrieveFailed": "Fehler beim Abrufen des Secrets",
    "msg.successTitle": "Erfolg",
    "msg.errorTitle": "Fehler",
    "msg.copyFailed":
      "Kopieren fehlgeschlagen. Bitte manuell auswählen und kopieren.",
    "msg.createNote":
      "Hinweis: Das Secret wird nach dem ersten Zugriff oder bei Ablauf gelöscht.",
    "msg.createNoteText":
      "Bitte Vorsicht beim Teilen der URL. Das Secret wird nach dem ersten Zugriff oder bei Ablauf gelöscht.",
    "msg.shareInstructions":
      "Diese URL kann nun mit dem vorgesehenen Empfänger geteilt werden. Das Secret ist verschlüsselt und kann nur einmal abgerufen werden.",
    "msg.clipboardError": "Zwischenablage-Fehler",
    "msg.clipboardRequired": "Zwischenablage-Zugriff erforderlich",
    "msg.clipboardRequiredDetail":
      "Bitte den Button klicken, um den geteilten Inhalt aus der Zwischenablage zu lesen.",
    "msg.clipboardPermissionDenied":
      "Zwischenablage-Zugriff verweigert. Bitte Berechtigung erteilen und erneut versuchen.",
    "msg.clipboardInvalidJson": "Zwischenablage enthält kein gültiges JSON",
    "msg.clipboardEmpty": "Zwischenablage ist leer",
    "msg.readingClipboard": "Zwischenablage wird gelesen...",
    "msg.creatingSecret": "Secret wird erstellt...",
    "msg.shareSuccess":
      "Das Secret wurde erstellt und die URL in die Zwischenablage kopiert:",
    "msg.expectedJsonFormat": "Erwartetes JSON-Format:",
    "msg.retrieveNote":
      "Hinweis: Dieses Secret wurde vom Server gelöscht und kann nicht erneut abgerufen werden.",
    "msg.retrieveNoteText":
      "Dieses Secret wurde vom Server gelöscht und kann nicht erneut abgerufen werden.",
    "msg.downloaded": "Secret als Textdatei heruntergeladen",
    "msg.binaryDetected":
      "Bitte Download-Button verwenden, um die Datei zu speichern.",

    // Aria labels
    "aria.copySecret": "Secret in die Zwischenablage kopieren",
    "aria.downloadSecret": "Secret als Datei herunterladen",
    "aria.secretInput":
      "Bitte die geheime Nachricht eingeben, die sicher geteult werden soll",
    "aria.fileInput":
      "Datei zum sicheren Teilen auswählen. Die Datei wird vor dem Versenden verschlüsselt.",
    "aria.tokenInput":
      "Bitte den Authentifizierungs-Token eingeben, falls vom Server erforderlich",
    "aria.expiresSelect":
      "Bitte die Zeit auswählen, nach der das Secret abläuft",
    "aria.urlInput":
      "Bitte die vollständige URL einschließlich des Schlüssels nach dem Hash eingeben",
    "aria.keyInput": "Bitte den Base64-kodierten geheimen Schlüssel eingeben",
    "aria.themeToggle": "Zwischen hellem und dunklem Modus wechseln",
    "aria.switchToLight": "Zum hellen Modus wechseln",
    "aria.switchToDark": "Zum dunklen Modus wechseln",
    "aria.logoHome": "Zur Startseite gehen",

    // Meta descriptions
    "meta.create": "One-Time-Secrets sicher erstellen und teilen mit Hakanai",
    "meta.get": "One-Time-Secrets sicher teilen mit Hakanai",
    "meta.homepage": "Hakanai - Zero-Knowledge One-Time Secret-Sharing Service",

    // Homepage content
    "homepage.tagline":
      "Secrets sicher teilen mit Zero-Knowledge-Verschlüsselung",
    "homepage.create.description":
      "Textnachrichten oder Dateien sicher teilen. Die Verschlüsselung erfolgt im Browser.",
    "homepage.create.button": "Secret erstellen",
    "homepage.retrieve.description":
      "Hier kann der Secret-Link eingegeben werden, um das Einmal-Secret zu entschlüsseln und anzuzeigen.",
    "homepage.retrieve.button": "Secret abrufen",
    "homepage.how.feature1.title": "Zero-Knowledge",
    "homepage.how.feature1.description":
      "Die Secrets werden im Browser verschlüsselt, bevor sie gesendet werden",
    "homepage.how.feature2.title": "Einmalig",
    "homepage.how.feature2.description":
      "Secrets werden nach dem ersten Abruf gelöscht",
    "homepage.how.feature3.title": "Sicher",
    "homepage.how.feature3.description":
      "AES-256-GCM-Verschlüsselung mit sicherer Schlüsselerzeugung",
    "homepage.how.feature4.title": "Privat",
    "homepage.how.feature4.description":
      "Der Server sieht niemals die unverschlüsselten Daten",
    "homepage.docs.link": "API-Dokumentation anzeigen",

    // Footer links
    "footer.privacy": "Datenschutzerklärung",

    // Page privacy
    "page.privacy.title": "Datenschutzerklärung",

    // Error codes
    "error.SEND_FAILED": "Fehler beim Senden des Secrets",
    "error.AUTHENTICATION_REQUIRED":
      "Authentifizierung erforderlich - Bitte Authentifizierungs-Token eingeben",
    "error.INVALID_TOKEN":
      "Ungültiges Authentifizierungs-Token - Bitte Token überprüfen und erneut versuchen",
    "error.SECRET_NOT_FOUND": "Secret nicht gefunden oder abgelaufen",
    "error.SECRET_ALREADY_ACCESSED":
      "Secret wurde bereits abgerufen und ist nicht mehr verfügbar",
    "error.RETRIEVE_FAILED": "Fehler beim Abrufen des Secrets",
    "error.MISSING_DECRYPTION_KEY": "Kein Schlüssel in der URL gefunden",
  },
};

const LANGUAGE_STORAGE_KEY = "hakanai-lang";

/**
 * Internationalization system for multi-language support
 * @class I18n
 */
class I18n {
  private currentLang: LanguageCode = "en";

  constructor() {
    this.init();
  }

  private init(): void {
    this.currentLang = this.detectLanguage();
    this.applyTranslations();
    this.setupLanguageSwitcher();
    this.dispatchInitializedEvent();
  }

  private detectLanguage(): LanguageCode {
    const savedLang = this.getStoredLanguage();
    if (savedLang) return savedLang;

    const browserLang = navigator.language.substring(0, 2) as LanguageCode;
    return this.isValidLanguage(browserLang) ? browserLang : "en";
  }

  private getStoredLanguage(): LanguageCode | null {
    try {
      const saved = localStorage.getItem(LANGUAGE_STORAGE_KEY);
      return this.isValidLanguage(saved) ? (saved as LanguageCode) : null;
    } catch {
      return null;
    }
  }

  private isValidLanguage(lang: unknown): lang is LanguageCode {
    return lang === "en" || lang === "de";
  }

  /**
   * Get translated text for a given key
   * @param key - Translation key to look up
   * @returns Translated text in current language, English fallback, or key itself
   */
  t(key: TranslationKey): string {
    return translations[this.currentLang][key] || translations.en[key] || key;
  }

  /**
   * Set the current language
   * @param lang - Language code to switch to (en, de)
   */
  setLanguage(lang: string): void {
    if (this.isValidLanguage(lang)) {
      this.currentLang = lang;
      this.storeLanguage(lang);
      this.applyTranslations();
    }
  }

  private storeLanguage(lang: LanguageCode): void {
    try {
      localStorage.setItem(LANGUAGE_STORAGE_KEY, lang);
    } catch (error) {
      console.warn("Failed to save language preference:", error);
    }
  }

  private applyTranslations(): void {
    this.translateElements();
    this.updatePageMetadata();
    this.dispatchLanguageChangeEvent();
  }

  private translateElements(): void {
    this.translateTextContent();
    this.translateHtmlContent();
    this.translatePlaceholders();
    this.translateAriaLabels();
  }

  private translateTextContent(): void {
    document.querySelectorAll<HTMLElement>("[data-i18n]").forEach((element) => {
      const key = element.getAttribute("data-i18n");
      if (key) {
        element.textContent = this.t(key);
        element.classList.add("i18n-loaded");
      }
    });
  }

  private translateHtmlContent(): void {
    document
      .querySelectorAll<HTMLElement>("[data-i18n-html]")
      .forEach((element) => {
        const key = element.getAttribute("data-i18n-html");
        if (key) {
          element.innerHTML = this.t(key);
          element.classList.add("i18n-loaded");
        }
      });
  }

  private translatePlaceholders(): void {
    document
      .querySelectorAll<
        HTMLInputElement | HTMLTextAreaElement
      >("[data-i18n-placeholder]")
      .forEach((element) => {
        const key = element.getAttribute("data-i18n-placeholder");
        if (key) {
          element.placeholder = this.t(key);
          element.classList.add("i18n-loaded");
        }
      });
  }

  private translateAriaLabels(): void {
    document
      .querySelectorAll<HTMLElement>("[data-i18n-aria-label]")
      .forEach((element) => {
        const key = element.getAttribute("data-i18n-aria-label");
        if (key) {
          element.setAttribute("aria-label", this.t(key));
          element.classList.add("i18n-loaded");
        }
      });

    // Also handle data-i18n-aria attributes
    document
      .querySelectorAll<HTMLElement>("[data-i18n-aria]")
      .forEach((element) => {
        const key = element.getAttribute("data-i18n-aria");
        if (key) {
          element.setAttribute("aria-label", this.t(key));
          element.classList.add("i18n-loaded");
        }
      });
  }

  private updatePageMetadata(): void {
    this.updatePageTitle();
    this.updateMetaDescription();
    this.updateDocumentLanguage();
  }

  private updatePageTitle(): void {
    const titleElement =
      document.querySelector<HTMLElement>("[data-i18n-title]");
    if (titleElement) {
      const key = titleElement.getAttribute("data-i18n-title");
      if (key) {
        document.title = this.t(key);
      }
    }
  }

  private updateMetaDescription(): void {
    const metaDesc = document.querySelector<HTMLMetaElement>(
      'meta[name="description"][data-i18n-content]',
    );
    if (metaDesc) {
      const key = metaDesc.getAttribute("data-i18n-content");
      if (key) {
        metaDesc.content = this.t(key);
      }
    }
  }

  private updateDocumentLanguage(): void {
    document.documentElement.lang = this.currentLang;
  }

  private dispatchLanguageChangeEvent(): void {
    document.dispatchEvent(
      new CustomEvent("languageChanged", {
        detail: { language: this.currentLang },
      }),
    );
  }

  private dispatchInitializedEvent(): void {
    document.dispatchEvent(
      new CustomEvent("i18nInitialized", {
        detail: { language: this.currentLang },
      }),
    );
  }

  private setupLanguageSwitcher(): void {
    const switcher = document.getElementById(
      "language-switcher",
    ) as HTMLSelectElement | null;
    if (!switcher) return;

    this.populateLanguageOptions(switcher);
    this.addLanguageChangeListener(switcher);
  }

  private populateLanguageOptions(switcher: HTMLSelectElement): void {
    switcher.innerHTML = "";

    this.getAvailableLanguages().forEach((lang) => {
      const option = this.createLanguageOption(lang);
      switcher.appendChild(option);
    });
  }

  private createLanguageOption(lang: LanguageCode): HTMLOptionElement {
    const option = document.createElement("option");
    option.value = lang;
    option.textContent = lang.toUpperCase();
    option.selected = lang === this.currentLang;
    return option;
  }

  private addLanguageChangeListener(switcher: HTMLSelectElement): void {
    switcher.addEventListener("change", (e) => {
      const target = e.target as HTMLSelectElement;
      this.setLanguage(target.value);
    });
  }

  /**
   * Get list of available language codes
   * @returns Array of supported language codes
   */
  getAvailableLanguages(): LanguageCode[] {
    return Object.keys(translations) as LanguageCode[];
  }

  /**
   * Get the current active language
   * @returns Current language code
   */
  getCurrentLanguage(): LanguageCode {
    return this.currentLang;
  }
}

// Initialize i18n system
const initializeI18n = (): void => {
  const i18n = new I18n();
  (window as any).i18n = i18n;
};

if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", initializeI18n);
} else {
  initializeI18n();
}

// Note: No exports needed for browser usage - i18n is attached to window.i18n
// Exports below are for testing purposes only and will be removed in compiled JS
export {
  I18n,
  translations,
  type LanguageCode,
  type TranslationKey,
  type Translations,
};
