// SPDX-License-Identifier: Apache-2.0

const I18nKeys = {
  Page: {
    CreateTitle: "page.create.title",
    GetTitle: "page.get.title",
    HomepageTitle: "page.homepage.title",
    PrivacyTitle: "page.privacy.title",
    ShareTitle: "page.share.title",
  },

  Label: {
    AllowedIPs: "label.allowedIPs",
    AllowedCountries: "label.allowedCountries",
    AllowedASNs: "label.allowedASNs",
    ContentPreview: "label.contentPreview",
    Expires: "label.expires",
    ExpiresIn: "label.expiresIn",
    File: "label.file",
    FileSelect: "label.fileSelect",
    Filename: "label.filename",
    Key: "label.key",
    RestrictAccess: "label.restrictAccess",
    SaveToken: "label.saveToken",
    Secret: "label.secret",
    SecretType: "label.secretType",
    SeparateKey: "label.separateKey",
    Size: "label.size",
    Text: "label.text",
    Token: "label.token",
    Url: "label.url",
    Passphrase: "label.passphrase",
    PassphraseInput: "label.passphraseInput",
  },

  Placeholder: {
    Passphrase: "placeholder.passphrase",
    Secret: "placeholder.secret",
    Token: "placeholder.token",
  },

  Helper: {
    AllowedIPs: "helper.allowedIPs",
    AllowedCountries: "helper.allowedCountries",
    AllowedASNs: "helper.allowedASNs",
    Expires: "helper.expires",
    FileSelect: "helper.fileSelect",
    Key: "helper.key",
    RestrictAccess: "helper.restrictAccess",
    SaveToken: "helper.saveToken",
    Secret: "helper.secret",
    SeparateKey: "helper.separateKey",
    Token: "helper.token",
    Url: "helper.url",
    Passphrase: "helper.passphrase",
    PassphraseInput: "helper.passphraseInput",
  },

  Time: {
    Custom: "time.custom",
    Days: "time.days",
    FiveMin: "time.5min",
    Hours: "time.hours",
    Minutes: "time.minutes",
    OneHour: "time.1hour",
    Seconds: "time.seconds",
    SevenDays: "time.7days",
    ThirtyMin: "time.30min",
    TwelveHours: "time.12hours",
    TwentyFourHours: "time.24hours",
    TwoHours: "time.2hours",
  },

  Button: {
    ChooseFile: "button.chooseFile",
    Close: "button.close",
    Copied: "button.copied",
    Copy: "button.copy",
    Create: "button.create",
    Download: "button.download",
    ReadClipboard: "button.readClipboard",
    Retrieve: "button.retrieve",
    ShowQrCode: "button.showQrCode",
  },

  Dropzone: {
    Primary: "dropzone.primary",
    Secondary: "dropzone.secondary",
    Helper: "dropzone.helper",
  },

  Msg: {
    BinaryDetected: "msg.binaryDetected",
    ClipboardEmpty: "msg.clipboardEmpty",
    ClipboardError: "msg.clipboardError",
    ClipboardInvalidJson: "msg.clipboardInvalidJson",
    ClipboardPermissionDenied: "msg.clipboardPermissionDenied",
    ClipboardRequired: "msg.clipboardRequired",
    ClipboardRequiredDetail: "msg.clipboardRequiredDetail",
    CopyFailed: "msg.copyFailed",
    CreateFailed: "msg.createFailed",
    CreateNote: "msg.createNote",
    Creating: "msg.creating",
    CreatingSecret: "msg.creatingSecret",
    Downloaded: "msg.downloaded",
    EmptyFile: "msg.emptyFile",
    EmptySecret: "msg.emptySecret",
    EmptyUrl: "msg.emptyUrl",
    ErrorTitle: "msg.errorTitle",
    ExpectedJsonFormat: "msg.expectedJsonFormat",
    FileReadError: "msg.fileReadError",
    InvalidFilename: "msg.invalidFilename",
    InvalidUrl: "msg.invalidUrl",
    JsRequired: "msg.jsRequired",
    JsRequiredDetail: "msg.jsRequiredDetail",
    MissingKey: "msg.missingKey",
    ReadingClipboard: "msg.readingClipboard",
    RetrieveFailed: "msg.retrieveFailed",
    RetrieveNote: "msg.retrieveNote",
    RetrieveCTA: "msg.retrieveCTA",
    Retrieving: "msg.retrieving",
    ShareInstructions: "msg.shareInstructions",
    ShareSuccess: "msg.shareSuccess",
    SuccessTitle: "msg.successTitle",
  },

  Restrictions: {
    Legend: "restrictions.legend",
    TabPassphrase: "restrictions.tab.passphrase",
    TabIP: "restrictions.tab.ip",
    TabCountry: "restrictions.tab.country",
    TabASN: "restrictions.tab.asn",
    Applied: "restrictions.applied",
  },

  Aria: {
    AllowedIPs: "aria.allowedIPs",
    AllowedCountries: "aria.allowedCountries",
    AllowedASNs: "aria.allowedASNs",
    CopySecret: "aria.copySecret",
    DownloadSecret: "aria.downloadSecret",
    DownloadQRCode: "aria.downloadQRCode",
    ExpiresSelect: "aria.expiresSelect",
    FileInput: "aria.fileInput",
    KeyInput: "aria.keyInput",
    LogoHome: "aria.logoHome",
    SecretInput: "aria.secretInput",
    SwitchToDark: "aria.switchToDark",
    SwitchToLight: "aria.switchToLight",
    ThemeToggle: "aria.themeToggle",
    TokenInput: "aria.tokenInput",
    UrlInput: "aria.urlInput",
    Passphrase: "aria.passphrase",
    PassphraseInput: "aria.passphraseInput",
  },

  Meta: {
    Create: "meta.create",
    Get: "meta.get",
    Homepage: "meta.homepage",
  },

  Homepage: {
    CreateButton: "homepage.create.button",
    CreateDescription: "homepage.create.description",
    DocsLink: "homepage.docs.link",
    Feature1Description: "homepage.how.feature1.description",
    Feature1Title: "homepage.how.feature1.title",
    Feature2Description: "homepage.how.feature2.description",
    Feature2Title: "homepage.how.feature2.title",
    Feature3Description: "homepage.how.feature3.description",
    Feature3Title: "homepage.how.feature3.title",
    Feature4Description: "homepage.how.feature4.description",
    Feature4Title: "homepage.how.feature4.title",
    Feature5Description: "homepage.how.feature5.description",
    Feature5Title: "homepage.how.feature5.title",
    RetrieveButton: "homepage.retrieve.button",
    RetrieveDescription: "homepage.retrieve.description",
    Tagline: "homepage.tagline",
  },

  Footer: {
    Privacy: "footer.privacy",
  },

  Error: {
    AccessDenied: "error.ACCESS_DENIED",
    AuthenticationRequired: "error.AUTHENTICATION_REQUIRED",
    Base64Error: "error.BASE64_ERROR",
    CryptoApiUnavailable: "error.CRYPTO_API_UNAVAILABLE",
    CryptoContextDisposed: "error.CRYPTO_CONTEXT_DISPOSED",
    DecryptionFailed: "error.DECRYPTION_FAILED",
    ExpectedString: "error.EXPECTED_STRING",
    ExpectedUint8Array: "error.EXPECTED_UINT8_ARRAY",
    HashValidationFailed: "error.HASH_VALIDATION_FAILED",
    InvalidAuthToken: "error.INVALID_AUTH_TOKEN",
    InvalidEncryptedData: "error.INVALID_ENCRYPTED_DATA",
    InvalidHash: "error.INVALID_HASH",
    NotSupported: "error.NOT_SUPPORTED",
    InvalidInputFormat: "error.INVALID_INPUT_FORMAT",
    InvalidKey: "error.INVALID_KEY",
    InvalidPayload: "error.INVALID_PAYLOAD",
    InvalidRestrictions: "error.INVALID_RESTRICTIONS",
    InvalidSecretId: "error.INVALID_SECRET_ID",
    InvalidServerResponse: "error.INVALID_SERVER_RESPONSE",
    InvalidToken: "error.INVALID_TOKEN",
    InvalidTtl: "error.INVALID_TTL",
    InvalidUrlFormat: "error.INVALID_URL_FORMAT",
    MissingAuthToken: "error.MISSING_AUTH_TOKEN",
    MissingDecryptionKey: "error.MISSING_DECRYPTION_KEY",
    MissingHash: "error.MISSING_HASH",
    MissingKey: "error.MISSING_KEY",
    MissingSecretId: "error.MISSING_SECRET_ID",
    PassphraseRequired: "error.PASSPHRASE_REQUIRED",
    PassphraseTooShort: "error.PASSPHRASE_TOO_SHORT",
    PayloadTooLarge: "error.PAYLOAD_TOO_LARGE",
    InvalidIPAddress: "error.INVALID_IP_ADDRESS",
    InvalidCountryCode: "error.INVALID_COUNTRY_CODE",
    InvalidASN: "error.INVALID_ASN",
    ASNMustBeNumber: "error.ASN_MUST_BE_NUMBER",
    RetrieveFailed: "error.RETRIEVE_FAILED",
    SecretAlreadyAccessed: "error.SECRET_ALREADY_ACCESSED",
    SecretNotFound: "error.SECRET_NOT_FOUND",
    SendFailed: "error.SEND_FAILED",
  },

  Validation: {
    EmptyJson: "validation.EMPTY_JSON",
    InvalidFilename: "validation.INVALID_FILENAME",
    InvalidJsonFormat: "validation.INVALID_JSON_FORMAT",
    InvalidToken: "validation.INVALID_TOKEN",
    InvalidTtl: "validation.INVALID_TTL",
    MissingData: "validation.MISSING_DATA",
  },
} as const;

export type LanguageCode = "en" | "de";

// Create a union type of all possible translation keys
type I18nKeyValues =
  (typeof I18nKeys)[keyof typeof I18nKeys][keyof (typeof I18nKeys)[keyof typeof I18nKeys]];
type TranslationKey = I18nKeyValues | string; // Allow string fallback for HTML data attributes

interface TranslationDictionary {
  [key: string]: string;
}

type Translations = {
  [lang in LanguageCode]: TranslationDictionary;
};

const translations: Translations = {
  en: {
    [I18nKeys.Page.CreateTitle]: "Hakanai - Create Secret",
    [I18nKeys.Page.GetTitle]: "Hakanai - Retrieve Secret",
    [I18nKeys.Page.HomepageTitle]: "Hakanai - One-Time Secret Sharing",
    [I18nKeys.Page.ShareTitle]: "Hakanai - Share Data",

    [I18nKeys.Label.AllowedIPs]: "IP Address Restrictions:",
    [I18nKeys.Label.AllowedCountries]: "Country Restrictions:",
    [I18nKeys.Label.AllowedASNs]: "Network (ASN) Restrictions:",
    [I18nKeys.Label.ContentPreview]: "Content Preview",
    [I18nKeys.Label.ExpiresIn]: "Expires in:",
    [I18nKeys.Label.Expires]: "Expires after:",
    [I18nKeys.Label.FileSelect]: "Select file to share:",
    [I18nKeys.Label.File]: "🗂️ File",
    [I18nKeys.Label.Filename]: "Filename:",
    [I18nKeys.Label.Key]: "Decryption Key:",
    [I18nKeys.Label.RestrictAccess]: "Restrict access to this secret",
    [I18nKeys.Label.SaveToken]: "Remember authentication token",
    [I18nKeys.Label.SecretType]: "Secret Type:",
    [I18nKeys.Label.Secret]: "Message:",
    [I18nKeys.Label.SeparateKey]: "Show Key separately",
    [I18nKeys.Label.Size]: "Size:",
    [I18nKeys.Label.Text]: "📝 Text Message",
    [I18nKeys.Label.Token]: "Token:",
    [I18nKeys.Label.Url]: "Secret URL:",
    [I18nKeys.Label.Passphrase]: "Passphrase Protection:",
    [I18nKeys.Label.PassphraseInput]: "Passphrase:",

    [I18nKeys.Placeholder.Secret]: "Enter your secret message here...",
    [I18nKeys.Placeholder.Passphrase]:
      "Enter passphrase to protect this secret",
    [I18nKeys.Placeholder.Token]: "Enter authentication token here...",

    [I18nKeys.Restrictions.Legend]: "Access Restrictions (Optional):",
    [I18nKeys.Restrictions.TabPassphrase]: "Passphrase",
    [I18nKeys.Restrictions.TabIP]: "IP",
    [I18nKeys.Restrictions.TabCountry]: "Country",
    [I18nKeys.Restrictions.TabASN]: "Network",
    [I18nKeys.Restrictions.Applied]: "Access Restrictions Applied:",

    [I18nKeys.Helper.AllowedIPs]:
      "Enter IP addresses or CIDR ranges (one per line) that can access this secret. Leave empty to allow access from any IP address.",
    [I18nKeys.Helper.AllowedCountries]:
      "Enter ISO 3166-1 alpha-2 country codes (one per line) that can access this secret. Leave empty to allow access from any country.",
    [I18nKeys.Helper.AllowedASNs]:
      "Enter Autonomous System Numbers (ASNs) (one per line) that can access this secret. Leave empty to allow access from any ASN.",
    [I18nKeys.Helper.Url]:
      "Enter the URL to access the secret.\nThe decryption key after # is never sent to the server",
    [I18nKeys.Helper.Secret]:
      "Your message will be encrypted before leaving your browser",
    [I18nKeys.Helper.FileSelect]: "File will be encrypted before upload.",
    [I18nKeys.Helper.Token]: "Leave empty if no authentication is required",
    [I18nKeys.Helper.Key]:
      "Base64-encoded decryption key with optional hash (shared separately)",
    [I18nKeys.Helper.SeparateKey]:
      "When enabled, the URL and decryption key are displayed separately, allowing you to share them through different channels for enhanced security.",
    [I18nKeys.Helper.RestrictAccess]: "Limit who can access this secret.",
    [I18nKeys.Helper.SaveToken]:
      "Token will be stored securely in your browser for the current session only. You will need to re-enter it when you start a new browser session.",
    [I18nKeys.Helper.Expires]:
      "Secret will self-destruct after this time or first view",
    [I18nKeys.Helper.Passphrase]:
      "Require a passphrase to access this secret. The passphrase can be shared through a different channel (e.g., phone call) for enhanced security.",
    [I18nKeys.Helper.PassphraseInput]:
      "This secret requires a passphrase to access",

    [I18nKeys.Aria.Passphrase]:
      "Enter a passphrase that will be required to access this secret",
    [I18nKeys.Aria.PassphraseInput]:
      "Enter the passphrase for this protected secret",

    [I18nKeys.Time.FiveMin]: "5 minutes",
    [I18nKeys.Time.ThirtyMin]: "30 minutes",
    [I18nKeys.Time.OneHour]: "1 hour",
    [I18nKeys.Time.TwoHours]: "2 hours",
    [I18nKeys.Time.TwelveHours]: "12 hours",
    [I18nKeys.Time.TwentyFourHours]: "24 hours",
    [I18nKeys.Time.SevenDays]: "7 days",
    [I18nKeys.Time.Custom]: "Custom...",
    [I18nKeys.Time.Minutes]: "minutes",
    [I18nKeys.Time.Hours]: "hours",
    [I18nKeys.Time.Days]: "days",
    [I18nKeys.Time.Seconds]: "seconds",

    [I18nKeys.Button.Create]: "🛡️ Create Secret",
    [I18nKeys.Button.Retrieve]: "📖 Retrieve Secret",
    [I18nKeys.Button.Copy]: "📋 Copy",
    [I18nKeys.Button.Copied]: "Copied!",
    [I18nKeys.Button.Close]: "Close",
    [I18nKeys.Button.ShowQrCode]: "Show QR Code",
    [I18nKeys.Button.Download]: "💾 Download",
    [I18nKeys.Button.ChooseFile]: "📁 Choose File",
    [I18nKeys.Button.ReadClipboard]: "📄 Read Clipboard",

    [I18nKeys.Dropzone.Primary]: "Drop files here or click to select",
    [I18nKeys.Dropzone.Secondary]: "Supports all file types",
    [I18nKeys.Dropzone.Helper]: "File will be encrypted before upload.",

    [I18nKeys.Msg.Creating]: "Creating secret...",
    [I18nKeys.Msg.Retrieving]: "Retrieving secret...",
    [I18nKeys.Msg.JsRequired]: "JavaScript Required",
    [I18nKeys.Msg.JsRequiredDetail]:
      "This application requires JavaScript to encrypt secrets securely in your browser.",
    [I18nKeys.Msg.EmptySecret]: "Please enter a secret to share",
    [I18nKeys.Msg.EmptyFile]: "Please select a file to share",
    [I18nKeys.Msg.CreateFailed]: "Failed to create secret",
    [I18nKeys.Msg.FileReadError]: "Error reading file",
    [I18nKeys.Msg.InvalidFilename]:
      "Invalid filename. Please select a file with a valid name.",
    [I18nKeys.Msg.EmptyUrl]: "Please enter a valid secret URL",
    [I18nKeys.Msg.InvalidUrl]:
      "Invalid URL format. Please include the full URL with the secret key after #",
    [I18nKeys.Msg.MissingKey]: "Please enter the decryption key",
    [I18nKeys.Msg.RetrieveFailed]: "Failed to retrieve secret",
    [I18nKeys.Msg.SuccessTitle]: "Success",
    [I18nKeys.Msg.ErrorTitle]: "Error",
    [I18nKeys.Msg.CopyFailed]:
      "Failed to copy. Please select and copy manually.",
    [I18nKeys.Msg.CreateNote]:
      "The secret will be deleted after the first access or when it expires.",
    [I18nKeys.Msg.ShareInstructions]:
      "Share this URL with the intended recipient.\nThe secret is encrypted and can only be accessed once.",
    [I18nKeys.Msg.ClipboardError]: "Clipboard Error",
    [I18nKeys.Msg.ClipboardRequired]: "Clipboard Access Required",
    [I18nKeys.Msg.ClipboardRequiredDetail]:
      "Click the button below to read the shared content from your clipboard.",
    [I18nKeys.Msg.ClipboardPermissionDenied]:
      "Clipboard access denied. Please grant permission and try again.",
    [I18nKeys.Msg.ClipboardInvalidJson]:
      "Clipboard does not contain valid JSON",
    [I18nKeys.Msg.ClipboardEmpty]: "Clipboard is empty",
    [I18nKeys.Msg.ReadingClipboard]: "Reading clipboard...",
    [I18nKeys.Msg.CreatingSecret]: "Creating secret...",
    [I18nKeys.Msg.ShareSuccess]:
      "Your secret has been created and the URL copied to clipboard:",
    [I18nKeys.Msg.ExpectedJsonFormat]: "Expected JSON format:",
    [I18nKeys.Msg.RetrieveNote]:
      "This secret has been deleted from the server and cannot be accessed again.",
    [I18nKeys.Msg.RetrieveCTA]:
      "Learn more and share your own secrets securely",
    [I18nKeys.Msg.Downloaded]: "Secret downloaded as text file",
    [I18nKeys.Msg.BinaryDetected]: "Use download button to save the file.",

    [I18nKeys.Aria.CopySecret]: "Copy secret to clipboard",
    [I18nKeys.Aria.DownloadSecret]: "Download secret as file",
    [I18nKeys.Aria.DownloadQRCode]: "Download QR code as SVG file",
    [I18nKeys.Aria.AllowedIPs]:
      "Enter IP addresses or CIDR ranges, one per line, to restrict access to this secret",
    [I18nKeys.Aria.AllowedCountries]:
      "Enter 2-letter country codes, one per line, to restrict access to this secret by geographic location",
    [I18nKeys.Aria.AllowedASNs]:
      "Enter ASN numbers, one per line, to restrict access to this secret by network provider",
    [I18nKeys.Aria.SecretInput]:
      "Enter the secret message you want to share securely",
    [I18nKeys.Aria.FileInput]:
      "Select a file to share securely. The file will be encrypted before being sent.",
    [I18nKeys.Aria.TokenInput]:
      "Enter the authentication token if required by the server",
    [I18nKeys.Aria.ExpiresSelect]:
      "Select how long the secret should be available before it expires",
    [I18nKeys.Aria.UrlInput]:
      "Enter the full URL including the secret key after the hash",
    [I18nKeys.Aria.KeyInput]: "Enter the base64-encoded decryption key",
    [I18nKeys.Aria.ThemeToggle]: "Switch between light and dark mode",
    [I18nKeys.Aria.SwitchToLight]: "Switch to light mode",
    [I18nKeys.Aria.SwitchToDark]: "Switch to dark mode",
    [I18nKeys.Aria.LogoHome]: "Go to home page",

    [I18nKeys.Meta.Create]:
      "Create and share one-time secrets securely with Hakanai - zero-knowledge secret sharing",
    [I18nKeys.Meta.Get]:
      "Retrieve your one-time secret securely with Hakanai - zero-knowledge secret sharing",
    [I18nKeys.Meta.Homepage]:
      "Hakanai - Zero-knowledge one-time secret sharing service",

    [I18nKeys.Homepage.Tagline]:
      "Share secrets securely with zero-knowledge encryption",
    [I18nKeys.Homepage.CreateDescription]:
      "Share text messages or files securely. All encryption happens in your browser.",
    [I18nKeys.Homepage.CreateButton]: "✨ Create Secret",
    [I18nKeys.Homepage.RetrieveDescription]:
      "Have a secret URL? Enter it here to decrypt and view your one-time secret.",
    [I18nKeys.Homepage.RetrieveButton]: "📨 Retrieve Secret",
    [I18nKeys.Homepage.Feature1Title]: "Zero-Knowledge",
    [I18nKeys.Homepage.Feature1Description]:
      "Your secrets are encrypted in your browser before being sent",
    [I18nKeys.Homepage.Feature2Title]: "One-Time",
    [I18nKeys.Homepage.Feature2Description]:
      "Secrets are destroyed after being viewed once",
    [I18nKeys.Homepage.Feature3Title]: "Secure",
    [I18nKeys.Homepage.Feature3Description]:
      "AES-256-GCM encryption with SHA-256 based content integrity verification",
    [I18nKeys.Homepage.Feature4Title]: "Private",
    [I18nKeys.Homepage.Feature4Description]:
      "The server never sees your unencrypted data",
    [I18nKeys.Homepage.Feature5Title]: "Open Source",
    [I18nKeys.Homepage.Feature5Description]:
      "Fully open source and auditable on GitHub",
    [I18nKeys.Homepage.DocsLink]: "View API Documentation",

    [I18nKeys.Footer.Privacy]: "Privacy Policy",

    [I18nKeys.Page.PrivacyTitle]: "Privacy Policy",

    [I18nKeys.Error.AccessDenied]:
      "Access denied - you are not allowed to access the secret",
    [I18nKeys.Error.SendFailed]: "Failed to send secret",
    [I18nKeys.Error.AuthenticationRequired]:
      "Authentication required - Please enter your authentication token",
    [I18nKeys.Error.InvalidToken]:
      "Invalid authentication token - Please check your token and try again",
    [I18nKeys.Error.SecretNotFound]: "Secret not found or has expired",
    [I18nKeys.Error.SecretAlreadyAccessed]:
      "Secret has been accessed and is no longer available",
    [I18nKeys.Error.RetrieveFailed]: "Failed to retrieve secret",
    [I18nKeys.Error.MissingDecryptionKey]: "No decryption key found in URL",
    [I18nKeys.Error.MissingHash]:
      "No content integrity verification hash found in URL",
    [I18nKeys.Error.PassphraseRequired]:
      "This secret is protected and requires a passphrase to access",
    [I18nKeys.Error.PassphraseTooShort]:
      "Passphrase must be at least 8 characters long",
    [I18nKeys.Error.InvalidIPAddress]: "Invalid IP address or CIDR notation",
    [I18nKeys.Error.InvalidCountryCode]:
      "Invalid country code. Must be a 2-letter uppercase code (e.g., US, DE, CA)",
    [I18nKeys.Error.InvalidASN]:
      "Invalid ASN. Must be between 1 and 4294967295",
    [I18nKeys.Error.ASNMustBeNumber]: "ASN must be a number",
    [I18nKeys.Error.PayloadTooLarge]: "Secret size exceeds the limit",
    [I18nKeys.Error.HashValidationFailed]:
      "Hash validation failed - data may be tempered or corrupted",
    [I18nKeys.Error.NotSupported]:
      "Feature not supported - the server does not support this operation",

    [I18nKeys.Validation.MissingData]: "Missing or invalid data field",
    [I18nKeys.Validation.InvalidFilename]:
      "Invalid filename field - must be text",
    [I18nKeys.Validation.InvalidToken]: "Invalid token field - must be text",
    [I18nKeys.Validation.InvalidTtl]:
      "Invalid expiration time - must be a positive number",
    [I18nKeys.Validation.EmptyJson]: "Clipboard content is empty",
    [I18nKeys.Validation.InvalidJsonFormat]:
      "Invalid clipboard format - not valid JSON",

    [I18nKeys.Error.ExpectedUint8Array]:
      "Input must be a Uint8Array (binary data)",
    [I18nKeys.Error.ExpectedString]: "Input must be a string (text data)",
    [I18nKeys.Error.InvalidInputFormat]:
      "Input contains invalid characters or format",
    [I18nKeys.Error.MissingKey]: "Secret key is required",
    [I18nKeys.Error.InvalidKey]: "Secret key has invalid length or format",
    [I18nKeys.Error.InvalidHash]:
      "Hash format is invalid (must be 22 characters long)",
    [I18nKeys.Error.CryptoApiUnavailable]:
      "Web Crypto API is not available in this browser",
    [I18nKeys.Error.InvalidTtl]: "TTL value must be a positive integer",
    [I18nKeys.Error.MissingAuthToken]: "Authentication token is required",
    [I18nKeys.Error.InvalidAuthToken]: "Authentication token format is invalid",
    [I18nKeys.Error.Base64Error]: "Base64 encoding/decoding failed",
    [I18nKeys.Error.InvalidEncryptedData]:
      "Encrypted data is corrupted or too short",
    [I18nKeys.Error.DecryptionFailed]:
      "Decryption failed: invalid key or corrupted data",
    [I18nKeys.Error.InvalidUrlFormat]: "Invalid URL format",
    [I18nKeys.Error.MissingSecretId]: "URL is missing secret ID",
    [I18nKeys.Error.InvalidSecretId]: "Secret ID format is invalid",
    [I18nKeys.Error.InvalidPayload]: "Payload object is invalid or malformed",
    [I18nKeys.Error.InvalidRestrictions]:
      "IP restrictions are invalid or malformed",
    [I18nKeys.Error.InvalidServerResponse]:
      "Server response is missing required data",
    [I18nKeys.Error.CryptoContextDisposed]:
      "Crypto context has been disposed and cannot be reused",
  },
  de: {
    [I18nKeys.Page.CreateTitle]: "Hakanai - Secret erstellen",
    [I18nKeys.Page.GetTitle]: "Hakanai - Secret abrufen",
    [I18nKeys.Page.HomepageTitle]: "Hakanai - Einmal-Secret-Sharing",
    [I18nKeys.Page.ShareTitle]: "Hakanai - Daten teilen",

    [I18nKeys.Label.AllowedIPs]: "IP-Adress-Beschränkungen:",
    [I18nKeys.Label.AllowedCountries]: "Länder-Beschränkungen:",
    [I18nKeys.Label.AllowedASNs]: "Netzwerk-(ASN-)Beschränkungen:",
    [I18nKeys.Label.Secret]: "Text:",
    [I18nKeys.Label.SecretType]: "Secret-Typ:",
    [I18nKeys.Label.Text]: "📝 Text-Nachricht",
    [I18nKeys.Label.File]: "🗂️ Datei",
    [I18nKeys.Label.FileSelect]: "Datei zum Teilen auswählen:",
    [I18nKeys.Label.Token]: "Token:",
    [I18nKeys.Label.Expires]: "Läuft ab nach:",
    [I18nKeys.Label.Url]: "Secret-URL:",
    [I18nKeys.Label.Key]: "Geheimer Schlüssel:",
    [I18nKeys.Label.RestrictAccess]: "Zugriff auf dieses Secret beschränken",
    [I18nKeys.Label.SeparateKey]: "Schlüssel separat anzeigen",
    [I18nKeys.Label.SaveToken]: "Token merken",
    [I18nKeys.Label.Filename]: "Dateiname:",
    [I18nKeys.Label.Size]: "Größe:",
    [I18nKeys.Label.ExpiresIn]: "Läuft ab in:",
    [I18nKeys.Label.ContentPreview]: "Inhaltsvorschau",
    [I18nKeys.Label.Passphrase]: "Passphrase-Schutz:",
    [I18nKeys.Label.PassphraseInput]: "Passphrase:",

    [I18nKeys.Placeholder.Secret]: "Hier wird gen geheime Text eingegeben...",
    [I18nKeys.Placeholder.Passphrase]:
      "Passphrase zum Schutz dieses Secrets eingeben",
    [I18nKeys.Placeholder.Token]: "Authentifizierungs-Token eingeben",

    [I18nKeys.Restrictions.Legend]: "Zugriffsbeschränkungen (Optional):",
    [I18nKeys.Restrictions.TabPassphrase]: "Passphrase",
    [I18nKeys.Restrictions.TabIP]: "IP",
    [I18nKeys.Restrictions.TabCountry]: "Land",
    [I18nKeys.Restrictions.TabASN]: "Netzwerk",
    [I18nKeys.Restrictions.Applied]: "Zugriffsbeschränkungen:",

    [I18nKeys.Helper.AllowedIPs]:
      "IP-Adressen oder CIDR-Bereiche (eine pro Zeile), die auf dieses Secret zugreifen können. Leer lassen, um Zugriff von jeder IP-Adresse zu ermöglichen.",
    [I18nKeys.Helper.AllowedCountries]:
      "ISO 3166-1 alpha-2 Ländercodes (eine pro Zeile), die auf dieses Secret zugreifen können. Leer lassen, um Zugriff aus jedem Land zu ermöglichen.",
    [I18nKeys.Helper.AllowedASNs]:
      "Autonome Systemnummern (ASNs) (eine pro Zeile), die auf dieses Secret zugreifen können. Leer lassen um Zugriff aus allen autonomen Systemen zu ermöglichen.",
    [I18nKeys.Helper.Url]:
      "URL eigeben um auf das Secret zuzugreifen.\nDer geheime Schlüssel nach dem # wird niemals an den Server gesendet",
    [I18nKeys.Helper.Secret]:
      "Die Nachricht wird verschlüsselt, bevor sie den Browser verlässt",
    [I18nKeys.Helper.FileSelect]:
      "Die Datei wird vor dem Upload verschlüsselt.",
    [I18nKeys.Helper.Token]:
      "Kann leer gelassen werden, wenn keine Authentifizierung erforderlich ist",
    [I18nKeys.Helper.Key]:
      "Base64-kodierter geheimer Schlüssel mit optionalen Hash-Informationen (separat geteilt)",
    [I18nKeys.Helper.SeparateKey]:
      "Wenn aktiviert, werden URL und geheimer Schlüssel separat angezeigt, so dass sie über verschiedene Kanäle für erweiterte Sicherheit geteilt werden können.",
    [I18nKeys.Helper.RestrictAccess]:
      "Zugriff auf dieses Secret auf IP-Adresse, Land oder Netzwerkanbieter beschränken.",
    [I18nKeys.Helper.SaveToken]:
      "Token wird sicher für die Session im Browser gespeichert. Nach dem Schließen des Tabs muss dieses neu eigegeben werden.",
    [I18nKeys.Helper.Expires]:
      "Das Secret wird nach dieser Zeit oder beim ersten Zugriff selbst zerstört",
    [I18nKeys.Helper.Passphrase]:
      "Erfordert eine Passphrase zum Zugriff auf dieses Secret. Für erhöhte Sicheheit kann die Passphrase über einen anderen Kanal (z.B. Telefonanruf) geteilt werden.",
    [I18nKeys.Helper.PassphraseInput]:
      "Dieses Secret erfordert eine Passphrase für den Zugriff",

    [I18nKeys.Aria.Passphrase]:
      "Passphrase eingeben, die für den Zugriff auf dieses Secret erforderlich ist",
    [I18nKeys.Aria.PassphraseInput]:
      "Passphrase für dieses geschützte Secret eingeben",

    [I18nKeys.Time.FiveMin]: "5 Minuten",
    [I18nKeys.Time.ThirtyMin]: "30 Minuten",
    [I18nKeys.Time.OneHour]: "1 Stunde",
    [I18nKeys.Time.TwoHours]: "2 Stunden",
    [I18nKeys.Time.TwelveHours]: "12 Stunden",
    [I18nKeys.Time.TwentyFourHours]: "24 Stunden",
    [I18nKeys.Time.SevenDays]: "7 Tage",
    [I18nKeys.Time.Custom]: "Benutzerdefiniert...",
    [I18nKeys.Time.Minutes]: "Minuten",
    [I18nKeys.Time.Hours]: "Stunden",
    [I18nKeys.Time.Days]: "Tage",
    [I18nKeys.Time.Seconds]: "Sekunden",

    [I18nKeys.Button.Create]: "🛡️ Secret erstellen",
    [I18nKeys.Button.Retrieve]: "📖 Secret abrufen",
    [I18nKeys.Button.Copy]: "📋 Kopieren",
    [I18nKeys.Button.Copied]: "Kopiert!",
    [I18nKeys.Button.Close]: "Schließen",
    [I18nKeys.Button.ShowQrCode]: "QR-Code anzeigen",
    [I18nKeys.Button.Download]: "💾 Speichern",
    [I18nKeys.Button.ChooseFile]: "📁 Datei auswählen",
    [I18nKeys.Button.ReadClipboard]: "📄 Zwischenablage lesen",

    [I18nKeys.Dropzone.Primary]:
      "Dateien hier ablegen oder zum Auswählen klicken",
    [I18nKeys.Dropzone.Secondary]: "Unterstützt alle Dateitypen",
    [I18nKeys.Dropzone.Helper]: "Die Datei wird vor dem Upload verschlüsselt.",

    [I18nKeys.Msg.Creating]: "Secret wird erstellt...",
    [I18nKeys.Msg.Retrieving]: "Secret wird abgerufen...",
    [I18nKeys.Msg.JsRequired]: "JavaScript erforderlich",
    [I18nKeys.Msg.JsRequiredDetail]:
      "Diese Anwendung benötigt JavaScript, um Secrets sicher im Browser zu verschlüsseln.",
    [I18nKeys.Msg.EmptySecret]: "Bitte den Text für das Secret eingeben",
    [I18nKeys.Msg.EmptyFile]: "Bitte eine Datei zum Teilen auswählen",
    [I18nKeys.Msg.CreateFailed]: "Fehler beim Erstellen des Secrets",
    [I18nKeys.Msg.FileReadError]: "Fehler beim Lesen der Datei",
    [I18nKeys.Msg.InvalidFilename]:
      "Ungültiger Dateiname. Bitte eine Datei mit einem gültigen Namen auswählen.",
    [I18nKeys.Msg.EmptyUrl]: "Bitte eine gültige Secret-URL eingeben",
    [I18nKeys.Msg.InvalidUrl]:
      "Ungültiges URL-Format. Bitte vollständige URL einschließlich des Teils nach dem # eingeben",
    [I18nKeys.Msg.MissingKey]: "Bitte den geheimen Schlüssel eingeben",
    [I18nKeys.Msg.RetrieveFailed]: "Fehler beim Abrufen des Secrets",
    [I18nKeys.Msg.SuccessTitle]: "Erfolg",
    [I18nKeys.Msg.ErrorTitle]: "Fehler",
    [I18nKeys.Msg.CopyFailed]:
      "Kopieren fehlgeschlagen. Bitte manuell auswählen und kopieren.",
    [I18nKeys.Msg.CreateNote]:
      "Das Secret wird nach dem ersten Zugriff oder bei Ablauf gelöscht.",
    [I18nKeys.Msg.ShareInstructions]:
      "Diese URL kann nun mit dem vorgesehenen Empfänger geteilt werden.\nDas Secret ist verschlüsselt und kann nur einmal abgerufen werden.",
    [I18nKeys.Msg.ClipboardError]: "Zwischenablage-Fehler",
    [I18nKeys.Msg.ClipboardRequired]: "Zwischenablage-Zugriff erforderlich",
    [I18nKeys.Msg.ClipboardRequiredDetail]:
      "Bitte den Button klicken, um den geteilten Inhalt aus der Zwischenablage zu lesen.",
    [I18nKeys.Msg.ClipboardPermissionDenied]:
      "Zwischenablage-Zugriff verweigert. Bitte Berechtigung erteilen und erneut versuchen.",
    [I18nKeys.Msg.ClipboardInvalidJson]:
      "Zwischenablage enthält kein gültiges JSON",
    [I18nKeys.Msg.ClipboardEmpty]: "Zwischenablage ist leer",
    [I18nKeys.Msg.ReadingClipboard]: "Zwischenablage wird gelesen...",
    [I18nKeys.Msg.CreatingSecret]: "Secret wird erstellt...",
    [I18nKeys.Msg.ShareSuccess]:
      "Das Secret wurde erstellt und die URL in die Zwischenablage kopiert:",
    [I18nKeys.Msg.ExpectedJsonFormat]: "Erwartetes JSON-Format:",
    [I18nKeys.Msg.RetrieveNote]:
      "Dieses Secret wurde vom Server gelöscht und kann nicht erneut abgerufen werden.",
    [I18nKeys.Msg.RetrieveCTA]:
      "Mehr erfahren und eigene Secrets sicher teilen",
    [I18nKeys.Msg.Downloaded]: "Secret als Textdatei heruntergeladen",
    [I18nKeys.Msg.BinaryDetected]:
      "Bitte Download-Button verwenden, um die Datei zu speichern.",

    [I18nKeys.Aria.AllowedIPs]:
      "IP-Adressen oder CIDR-Bereiche eingeben, eine pro Zeile, um den Zugriff auf dieses Secret zu beschränken",
    [I18nKeys.Aria.AllowedCountries]:
      "2-Buchstaben-Ländercodes eingeben, eine pro Zeile, um den Zugriff auf dieses Secret nach geografischer Lage zu beschränken",
    [I18nKeys.Aria.AllowedASNs]:
      "ASN-Nummern eingeben, eine pro Zeile, um den Zugriff auf dieses Secret nach Netzwerkanbieter zu beschränken",
    [I18nKeys.Aria.CopySecret]: "Secret in die Zwischenablage kopieren",
    [I18nKeys.Aria.DownloadSecret]: "Secret als Datei herunterladen",
    [I18nKeys.Aria.DownloadQRCode]: "QR-Code als SVG-Datei herunterladen",
    [I18nKeys.Aria.SecretInput]:
      "Bitte die geheime Nachricht eingeben, die sicher geteult werden soll",
    [I18nKeys.Aria.FileInput]:
      "Datei zum sicheren Teilen auswählen. Die Datei wird vor dem Versenden verschlüsselt.",
    [I18nKeys.Aria.TokenInput]:
      "Bitte den Authentifizierungs-Token eingeben, falls vom Server erforderlich",
    [I18nKeys.Aria.ExpiresSelect]:
      "Bitte die Zeit auswählen, nach der das Secret abläuft",
    [I18nKeys.Aria.UrlInput]:
      "Bitte die vollständige URL einschließlich des Schlüssels nach dem Hash eingeben",
    [I18nKeys.Aria.KeyInput]:
      "Bitte den Base64-kodierten geheimen Schlüssel eingeben",
    [I18nKeys.Aria.ThemeToggle]: "Zwischen hellem und dunklem Modus wechseln",
    [I18nKeys.Aria.SwitchToLight]: "Zum hellen Modus wechseln",
    [I18nKeys.Aria.SwitchToDark]: "Zum dunklen Modus wechseln",
    [I18nKeys.Aria.LogoHome]: "Zur Startseite gehen",

    [I18nKeys.Meta.Create]:
      "One-Time-Secrets sicher erstellen und teilen mit Hakanai",
    [I18nKeys.Meta.Get]: "One-Time-Secrets sicher teilen mit Hakanai",
    [I18nKeys.Meta.Homepage]:
      "Hakanai - Zero-Knowledge One-Time Secret-Sharing Service",

    [I18nKeys.Homepage.Tagline]:
      "Secrets sicher teilen mit Zero-Knowledge-Verschlüsselung",
    [I18nKeys.Homepage.CreateDescription]:
      "Textnachrichten oder Dateien sicher teilen. Die Verschlüsselung erfolgt im Browser.",
    [I18nKeys.Homepage.CreateButton]: "✨ Secret erstellen",
    [I18nKeys.Homepage.RetrieveDescription]:
      "Hier kann der Secret-Link eingegeben werden, um das Einmal-Secret zu entschlüsseln und anzuzeigen.",
    [I18nKeys.Homepage.RetrieveButton]: "📨 Secret abrufen",
    [I18nKeys.Homepage.Feature1Title]: "Zero-Knowledge",
    [I18nKeys.Homepage.Feature1Description]:
      "Die Secrets werden im Browser verschlüsselt, bevor sie gesendet werden",
    [I18nKeys.Homepage.Feature2Title]: "Einmalig",
    [I18nKeys.Homepage.Feature2Description]:
      "Secrets werden nach dem ersten Abruf gelöscht",
    [I18nKeys.Homepage.Feature3Title]: "Sicher",
    [I18nKeys.Homepage.Feature3Description]:
      "AES-256-GCM-Verschlüsselung mit SHA-256 basierter Integritätsprüfung",
    [I18nKeys.Homepage.Feature4Title]: "Privat",
    [I18nKeys.Homepage.Feature4Description]:
      "Der Server sieht niemals die unverschlüsselten Daten",
    [I18nKeys.Homepage.Feature5Title]: "Open Source",
    [I18nKeys.Homepage.Feature5Description]:
      "Vollständig Open Source und auf GitHub auditierbar",
    [I18nKeys.Homepage.DocsLink]: "API-Dokumentation anzeigen",

    [I18nKeys.Footer.Privacy]: "Datenschutzerklärung",

    [I18nKeys.Page.PrivacyTitle]: "Datenschutzerklärung",

    [I18nKeys.Error.AccessDenied]:
      "Zugriff verweigert - Keine Berechtigung auf das Secret zuzugreifen",
    [I18nKeys.Error.SendFailed]: "Fehler beim Senden des Secrets",
    [I18nKeys.Error.AuthenticationRequired]:
      "Authentifizierung erforderlich - Bitte Authentifizierungs-Token eingeben",
    [I18nKeys.Error.InvalidToken]:
      "Ungültiges Authentifizierungs-Token - Bitte Token überprüfen und erneut versuchen",
    [I18nKeys.Error.SecretNotFound]: "Secret nicht gefunden oder abgelaufen",
    [I18nKeys.Error.SecretAlreadyAccessed]:
      "Secret wurde bereits abgerufen und ist nicht mehr verfügbar",
    [I18nKeys.Error.RetrieveFailed]: "Fehler beim Abrufen des Secrets",
    [I18nKeys.Error.MissingDecryptionKey]: "Kein Schlüssel in der URL gefunden",
    [I18nKeys.Error.MissingHash]:
      "Kein Verifizierungs-Hash-Code in der URL gefunden",
    [I18nKeys.Error.PassphraseRequired]:
      "Dieses Secret ist geschützt und erfordert eine Passphrase zum Zugriff",
    [I18nKeys.Error.PassphraseTooShort]:
      "Passphrase muss mindestens 8 Zeichen lang sein",
    [I18nKeys.Error.InvalidIPAddress]:
      "Ungültige IP-Adresse oder CIDR-Notation",
    [I18nKeys.Error.InvalidCountryCode]:
      "Ungültiger Ländercode. Muss ein 2-stelliger Großbuchstaben-Code sein (z.B. US, DE, CA)",
    [I18nKeys.Error.InvalidASN]:
      "Ungültige ASN. Muss zwischen 1 und 4294967295 liegen",
    [I18nKeys.Error.ASNMustBeNumber]: "ASN muss eine Zahl sein",

    [I18nKeys.Validation.MissingData]: "Fehlende oder ungültige Daten",
    [I18nKeys.Validation.InvalidFilename]:
      "Ungültiger Dateiname - muss Text sein",
    [I18nKeys.Validation.InvalidToken]: "Ungültiger Token - muss Text sein",
    [I18nKeys.Validation.InvalidTtl]:
      "Ungültige Ablaufzeit - muss eine positive Zahl sein",
    [I18nKeys.Validation.EmptyJson]: "Zwischenablage ist leer",
    [I18nKeys.Validation.InvalidJsonFormat]:
      "Ungültiges Format der Zwischenablage - kein gültiges JSON",
    [I18nKeys.Error.PayloadTooLarge]: "Secret-Größe überschreitet das Limit",

    [I18nKeys.Error.ExpectedUint8Array]:
      "Eingabe muss ein Uint8Array (binäre Daten) sein",
    [I18nKeys.Error.ExpectedString]: "Eingabe muss ein String (Textdaten) sein",
    [I18nKeys.Error.InvalidInputFormat]:
      "Eingabe enthält ungültige Zeichen oder Format",
    [I18nKeys.Error.MissingKey]: "Geheimer Schlüssel ist erforderlich",
    [I18nKeys.Error.InvalidKey]:
      "Verschlüsselungsschlüssel hat ungültige Länge oder Format",
    [I18nKeys.Error.InvalidHash]:
      "Hash-Format ist ungültig (muss 22 Zeichen lang sein)",
    [I18nKeys.Error.CryptoApiUnavailable]:
      "Web Crypto API ist in diesem Browser nicht verfügbar",
    [I18nKeys.Error.InvalidTtl]: "TTL-Wert muss eine positive Ganzzahl sein",
    [I18nKeys.Error.MissingAuthToken]:
      "Authentifizierungs-Token ist erforderlich",
    [I18nKeys.Error.InvalidAuthToken]:
      "Authentifizierungs-Token-Format ist ungültig",
    [I18nKeys.Error.Base64Error]: "Base64-Kodierung/Dekodierung fehlgeschlagen",
    [I18nKeys.Error.InvalidEncryptedData]:
      "Verschlüsselte Daten sind beschädigt oder ungültig",
    [I18nKeys.Error.DecryptionFailed]:
      "Entschlüsselung fehlgeschlagen: ungültiger Schlüssel oder beschädigte Daten",
    [I18nKeys.Error.InvalidUrlFormat]: "Ungültiges URL-Format",
    [I18nKeys.Error.MissingSecretId]: "URL fehlt die Secret-ID",
    [I18nKeys.Error.InvalidSecretId]: "Secret-ID-Format ist ungültig",
    [I18nKeys.Error.InvalidPayload]:
      "Payload-Objekt ist ungültig oder fehlerhaft",
    [I18nKeys.Error.InvalidRestrictions]:
      "IP-Beschränkungen sind ungültig oder fehlerhaft",
    [I18nKeys.Error.InvalidServerResponse]:
      "Server-Antwort fehlt erforderliche Daten",
    [I18nKeys.Error.CryptoContextDisposed]:
      "Crypto-Kontext wurde entsorgt und kann nicht wiederverwendet werden",
    [I18nKeys.Error.HashValidationFailed]:
      "Validierung fehlgeschlagen - Daten könnten beschädigt oder verändert worden sein",
    [I18nKeys.Error.NotSupported]:
      "Funktion nicht unterstützt - der Server unterstützt diese Operation nicht",
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
    return translations[this.currentLang][key] ?? translations.en[key] ?? key;
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
      const newLang = target.value;

      if (!this.isValidLanguage(newLang)) {
        return;
      }

      this.storeLanguage(newLang as LanguageCode);

      // Reload the page to apply the new language across all components
      window.location.reload();
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
export function initI18n() {
  const i18n = new I18n();
  (window as any).i18n = i18n;
}

// Note: No exports needed for browser usage - i18n is attached to window.i18n
// Exports below are for testing purposes only and will be removed in compiled JS
export { I18n, I18nKeys, translations, type TranslationKey, type Translations };
