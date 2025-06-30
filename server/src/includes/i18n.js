// Lightweight internationalization system for Hakanai
(function () {
  "use strict";

  // Translation dictionaries
  const translations = {
    en: {
      // Page titles
      "page.create.title": "Hakanai - Create Secret",
      "page.get.title": "Hakanai - Retrieve Secret",

      // Headers
      "header.create": "Create Secret",
      "header.get": "One Time Secret Retrieval",

      // Labels and placeholders
      "label.secret": "Secret message:",
      "placeholder.secret": "Enter your secret message here...",
      "label.token": "Token:",
      "placeholder.token":
        "Enter authentication token (leave empty if none required)",
      "label.expires": "Expires after:",
      "label.url": "Enter URL:",

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
      "button.download": "Download",

      // Messages
      "msg.creating": "Creating secret...",
      "msg.retrieving": "Retrieving secret...",
      "msg.jsRequired": "JavaScript Required",
      "msg.jsRequiredDetail":
        "This application requires JavaScript to encrypt secrets securely in your browser.",
      "msg.emptySecret": "Please enter a secret to share",
      "msg.createFailed": "Failed to create secret",
      "msg.emptyUrl": "Please enter a valid secret URL",
      "msg.invalidUrl":
        "Invalid URL format. Please include the full URL with the secret key after #",
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

      // Aria labels
      "aria.secretInput": "Enter the secret message you want to share securely",
      "aria.tokenInput":
        "Enter the authentication token if required by the server",
      "aria.expiresSelect":
        "Select how long the secret should be available before it expires",
      "aria.urlInput":
        "Enter the full URL including the secret key after the hash",

      // Links
      "link.github": "View on GitHub",

      // Meta descriptions
      "meta.create":
        "Create and share one-time secrets securely with Hakanai - zero-knowledge secret sharing",
      "meta.get":
        "Retrieve your one-time secret securely with Hakanai - zero-knowledge secret sharing",
    },
    de: {
      // Page titles
      "page.create.title": "Hakanai - Secret erstellen",
      "page.get.title": "Hakanai - Secret abrufen",

      // Headers
      "header.create": "Einmal-Secret erstellen",
      "header.get": "Einmal-Secret abrufen",

      // Labels and placeholders
      "label.secret": "Text:",
      "placeholder.secret": "Hier wird gen geheime Text eingegeben...",
      "label.token": "Token:",
      "placeholder.token":
        "Authentifizierungs-Token eingeben (leer lassen, falls nicht erforderlich)",
      "label.expires": "Läuft ab nach:",
      "label.url": "URL eingeben:",

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
      "button.download": "Herunterladen",

      // Messages
      "msg.creating": "Secret wird erstellt...",
      "msg.retrieving": "Secret wird abgerufen...",
      "msg.jsRequired": "JavaScript erforderlich",
      "msg.jsRequiredDetail":
        "Diese Anwendung benötigt JavaScript, um Secrets sicher in Ihrem Browser zu verschlüsseln.",
      "msg.emptySecret": "Bitte den Text für das Secret eingeben",
      "msg.createFailed": "Fehler beim Erstellen des Secrets",
      "msg.emptyUrl": "Bitte eine gültige Secret-URL eingeben",
      "msg.invalidUrl":
        "Ungültiges URL-Format. Bitte geben Sie die vollständige URL einschließlich des Teils nach dem # ein",
      "msg.retrieveFailed": "Fehler beim Abrufen des Secrets",
      "msg.successTitle": "Erfolg",
      "msg.errorTitle": "Fehler",
      "msg.copyFailed":
        "Kopieren fehlgeschlagen. Bitte manuell auswählen und kopieren.",
      "msg.createNote":
        "Hinweis: Das Secret wird nach dem ersten Zugriff oder bei Ablauf gelöscht.",
      "msg.createNoteText":
        "Teilen Sie diese URL vorsichtig. Das Secret wird nach dem ersten Zugriff oder bei Ablauf gelöscht.",
      "msg.shareInstructions":
        "Diese URL kann nun mit dem vorgesehenen Empfänger geteilt werden. Das Secret ist verschlüsselt und kann nur einmal abgerufen werden.",
      "msg.retrieveNote":
        "Hinweis: Dieses Secret wurde vom Server gelöscht und kann nicht erneut abgerufen werden.",
      "msg.retrieveNoteText":
        "Dieses Secret wurde vom Server gelöscht und kann nicht erneut abgerufen werden.",
      "msg.downloaded": "Secret als Textdatei heruntergeladen",

      // Aria labels
      "aria.secretInput":
        "Bitte die geheime Nachricht eingeben, die sicher geteult werden soll",
      "aria.tokenInput":
        "Bitte den Authentifizierungs-Token eingeben, falls vom Server erforderlich",
      "aria.expiresSelect":
        "Bitte die Zeit auswählen, nach der das Secret abläuft",
      "aria.urlInput":
        "Bitte die vollständige URL einschließlich des Secret-Schlüssels nach dem Hash eingeben",

      // Links
      "link.github": "Auf GitHub ansehen",

      // Meta descriptions
      "meta.create": "One-Time-Secrets sicher erstellen und teilen mit Hakanai",
      "meta.get": "One-Time-Secrets sicher teilen mit Hakanai",
    },
  };

  // Language detection and management
  const i18n = {
    currentLang: "en",

    // Initialize i18n system
    init() {
      // Try to get language from localStorage
      const savedLang = localStorage.getItem("hakanai-lang");
      if (savedLang && translations[savedLang]) {
        this.currentLang = savedLang;
      } else {
        // Detect browser language
        const browserLang = navigator.language.substring(0, 2);
        if (translations[browserLang]) {
          this.currentLang = browserLang;
        }
      }

      // Apply translations immediately
      this.applyTranslations();

      // Set up language switcher if present
      this.setupLanguageSwitcher();
    },

    // Get a translation by key
    t(key) {
      return (
        translations[this.currentLang][key] || translations["en"][key] || key
      );
    },

    // Change language
    setLanguage(lang) {
      if (translations[lang]) {
        this.currentLang = lang;
        localStorage.setItem("hakanai-lang", lang);
        this.applyTranslations();
      }
    },

    // Apply translations to the DOM
    applyTranslations() {
      // Update all elements with data-i18n attribute
      document.querySelectorAll("[data-i18n]").forEach((element) => {
        const key = element.getAttribute("data-i18n");
        element.textContent = this.t(key);
      });

      // Update all elements with data-i18n-placeholder attribute
      document
        .querySelectorAll("[data-i18n-placeholder]")
        .forEach((element) => {
          const key = element.getAttribute("data-i18n-placeholder");
          element.placeholder = this.t(key);
        });

      // Update all elements with data-i18n-aria-label attribute
      document.querySelectorAll("[data-i18n-aria-label]").forEach((element) => {
        const key = element.getAttribute("data-i18n-aria-label");
        element.setAttribute("aria-label", this.t(key));
      });

      // Update page title
      const titleElement = document.querySelector("[data-i18n-title]");
      if (titleElement) {
        document.title = this.t(titleElement.getAttribute("data-i18n-title"));
      }

      // Update meta description
      const metaDesc = document.querySelector(
        'meta[name="description"][data-i18n-content]',
      );
      if (metaDesc) {
        metaDesc.content = this.t(metaDesc.getAttribute("data-i18n-content"));
      }

      // Update document language
      document.documentElement.lang = this.currentLang;

      // Trigger custom event for dynamic content
      document.dispatchEvent(
        new CustomEvent("languageChanged", {
          detail: { language: this.currentLang },
        }),
      );
    },

    // Set up language switcher
    setupLanguageSwitcher() {
      const switcher = document.getElementById("language-switcher");
      if (switcher) {
        // Clear existing options
        switcher.innerHTML = "";

        // Add language options
        Object.keys(translations).forEach((lang) => {
          const option = document.createElement("option");
          option.value = lang;
          option.textContent = lang.toUpperCase();
          if (lang === this.currentLang) {
            option.selected = true;
          }
          switcher.appendChild(option);
        });

        // Add change listener
        switcher.addEventListener("change", (e) => {
          this.setLanguage(e.target.value);
        });
      }
    },

    // Get all available languages
    getAvailableLanguages() {
      return Object.keys(translations);
    },
  };

  // Initialize when DOM is ready
  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", () => i18n.init());
  } else {
    i18n.init();
  }

  // Expose i18n to global scope
  window.i18n = i18n;
})();
