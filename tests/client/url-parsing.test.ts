// SPDX-License-Identifier: MIT

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
          url: "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA",
          expected: {
            secretId: "550e8400-e29b-41d4-a716-446655440000",
            secretKey: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", // 43 chars
            hash: "47DEQpj8HBSa-_TImW-5JA",
          },
        },
        {
          url: "http://localhost:8080/s/123e4567-e89b-12d3-a456-426614174000#bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb:LPJNul-wow4m6Dsqxbning",
          expected: {
            secretId: "123e4567-e89b-12d3-a456-426614174000",
            secretKey: "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", // 43 chars
            hash: "LPJNul-wow4m6Dsqxbning",
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
            hash: undefined, // No hash in legacy format
          },
        },
        {
          url: "http://localhost:8080/s/123e4567-e89b-12d3-a456-426614174000#ccccccccccccccccccccccccccccccccccccccccccc",
          expected: {
            secretId: "123e4567-e89b-12d3-a456-426614174000",
            secretKey: "ccccccccccccccccccccccccccccccccccccccccccc", // 43 chars
            hash: undefined,
          },
        },
      ];

      for (const testCase of testCases) {
        const result = UrlParser.parseSecretUrl(testCase.url);
        expect(result).toEqual(testCase.expected);
      }
    });

    test("handles valid base64url hash characters", () => {
      const urls = [
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:LPJNul-wow4m6Dsqxbning",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:XohImNooBHFR0OVvjcYpJw",
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
      expect(result.hash).toBeUndefined();
    });

    test("correctly strips hash prefix before splitting", () => {
      const url =
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA";

      const result = UrlParser.parseSecretUrl(url);

      // Key should not include the '#' prefix
      expect(result.secretKey).toBe(
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
      );
      expect(result.secretKey).not.toMatch(/^#/);

      // Hash should be valid
      expect(result.hash).toBe("47DEQpj8HBSa-_TImW-5JA");
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

      // Hash should be undefined for legacy URLs
      expect(result.hash).toBeUndefined();
    });

    test("rejects URLs with invalid hash format", () => {
      const invalidUrls = [
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:invalid_hash_format",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:too_short",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:contains_invalid+chars",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:contains_invalid/chars",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:contains_invalid=chars",
      ];

      for (const url of invalidUrls) {
        expect(() => UrlParser.parseSecretUrl(url)).toThrow();
        try {
          UrlParser.parseSecretUrl(url);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_HASH);
          expect(error.message).toBe(
            "Hash must be a 22-character base64url string (truncated SHA-256)",
          );
        }
      }
    });

    test("rejects URLs with invalid key format even with valid hash", () => {
      const invalidUrls = [
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#invalid_key:47DEQpj8HBSa-_TImW-5JA",
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#too_short:47DEQpj8HBSa-_TImW-5JA",
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
        "https://example.com/s/invalid-uuid#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA",
        "https://example.com/s/550e8400-e29b-41d4-a716-44665544000Z#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA",
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
        "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA:extra:data";

      // This should actually succeed because fragmentParts[1] is just the valid hash
      const result = UrlParser.parseSecretUrl(url);
      expect(result.secretKey).toBe(
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
      );
      expect(result.hash).toBe("47DEQpj8HBSa-_TImW-5JA");

      // The extra parts after the second colon are ignored
    });

    test("provides helpful error messages", () => {
      try {
        UrlParser.parseSecretUrl(
          "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:invalid",
        );
      } catch (error: any) {
        expect(error.message).toBe(
          "Hash must be a 22-character base64url string (truncated SHA-256)",
        );
      }

      try {
        UrlParser.parseSecretUrl(
          "https://example.com/s/550e8400-e29b-41d4-a716-446655440000#invalid:47DEQpj8HBSa-_TImW-5JA",
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
