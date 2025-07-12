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

    // Headers
    "header.create": "Create One-Time Secret",
    "header.get": "One-Time Secret",

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
    "label.separateKey": "Show URL and key separately for enhanced security",
    "label.filename": "Filename:",

    // Placeholders
    "placeholder.secret": "Enter your secret message here...",
    "placeholder.token": "Enter authentication token here...",

    // Helper texts
    "helper.url": "The decryption key after # is never sent to the server",
    "helper.secret":
      "Your message will be encrypted before leaving your browser",
    "helper.fileSelect":
      "Maximum file size: 10MB. File will be encrypted before upload.",
    "helper.token": "Leave empty if no authentication is required",
    "helper.key": "Base64-encoded decryption key (shared separately)",
    "helper.separateKey":
      "When enabled, the URL and decryption key are displayed separately, allowing you to share them through different channels for enhanced security.",
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

    // Messages
    "msg.creating": "Creating secret...",
    "msg.retrieving": "Retrieving secret...",
    "msg.jsRequired": "JavaScript Required",
    "msg.jsRequiredDetail":
      "This application requires JavaScript to encrypt secrets securely in your browser.",
    "msg.emptySecret": "Please enter a secret to share",
    "msg.emptyFile": "Please select a file to share",
    "msg.createFailed": "Failed to create secret",
    "msg.fileTooLarge": "File size exceeds 10MB limit",
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

    // Meta descriptions
    "meta.create":
      "Create and share one-time secrets securely with Hakanai - zero-knowledge secret sharing",
    "meta.get":
      "Retrieve your one-time secret securely with Hakanai - zero-knowledge secret sharing",

    // Error codes
    "error.SEND_FAILED": "Failed to send secret",
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

    // Headers
    "header.create": "Einmal-Secret erstellen",
    "header.get": "Einmal-Secret",

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
    "label.separateKey":
      "URL und Schlüssel separat anzeigen für erweiterte Sicherheit",
    "label.filename": "Dateiname:",

    // Placeholders
    "placeholder.secret": "Hier wird gen geheime Text eingegeben...",
    "placeholder.token": "Authentifizierungs-Token eingeben",

    // Helper texts
    "helper.url":
      "Der geheime Schlüssel nach dem # wird niemals an den Server gesendet",
    "helper.secret":
      "Die Nachricht wird verschlüsselt, bevor sie Ihren Browser verlässt",
    "helper.fileSelect":
      "Maximale Dateigröße: 10MB. Die Datei wird vor dem Upload verschlüsselt.",
    "helper.token":
      "Kann leer gelassen werden, wenn keine Authentifizierung erforderlich ist",
    "helper.key": "Base64-kodierter geheimer Schlüssel (separat geteilt)",
    "helper.separateKey":
      "Wenn aktiviert, werden URL und geheimer Schlüssel separat angezeigt, so dass sie über verschiedene Kanäle für erweiterte Sicherheit geteilt werden können.",
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

    // Messages
    "msg.creating": "Secret wird erstellt...",
    "msg.retrieving": "Secret wird abgerufen...",
    "msg.jsRequired": "JavaScript erforderlich",
    "msg.jsRequiredDetail":
      "Diese Anwendung benötigt JavaScript, um Secrets sicher in Ihrem Browser zu verschlüsseln.",
    "msg.emptySecret": "Bitte den Text für das Secret eingeben",
    "msg.emptyFile": "Bitte eine Datei zum Teilen auswählen",
    "msg.createFailed": "Fehler beim Erstellen des Secrets",
    "msg.fileTooLarge": "Dateigröße überschreitet das 10MB Limit",
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

    // Meta descriptions
    "meta.create": "One-Time-Secrets sicher erstellen und teilen mit Hakanai",
    "meta.get": "One-Time-Secrets sicher teilen mit Hakanai",

    // Error codes
    "error.SEND_FAILED": "Fehler beim Senden des Secrets",
    "error.SECRET_NOT_FOUND": "Secret nicht gefunden oder abgelaufen",
    "error.SECRET_ALREADY_ACCESSED":
      "Secret wurde bereits abgerufen und ist nicht mehr verfügbar",
    "error.RETRIEVE_FAILED": "Fehler beim Abrufen des Secrets",
    "error.MISSING_DECRYPTION_KEY": "Kein Schlüssel in der URL gefunden",
  },
};

const LANGUAGE_STORAGE_KEY = "hakanai-lang";

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

  t(key: TranslationKey): string {
    return translations[this.currentLang][key] || translations.en[key] || key;
  }

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

  getAvailableLanguages(): LanguageCode[] {
    return Object.keys(translations) as LanguageCode[];
  }

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
