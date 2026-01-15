// SPDX-License-Identifier: Apache-2.0

describe("get-secret.ts", () => {
  let getSecretModule: any;

  beforeAll(async () => {
    // Mock window.i18n with minimal implementation
    (global as any).window = {
      i18n: {
        t: (key: string) => key, // Just return the key to avoid hardcoded translations
      },
    };

    getSecretModule = await import("../src/get-secret");
  });

  describe("URL validation functions", () => {
    test("normalizeUrl adds protocol when missing", () => {
      const { normalizeUrl } = getSecretModule;

      expect(normalizeUrl("example.com/s/test-id#key")).toBe("https://example.com/s/test-id#key");
      expect(normalizeUrl("localhost:8080/s/test-id")).toBe("https://localhost:8080/s/test-id");
      expect(normalizeUrl("https://example.com/s/test-id#key")).toBe("https://example.com/s/test-id#key");
      expect(normalizeUrl("http://example.com/s/test-id")).toBe("http://example.com/s/test-id");
    });

    test("hasUrlFragment detects URL fragments correctly", () => {
      const { hasUrlFragment } = getSecretModule;

      expect(hasUrlFragment("https://example.com/s/test-id#key123")).toBe(true);
      expect(hasUrlFragment("https://example.com/s/test-id#")).toBe(false);
      expect(hasUrlFragment("https://example.com/s/test-id")).toBe(false);
      expect(hasUrlFragment("invalid-url")).toBe(false);
    });

    test("validateInputs returns appropriate validation results", () => {
      const { validateInputs } = getSecretModule;

      // Empty URL should return an error (any non-null string)
      const emptyUrlError = validateInputs("", "", false);
      expect(emptyUrlError).toBeTruthy();
      expect(typeof emptyUrlError).toBe("string");

      // URL without fragment and no key should return an error
      const missingKeyError = validateInputs("https://example.com/s/test-id", "", false);
      expect(missingKeyError).toBeTruthy();
      expect(typeof missingKeyError).toBe("string");

      // Different error types should be different messages
      expect(emptyUrlError).not.toBe(missingKeyError);

      // URL with fragment (should pass)
      expect(validateInputs("https://example.com/s/test-id#key", "", true)).toBe(null);

      // URL without fragment but with key (should pass)
      expect(validateInputs("https://example.com/s/test-id", "key123", false)).toBe(null);
    });
  });

  describe("URL format validation edge cases", () => {
    test("normalizeUrl handles edge cases", () => {
      const { normalizeUrl } = getSecretModule;

      // Test with realistic protocols only
      expect(normalizeUrl("http://example.com/test")).toBe("http://example.com/test");
      expect(normalizeUrl("https://example.com/test")).toBe("https://example.com/test");

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
      const { validateInputs } = getSecretModule;

      // Note: validateInputs receives already-trimmed URLs in real usage
      // Test empty URL (after trimming) - should return error
      const emptyUrlError1 = validateInputs("", "", false);
      const emptyUrlError2 = validateInputs("", "some-key", false);
      expect(emptyUrlError1).toBeTruthy();
      expect(emptyUrlError2).toBeTruthy();

      // Test missing key (after trimming) - should return error
      const missingKeyError = validateInputs("https://example.com/s/test", "", false);
      expect(missingKeyError).toBeTruthy();

      // Test valid combinations - should return null
      expect(validateInputs("https://example.com/s/test#key", "ignored", true)).toBe(null);
      expect(validateInputs("https://example.com/s/test", "valid-key", false)).toBe(null);
    });
  });

  describe("Security considerations", () => {
    test("URL normalization doesn't break valid URLs", () => {
      const { normalizeUrl } = getSecretModule;

      const secureUrl = "https://secure.example.com/s/secret-id#very-long-crypto-key-123456789";
      expect(normalizeUrl(secureUrl)).toBe(secureUrl);

      const localhostUrl = "http://localhost:8080/s/test#key";
      expect(normalizeUrl(localhostUrl)).toBe(localhostUrl);
    });

    test("hasUrlFragment correctly identifies crypto keys in URLs", () => {
      const { hasUrlFragment } = getSecretModule;

      // Real-world crypto key patterns
      expect(hasUrlFragment("https://example.com/s/ulid#AbCdEfGhIjKlMnOpQrStUvWxYz123456")).toBe(true);
      expect(hasUrlFragment("https://example.com/s/ulid#base64-url-safe_key")).toBe(true);

      // Should not false positive on empty fragments
      expect(hasUrlFragment("https://example.com/s/ulid#")).toBe(false);
    });
  });
});
