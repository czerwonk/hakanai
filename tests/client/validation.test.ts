/**
 * Input validation tests for all validation functions
 */

import { HakanaiErrorCodes } from "../../server/src/typescript/hakanai-client";

describe("InputValidation", () => {
  const { InputValidation } =
    require("../../server/src/typescript/hakanai-client") as any;

  describe("validateHash", () => {
    test("accepts valid 22-character base64url hashes", () => {
      const validHashes = [
        "47DEQpj8HBSa-_TImW-5JA", // Truncated SHA-256 of empty string
        "LPJNul-wow4m6Dsqxbning", // Truncated SHA-256 of "hello"
        "XohImNooBHFR0OVvjcYpJw", // Truncated SHA-256 of "password"
        "AAAAAAAAAAAAAAAAAAAAAA", // All A's (valid base64url) - 22 chars
        "0123456789_-0123456789", // With valid base64url chars - 22 chars
        "zzzzzzzzzzzzzzzzzzzzzz", // All z's (valid base64url) - 22 chars
      ];

      for (const hash of validHashes) {
        expect(() => InputValidation.validateHash(hash)).not.toThrow();
      }
    });

    test("rejects invalid hash formats", () => {
      const invalidHashes = [
        "too_short",
        "this_is_way_too_long_to_be_a_valid_22_character_hash_and_should_fail",
        "21_chars_12345678abc", // 21 chars
        "23_chars_123456789abcde", // 23 chars
        "contains_invalid+char1", // Contains + (not URL-safe)
        "contains_invalid/char2", // Contains / (not URL-safe)
        "contains_invalid=char3", // Contains = (padding not allowed)
        "contains_special@char4", // Contains @ (invalid)
      ];

      for (const hash of invalidHashes) {
        expect(() => InputValidation.validateHash(hash)).toThrow();
        try {
          InputValidation.validateHash(hash);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_HASH);
          expect(error.message).toBe(
            "Hash must be a 22-character base64url string (truncated SHA-256)",
          );
        }
      }
    });

    test("rejects non-string hash values", () => {
      const invalidInputs = [null, undefined, 123, {}, [], true, false];

      for (const input of invalidInputs) {
        expect(() => InputValidation.validateHash(input as any)).toThrow();
        try {
          InputValidation.validateHash(input as any);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_HASH);
          expect(error.message).toBe("Hash must be a string");
        }
      }
    });
  });

  describe("validateSecretKey", () => {
    test("accepts valid 43-character base64url keys", () => {
      const validKeys = [
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", // 43 chars, all A's
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQ", // 43 chars, mixed case
        "0123456789_-0123456789_-0123456789_-0123456", // 43 chars with valid chars
      ];

      for (const key of validKeys) {
        expect(() => InputValidation.validateSecretKey(key)).not.toThrow();
      }
    });

    test("rejects keys with invalid length", () => {
      const invalidKeys = [
        "too_short",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", // 42 chars
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA", // 44 chars
        "way_too_long_key_that_exceeds_43_characters_significantly",
      ];

      for (const key of invalidKeys) {
        expect(() => InputValidation.validateSecretKey(key)).toThrow();
        try {
          InputValidation.validateSecretKey(key);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_KEY);
        }
      }
    });

    test("rejects keys with invalid characters", () => {
      const invalidKeys = [
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA@A", // Contains @
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA+A", // Contains +
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/A", // Contains /
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=A", // Contains =
      ];

      for (const key of invalidKeys) {
        expect(() => InputValidation.validateSecretKey(key)).toThrow();
        try {
          InputValidation.validateSecretKey(key);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_KEY);
          expect(error.message).toBe(
            "Secret key must be a 43-character base64url string (32 bytes)",
          );
        }
      }
    });

    test("rejects empty keys", () => {
      expect(() => InputValidation.validateSecretKey("")).toThrow();
      expect(() => InputValidation.validateSecretKey("   ")).toThrow();

      try {
        InputValidation.validateSecretKey("");
      } catch (error: any) {
        expect(error.code).toBe(HakanaiErrorCodes.MISSING_KEY);
        expect(error.message).toBe("Secret key cannot be empty");
      }
    });
  });

  describe("validateSecretId", () => {
    test("accepts valid UUIDs", () => {
      const validUuids = [
        "550e8400-e29b-41d4-a716-446655440000", // v4 UUID
        "123e4567-e89b-12d3-a456-426614174000", // v1 UUID
        "01234567-89ab-1def-8123-456789abcdef", // v1 UUID with correct version
        "01234567-89AB-4DEF-9123-456789ABCDEF", // v4 UUID uppercase
      ];

      for (const uuid of validUuids) {
        expect(() => InputValidation.validateSecretId(uuid)).not.toThrow();
      }
    });

    test("rejects invalid UUID formats", () => {
      const invalidUuids = [
        "not-a-uuid",
        "550e8400-e29b-41d4-a716-44665544000", // Too short
        "550e8400-e29b-41d4-a716-4466554400000", // Too long
        "550e8400-e29b-41d4-a716-44665544000Z", // Invalid character
        "550e8400e29b41d4a716446655440000", // Missing dashes
      ];

      for (const uuid of invalidUuids) {
        expect(() => InputValidation.validateSecretId(uuid)).toThrow();
        try {
          InputValidation.validateSecretId(uuid);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_SECRET_ID);
          expect(error.message).toBe("Secret ID must be a valid UUID");
        }
      }
    });

    test("rejects empty secret IDs", () => {
      expect(() => InputValidation.validateSecretId("")).toThrow();
      expect(() => InputValidation.validateSecretId("   ")).toThrow();

      try {
        InputValidation.validateSecretId("");
      } catch (error: any) {
        expect(error.code).toBe(HakanaiErrorCodes.MISSING_SECRET_ID);
        expect(error.message).toBe("Secret ID cannot be empty");
      }
    });
  });

  describe("validateAuthToken", () => {
    test("accepts valid 43-character base64url tokens", () => {
      const validTokens = [
        "HUqlqUd68TmqGkNj5o7pMqRcJe2YIQqoOlMfSSYF5r8",
        "opBEGjLy_mkCsTbMog4nxnvstB39kNx8K7450KHHH4E",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
      ];

      for (const token of validTokens) {
        expect(() => InputValidation.validateAuthToken(token)).not.toThrow();
      }
    });

    test("accepts empty token (no authentication)", () => {
      expect(() => InputValidation.validateAuthToken("")).not.toThrow();
      expect(() => InputValidation.validateAuthToken("   ")).not.toThrow();
    });

    test("rejects invalid token formats", () => {
      const invalidTokens = [
        "too_short",
        "way_too_long_token_that_exceeds_43_characters_significantly",
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA+", // Contains +
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA/", // Contains /
        "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=", // Contains =
      ];

      for (const token of invalidTokens) {
        expect(() => InputValidation.validateAuthToken(token)).toThrow();
        try {
          InputValidation.validateAuthToken(token);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_AUTH_TOKEN);
          expect(error.message).toBe(
            "Auth token must be a 43-character base64url string (server-generated format)",
          );
        }
      }
    });
  });

  describe("validateTTL", () => {
    test("accepts valid TTL values", () => {
      const validTtls = [1, 60, 3600, 86400, 604800];

      for (const ttl of validTtls) {
        expect(() => InputValidation.validateTTL(ttl)).not.toThrow();
      }
    });

    test("rejects invalid TTL values", () => {
      const invalidTtls = [0, -1, -3600, 1.5, NaN, Infinity];

      for (const ttl of invalidTtls) {
        expect(() => InputValidation.validateTTL(ttl)).toThrow();
        try {
          InputValidation.validateTTL(ttl);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_TTL);
        }
      }
    });

    test("rejects non-number TTL values", () => {
      const invalidInputs = ["60", null, undefined, {}, [], true];

      for (const input of invalidInputs) {
        expect(() => InputValidation.validateTTL(input as any)).toThrow();
        try {
          InputValidation.validateTTL(input as any);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_TTL);
          expect(error.message).toBe("TTL must be a number");
        }
      }
    });
  });
});
