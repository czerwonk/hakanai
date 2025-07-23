describe("get-secret.ts", () => {
  let getSecretModule: any;

  beforeAll(async () => {
    // Mock window.i18n for updateUIStrings test
    (global as any).window = {
      i18n: {
        t: (key: string) => {
          const translations: { [key: string]: string } = {
            "msg.emptyUrl": "Please enter a valid secret URL",
            "msg.invalidUrl": "Invalid URL format",
            "msg.missingKey": "Please enter the decryption key",
            "msg.retrieveFailed": "Failed to retrieve secret",
            "msg.successTitle": "Secret Retrieved Successfully",
            "msg.errorTitle": "Error",
            "button.copy": "Copy",
            "msg.copyFailed": "Failed to copy",
            "button.download": "Download",
            "msg.retrieveNote": "Note: This secret has been deleted",
            "msg.binaryDetected": "Binary file detected",
            "aria.copySecret": "Copy secret to clipboard",
            "aria.downloadSecret": "Download secret as file",
            "label.filename": "Filename:",
          };
          return translations[key] || key;
        },
      },
    };

    getSecretModule = await import("../server/src/typescript/get-secret");
  });

  describe("URL validation functions", () => {
    test("normalizeUrl adds protocol when missing", () => {
      const { normalizeUrl } = getSecretModule;

      expect(normalizeUrl("example.com/s/test-id#key")).toBe(
        "https://example.com/s/test-id#key",
      );
      expect(normalizeUrl("localhost:8080/s/test-id")).toBe(
        "https://localhost:8080/s/test-id",
      );
      expect(normalizeUrl("https://example.com/s/test-id#key")).toBe(
        "https://example.com/s/test-id#key",
      );
      expect(normalizeUrl("http://example.com/s/test-id")).toBe(
        "http://example.com/s/test-id",
      );
    });

    test("hasUrlFragment detects URL fragments correctly", () => {
      const { hasUrlFragment } = getSecretModule;

      expect(hasUrlFragment("https://example.com/s/test-id#key123")).toBe(true);
      expect(hasUrlFragment("https://example.com/s/test-id#")).toBe(false);
      expect(hasUrlFragment("https://example.com/s/test-id")).toBe(false);
      expect(hasUrlFragment("invalid-url")).toBe(false);
    });

    test("validateInputs returns correct error messages", () => {
      const { validateInputs } = getSecretModule;

      // Empty URL
      expect(validateInputs("", "", false)).toBe(
        "Please enter a valid secret URL",
      );

      // URL without fragment and no key
      expect(validateInputs("https://example.com/s/test-id", "", false)).toBe(
        "Please enter the decryption key",
      );

      // URL with fragment (should pass)
      expect(
        validateInputs("https://example.com/s/test-id#key", "", true),
      ).toBe(null);

      // URL without fragment but with key (should pass)
      expect(
        validateInputs("https://example.com/s/test-id", "key123", false),
      ).toBe(null);
    });
  });

  describe("Filename generation", () => {
    test("generateFilename uses payload filename when available", () => {
      const { generateFilename } = getSecretModule;

      const payloadWithFilename = { filename: "document.pdf" };
      expect(generateFilename(payloadWithFilename, false)).toBe("document.pdf");
    });

    test("generateFilename creates timestamp filename when no filename", () => {
      const { generateFilename } = getSecretModule;

      const payloadWithoutFilename = { filename: undefined };
      const result = generateFilename(payloadWithoutFilename, false);

      expect(result).toMatch(
        /^hakanai-secret-\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}.*\.txt$/,
      );
    });

    test("generateFilename handles null filename", () => {
      const { generateFilename } = getSecretModule;

      const payloadWithNullFilename = { filename: null };
      const result = generateFilename(payloadWithNullFilename, false);

      expect(result).toMatch(
        /^hakanai-secret-\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}.*\.txt$/,
      );
    });

    test("generateFilename uses .bin extension for binary content", () => {
      const { generateFilename } = getSecretModule;

      const payloadWithoutFilename = { filename: undefined };
      const result = generateFilename(payloadWithoutFilename, true);

      expect(result).toMatch(
        /^hakanai-secret-\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}.*\.bin$/,
      );
    });

    test("generateFilename uses .txt extension for text content", () => {
      const { generateFilename } = getSecretModule;

      const payloadWithoutFilename = { filename: undefined };
      const result = generateFilename(payloadWithoutFilename, false);

      expect(result).toMatch(
        /^hakanai-secret-\d{4}-\d{2}-\d{2}T\d{2}-\d{2}-\d{2}.*\.txt$/,
      );
    });

    test("generateFilename prefers payload filename over binary detection", () => {
      const { generateFilename } = getSecretModule;

      const payloadWithFilename = { filename: "important.pdf" };
      const result = generateFilename(payloadWithFilename, true);

      expect(result).toBe("important.pdf");
    });
  });

  describe("UI string management", () => {
    test("updateUIStrings updates UI_STRINGS with translations", () => {
      const { updateUIStrings, UI_STRINGS } = getSecretModule;

      // Call updateUIStrings to populate with translations
      updateUIStrings();

      // Since updateUIStrings depends on window.i18n.t(), check that strings are updated
      // The exact values depend on our mock, but they should not be the original keys
      expect(UI_STRINGS.EMPTY_URL).not.toBe("Please enter a valid secret URL"); // Original default
      expect(UI_STRINGS.SUCCESS_TITLE).not.toBe(
        "Secret Retrieved Successfully",
      ); // Original default
      expect(UI_STRINGS.ERROR_TITLE).not.toBe("Error"); // Original default

      // Should be the translation keys since our mock doesn't have those exact keys
      expect(typeof UI_STRINGS.EMPTY_URL).toBe("string");
      expect(UI_STRINGS.EMPTY_URL.length).toBeGreaterThan(0);
    });

    test("UI_STRINGS has sensible defaults", () => {
      const { UI_STRINGS } = getSecretModule;

      // Test that we have the expected string properties
      expect(typeof UI_STRINGS.EMPTY_URL).toBe("string");
      expect(typeof UI_STRINGS.INVALID_URL).toBe("string");
      expect(typeof UI_STRINGS.MISSING_KEY).toBe("string");
      expect(typeof UI_STRINGS.RETRIEVE_FAILED).toBe("string");
      expect(typeof UI_STRINGS.SUCCESS_TITLE).toBe("string");
      expect(typeof UI_STRINGS.ERROR_TITLE).toBe("string");
      expect(typeof UI_STRINGS.COPY_FAILED).toBe("string");
      expect(typeof UI_STRINGS.NOTE_TEXT).toBe("string");
      expect(typeof UI_STRINGS.BINARY_DETECTED).toBe("string");
      expect(typeof UI_STRINGS.FILENAME_LABEL).toBe("string");

      // Test that strings are not empty
      expect(UI_STRINGS.EMPTY_URL.length).toBeGreaterThan(0);
      expect(UI_STRINGS.SUCCESS_TITLE.length).toBeGreaterThan(0);
      expect(UI_STRINGS.ERROR_TITLE.length).toBeGreaterThan(0);
    });
  });

  describe("URL format validation edge cases", () => {
    test("normalizeUrl handles edge cases", () => {
      const { normalizeUrl } = getSecretModule;

      // Test with different protocols
      expect(normalizeUrl("ftp://example.com/test")).toBe(
        "ftp://example.com/test",
      );
      expect(normalizeUrl("file://local/path")).toBe("file://local/path");

      // Test empty and malformed inputs
      expect(normalizeUrl("")).toBe("https://");
      expect(normalizeUrl("example")).toBe("https://example");
    });

    test("hasUrlFragment handles malformed URLs gracefully", () => {
      const { hasUrlFragment } = getSecretModule;

      // Test malformed URLs that might throw in URL constructor
      expect(hasUrlFragment("not-a-url")).toBe(false);
      expect(hasUrlFragment("://malformed")).toBe(false);
      expect(hasUrlFragment("")).toBe(false);
      expect(hasUrlFragment("just-text")).toBe(false);
    });

    test("validateInputs handles various input combinations", () => {
      const { validateInputs, UI_STRINGS } = getSecretModule;

      // Note: validateInputs receives already-trimmed URLs in real usage
      // Test empty URL (after trimming)
      expect(validateInputs("", "", false)).toBe(UI_STRINGS.EMPTY_URL);
      expect(validateInputs("", "some-key", false)).toBe(UI_STRINGS.EMPTY_URL);

      // Test missing key (after trimming)
      expect(validateInputs("https://example.com/s/test", "", false)).toBe(
        UI_STRINGS.MISSING_KEY,
      );

      // Test valid combinations
      expect(
        validateInputs("https://example.com/s/test#key", "ignored", true),
      ).toBe(null);
      expect(
        validateInputs("https://example.com/s/test", "valid-key", false),
      ).toBe(null);
    });
  });

  describe("Security considerations", () => {
    test("filename generation creates safe filenames", () => {
      const { generateFilename } = getSecretModule;

      const timestamp = generateFilename({ filename: null });

      // Should not contain dangerous characters
      expect(timestamp).not.toMatch(/[<>:"/\\|?*]/);

      // Should be a reasonable length
      expect(timestamp.length).toBeLessThan(100);
      expect(timestamp.length).toBeGreaterThan(10);
    });

    test("URL normalization doesn't break valid URLs", () => {
      const { normalizeUrl } = getSecretModule;

      const secureUrl =
        "https://secure.example.com/s/secret-id#very-long-crypto-key-123456789";
      expect(normalizeUrl(secureUrl)).toBe(secureUrl);

      const localhostUrl = "http://localhost:8080/s/test#key";
      expect(normalizeUrl(localhostUrl)).toBe(localhostUrl);
    });

    test("hasUrlFragment correctly identifies crypto keys in URLs", () => {
      const { hasUrlFragment } = getSecretModule;

      // Real-world crypto key patterns
      expect(
        hasUrlFragment(
          "https://example.com/s/uuid#AbCdEfGhIjKlMnOpQrStUvWxYz123456",
        ),
      ).toBe(true);
      expect(
        hasUrlFragment("https://example.com/s/uuid#base64-url-safe_key"),
      ).toBe(true);

      // Should not false positive on empty fragments
      expect(hasUrlFragment("https://example.com/s/uuid#")).toBe(false);
    });
  });
});
