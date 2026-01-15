// SPDX-License-Identifier: Apache-2.0

/**
 * URL parsing tests for secret URLs with hash validation
 */

import { HakanaiErrorCodes } from "../../src/hakanai-client";

describe("UrlParser", () => {
  const { UrlParser } = require("../../src/hakanai-client") as any;

  describe("parseSecretUrl", () => {
    test("parses URLs with hash correctly", () => {
      const testCases = [
        {
          url: "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA",
          expected: {
            secretId: "01KF0SR30C1X5CASYPDAJ0G6GB",
            secretKey: "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", // 43 chars
            hash: "47DEQpj8HBSa-_TImW-5JA",
          },
        },
        {
          url: "http://localhost:8080/s/01KF0SR30C1X5CASYPDAJ0G6GB#bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb:LPJNul-wow4m6Dsqxbning",
          expected: {
            secretId: "01KF0SR30C1X5CASYPDAJ0G6GB",
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

    test("rejects URLs without hash (no legacy support)", () => {
      const testCases = [
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
        "http://localhost:8080/s/01KF0SR30C1X5CASYPDAJ0G6GB#ccccccccccccccccccccccccccccccccccccccccccc",
      ];

      for (const url of testCases) {
        expect(() => UrlParser.parseSecretUrl(url)).toThrow();
        try {
          UrlParser.parseSecretUrl(url);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.MISSING_HASH);
          expect(error.message).toBe("URL fragment must contain a hash for content integrity verification");
        }
      }
    });

    test("handles valid base64url hash characters", () => {
      const urls = [
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA",
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:LPJNul-wow4m6Dsqxbning",
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:XohImNooBHFR0OVvjcYpJw",
      ];

      for (const url of urls) {
        expect(() => UrlParser.parseSecretUrl(url)).not.toThrow();
      }
    });

    test("rejects URLs with colon but empty hash", () => {
      const url = "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:";

      expect(() => UrlParser.parseSecretUrl(url)).toThrow();
      try {
        UrlParser.parseSecretUrl(url);
      } catch (error: any) {
        expect(error.code).toBe(HakanaiErrorCodes.MISSING_HASH);
        expect(error.message).toBe("URL fragment must contain a hash for content integrity verification");
      }
    });

    test("correctly strips hash prefix before splitting", () => {
      const url =
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA";

      const result = UrlParser.parseSecretUrl(url);

      // Key should not include the '#' prefix
      expect(result.secretKey).toBe("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
      expect(result.secretKey).not.toMatch(/^#/);

      // Hash should be valid
      expect(result.hash).toBe("47DEQpj8HBSa-_TImW-5JA");
    });

    test("rejects URLs without hash (former legacy format)", () => {
      const url = "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

      expect(() => UrlParser.parseSecretUrl(url)).toThrow();
      try {
        UrlParser.parseSecretUrl(url);
      } catch (error: any) {
        expect(error.code).toBe(HakanaiErrorCodes.MISSING_HASH);
        expect(error.message).toBe("URL fragment must contain a hash for content integrity verification");
      }
    });

    test("rejects URLs with invalid hash format", () => {
      const invalidUrls = [
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:invalid_hash_format",
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:too_short",
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:contains_invalid+chars",
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:contains_invalid/chars",
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:contains_invalid=chars",
      ];

      for (const url of invalidUrls) {
        expect(() => UrlParser.parseSecretUrl(url)).toThrow();
        try {
          UrlParser.parseSecretUrl(url);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_HASH);
          expect(error.message).toBe("Hash must be a 22-character base64url string (truncated SHA-256)");
        }
      }
    });

    test("rejects URLs with invalid key format even with valid hash", () => {
      const invalidUrls = [
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#invalid_key:47DEQpj8HBSa-_TImW-5JA",
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#too_short:47DEQpj8HBSa-_TImW-5JA",
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
      const url =
        "https://example.com/s/invalid-ulid#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA";
      expect(() => UrlParser.parseSecretUrl(url)).toThrow();
      try {
        UrlParser.parseSecretUrl(url);
      } catch (error: any) {
        expect(error.code).toBe(HakanaiErrorCodes.INVALID_SECRET_ID);
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
          expect([HakanaiErrorCodes.INVALID_URL_FORMAT, HakanaiErrorCodes.MISSING_SECRET_ID]).toContain(error.code);
        }
      }
    });

    test("rejects URLs without fragment", () => {
      const url = "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB0";

      expect(() => UrlParser.parseSecretUrl(url)).toThrow();
      try {
        UrlParser.parseSecretUrl(url);
      } catch (error: any) {
        expect(error.code).toBe(HakanaiErrorCodes.INVALID_URL_FORMAT);
        expect(error.message).toBe("URL must contain decryption key and hash in fragment");
      }
    });

    test("handles multiple colons in fragment correctly", () => {
      // With multiple colons in JavaScript split(":"),
      // "key:hash:extra:data" becomes ["key", "hash", "extra", "data"]
      // So fragmentParts[1] would just be "hash", not "hash:extra:data"
      // Let's test this behavior
      const url =
        "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:47DEQpj8HBSa-_TImW-5JA:extra:data";

      // This should actually succeed because fragmentParts[1] is just the valid hash
      const result = UrlParser.parseSecretUrl(url);
      expect(result.secretKey).toBe("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
      expect(result.hash).toBe("47DEQpj8HBSa-_TImW-5JA");

      // The extra parts after the second colon are ignored
    });

    test("provides helpful error messages", () => {
      try {
        UrlParser.parseSecretUrl(
          "https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA:invalid",
        );
      } catch (error: any) {
        expect(error.message).toBe("Hash must be a 22-character base64url string (truncated SHA-256)");
      }

      try {
        UrlParser.parseSecretUrl("https://example.com/s/01KF0SR30C1X5CASYPDAJ0G6GB#invalid:47DEQpj8HBSa-_TImW-5JA");
      } catch (error: any) {
        expect(error.message).toBe("Secret key must be a 43-character base64url string (32 bytes)");
      }

      try {
        UrlParser.parseSecretUrl("");
      } catch (error: any) {
        expect(error.message).toBe("URL cannot be empty");
      }
    });
  });
});
