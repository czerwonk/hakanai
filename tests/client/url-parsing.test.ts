/**
 * URL parsing tests for secret URLs with hash validation
 */

import { HakanaiErrorCodes } from "../../server/src/typescript/hakanai-client";

describe("UrlParser", () => {
  const { UrlParser } =
    require("../../server/src/typescript/hakanai-client") as any;

  describe("parseSecretUrl", () => {
    test("parses URLs with hash correctly", () => {
      const testCases = [
        {
          url: "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
          expected: {
            secretId: "550e8400-e29b-41d4-a716-446655440000",
            secretKey: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", // 43 chars
            hash: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
          },
        },
        {
          url: "http://localhost:8080/s/123e4567-e89b-12d3-a456-426614174000#bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb:2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
          expected: {
            secretId: "123e4567-e89b-12d3-a456-426614174000",
            secretKey: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", // 43 chars
            hash: "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
          },
        },
      ];

      for (const testCase of testCases) {
        const result = UrlParser.parseSecretUrl(testCase.url);
        expect(result).toEqual(testCase.expected);
      }
    });

    test("parses URLs without hash correctly (legacy support)", () => {
      const testCases = [
        {
          url: "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
          expected: {
            secretId: "550e8400-e29b-41d4-a716-446655440000",
            secretKey: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", // 43 chars
            hash: "", // No hash in legacy format
          },
        },
        {
          url: "http://localhost:8080/s/123e4567-e89b-12d3-a456-426614174000#ccccccccccccccccccccccccccccccccccccccccccc",
          expected: {
            secretId: "123e4567-e89b-12d3-a456-426614174000",
            secretKey: "ccccccccccccccccccccccccccccccccccccccccccc", // 43 chars
            hash: "",
          },
        },
      ];

      for (const testCase of testCases) {
        const result = UrlParser.parseSecretUrl(testCase.url);
        expect(result).toEqual(testCase.expected);
      }
    });

    test("handles uppercase and mixed case hashes", () => {
      const urls = [
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:E3b0C44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      ];

      for (const url of urls) {
        expect(() => UrlParser.parseSecretUrl(url)).not.toThrow();
      }
    });

    test("handles edge case: colon without hash", () => {
      const url =
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:";

      const result = UrlParser.parseSecretUrl(url);
      expect(result.secretKey).toBe(
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
      );
      expect(result.hash).toBe(""); // Empty string after colon
    });

    test("correctly strips hash prefix before splitting", () => {
      const url =
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

      const result = UrlParser.parseSecretUrl(url);

      // Key should not include the '#' prefix
      expect(result.secretKey).toBe(
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
      );
      expect(result.secretKey).not.toMatch(/^#/);

      // Hash should be valid
      expect(result.hash).toBe(
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      );
    });

    test("handles legacy format correctly", () => {
      const url =
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

      const result = UrlParser.parseSecretUrl(url);

      // Key should not include the '#' prefix
      expect(result.secretKey).toBe(
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
      );
      expect(result.secretKey).not.toMatch(/^#/);

      // Hash should be empty for legacy URLs
      expect(result.hash).toBe("");
    });

    test("rejects URLs with invalid hash format", () => {
      const invalidUrls = [
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:invalid_hash",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:too_short",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:contains_invalid_chars_zzzz456789abcdef0123456789abcdef012345",
      ];

      for (const url of invalidUrls) {
        expect(() => UrlParser.parseSecretUrl(url)).toThrow();
        try {
          UrlParser.parseSecretUrl(url);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_HASH);
          expect(error.message).toBe(
            "Hash must be a 64-character hexadecimal string",
          );
        }
      }
    });

    test("rejects URLs with invalid key format even with valid hash", () => {
      const invalidUrls = [
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#invalid_key:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#too_short:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      ];

      for (const url of invalidUrls) {
        expect(() => UrlParser.parseSecretUrl(url)).toThrow();
        try {
          UrlParser.parseSecretUrl(url);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_KEY);
        }
      }
    });

    test("rejects URLs with invalid secret ID even with valid key and hash", () => {
      const invalidUrls = [
        "https://example.com/s/invalid-uuid#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        "https://example.com/s/550e8400-e29b-41d4-a716-44665544000Z#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      ];

      for (const url of invalidUrls) {
        expect(() => UrlParser.parseSecretUrl(url)).toThrow();
        try {
          UrlParser.parseSecretUrl(url);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_SECRET_ID);
        }
      }
    });

    test("rejects URLs with invalid URL format", () => {
      const invalidUrls = [
        "",
        "not-a-url",
        "https://example.com/no-secret-id",
        "https://example.com/s/", // Missing secret ID
      ];

      for (const url of invalidUrls) {
        expect(() => UrlParser.parseSecretUrl(url)).toThrow();
        try {
          UrlParser.parseSecretUrl(url);
        } catch (error: any) {
          expect([
            HakanaiErrorCodes.INVALID_URL_FORMAT,
            HakanaiErrorCodes.MISSING_SECRET_ID,
          ]).toContain(error.code);
        }
      }
    });

    test("rejects URLs without fragment", () => {
      const url = "https://example.com/s/550e8400-e29b-41d4-a716-446655440000";

      expect(() => UrlParser.parseSecretUrl(url)).toThrow();
      try {
        UrlParser.parseSecretUrl(url);
      } catch (error: any) {
        expect(error.code).toBe(HakanaiErrorCodes.MISSING_DECRYPTION_KEY);
        expect(error.message).toBe(
          "URL must contain decryption key in fragment",
        );
      }
    });

    test("handles multiple colons in fragment correctly", () => {
      // With multiple colons in JavaScript split(":"),
      // "key:hash:extra:data" becomes ["key", "hash", "extra", "data"]
      // So fragmentParts[1] would just be "hash", not "hash:extra:data"
      // Let's test this behavior
      const url =
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855:extra:data";

      // This should actually succeed because fragmentParts[1] is just the valid hash
      const result = UrlParser.parseSecretUrl(url);
      expect(result.secretKey).toBe("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
      expect(result.hash).toBe("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
      
      // The extra parts after the second colon are ignored
    });

    test("provides helpful error messages", () => {
      try {
        UrlParser.parseSecretUrl(
          "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:invalid",
        );
      } catch (error: any) {
        expect(error.message).toBe(
          "Hash must be a 64-character hexadecimal string",
        );
      }

      try {
        UrlParser.parseSecretUrl(
          "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#invalid:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
        );
      } catch (error: any) {
        expect(error.message).toBe(
          "Secret key must be a 43-character base64url string (32 bytes)",
        );
      }

      try {
        UrlParser.parseSecretUrl("");
      } catch (error: any) {
        expect(error.message).toBe("URL cannot be empty");
      }
    });
  });
});

