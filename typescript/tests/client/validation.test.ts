// SPDX-License-Identifier: Apache-2.0

/**
 * Input validation tests for all validation functions
 */

import { HakanaiErrorCodes } from "../../src/hakanai-client";

describe("InputValidation", () => {
  const { InputValidation } = require("../../src/hakanai-client") as any;

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

  describe("validateCountryCode", () => {
    test("accepts valid ISO 3166-1 alpha-2 country codes", () => {
      const validCountries = [
        "US",
        "DE",
        "CA",
        "GB",
        "FR",
        "JP",
        "AU",
        "BR",
        "CN",
        "IN",
      ];

      for (const country of validCountries) {
        expect(() =>
          InputValidation.validateCountryCode(country),
        ).not.toThrow();
      }
    });

    test("rejects invalid country code formats", () => {
      const invalidCountries = [
        "usa", // lowercase
        "USA", // 3 letters
        "germany", // full country name
        "1A", // number
        "U", // too short
        "us", // lowercase
        "U$", // special character
        "12", // numbers
      ];

      for (const country of invalidCountries) {
        expect(() => InputValidation.validateCountryCode(country)).toThrow();
        try {
          InputValidation.validateCountryCode(country);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
          expect(error.message).toContain("Invalid country code");
        }
      }
    });

    test("rejects empty country codes with specific message", () => {
      const emptyCountries = ["", "   "];

      for (const country of emptyCountries) {
        expect(() => InputValidation.validateCountryCode(country)).toThrow();
        try {
          InputValidation.validateCountryCode(country);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
          expect(error.message).toBe("Country code must be a non-empty string");
        }
      }
    });

    test("rejects non-string country values", () => {
      const invalidInputs = [null, undefined, 123, {}, [], true, false];

      for (const input of invalidInputs) {
        expect(() =>
          InputValidation.validateCountryCode(input as any),
        ).toThrow();
        try {
          InputValidation.validateCountryCode(input as any);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
          expect(error.message).toBe("Country code must be a non-empty string");
        }
      }
    });
  });

  describe("validateASN", () => {
    test("accepts valid ASN numbers", () => {
      const validASNs = [
        1, // Minimum valid ASN
        13335, // Cloudflare
        15169, // Google
        16509, // Amazon
        32934, // Facebook
        64512, // Start of private 16-bit range
        65534, // End of private 16-bit range
        100000, // Valid ASN
        4200000000, // Start of private 32-bit range
        4294967294, // End of private 32-bit range
        4294967295, // Maximum valid ASN (2^32 - 1)
      ];

      for (const asn of validASNs) {
        expect(() => InputValidation.validateASN(asn)).not.toThrow();
      }
    });

    test("rejects ASN 0 as reserved", () => {
      expect(() => InputValidation.validateASN(0)).toThrow();
      try {
        InputValidation.validateASN(0);
      } catch (error: any) {
        expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
        expect(error.message).toBe("ASN 0 is reserved and cannot be used");
      }
    });

    test("rejects negative ASN numbers", () => {
      const negativeASNs = [-1, -100, -65535, -4294967295];

      for (const asn of negativeASNs) {
        expect(() => InputValidation.validateASN(asn)).toThrow();
        try {
          InputValidation.validateASN(asn);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
          expect(error.message).toContain("Must be between 1 and 4294967295");
        }
      }
    });

    test("rejects ASN numbers above maximum (2^32)", () => {
      const oversizedASNs = [
        4294967296,
        4294967297,
        5000000000,
        Number.MAX_SAFE_INTEGER,
      ];

      for (const asn of oversizedASNs) {
        expect(() => InputValidation.validateASN(asn)).toThrow();
        try {
          InputValidation.validateASN(asn);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
          expect(error.message).toContain("Must be between 1 and 4294967295");
        }
      }
    });

    test("rejects non-integer ASN values", () => {
      const nonIntegerASNs = [1.5, 100.1, 65535.999, Math.PI];

      for (const asn of nonIntegerASNs) {
        expect(() => InputValidation.validateASN(asn)).toThrow();
        try {
          InputValidation.validateASN(asn);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
          expect(error.message).toBe("ASN must be an integer");
        }
      }
    });

    test("rejects non-number ASN values", () => {
      const invalidInputs = ["13335", null, undefined, {}, [], true, false];

      for (const input of invalidInputs) {
        expect(() => InputValidation.validateASN(input as any)).toThrow();
        try {
          InputValidation.validateASN(input as any);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
          expect(error.message).toBe("ASN must be a number");
        }
      }
    });

    test("accepts private ASN ranges without error", () => {
      // Private ASN ranges should be accepted (logging can happen server-side)
      const privateASNs = [
        64512, // Start of 16-bit private range
        65000, // Middle of 16-bit private range
        65534, // End of 16-bit private range
        4200000000, // Start of 32-bit private range
        4250000000, // Middle of 32-bit private range
        4294967294, // End of 32-bit private range
      ];

      for (const asn of privateASNs) {
        expect(() => InputValidation.validateASN(asn)).not.toThrow();
      }
    });
  });

  describe("validateRestrictions", () => {
    test("accepts valid restrictions object with allowed_ips", () => {
      const validRestrictions = [
        { allowed_ips: ["192.168.1.1"] },
        { allowed_ips: ["10.0.0.0/8", "172.16.0.0/12"] },
        { allowed_ips: ["2001:db8::/32"] },
        { allowed_ips: ["127.0.0.1", "::1"] },
        { allowed_ips: ["192.168.1.0/24", "2001:db8:85a3::8a2e:370:7334"] },
        {},
        { allowed_ips: [] },
      ];

      for (const restrictions of validRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).not.toThrow();
      }
    });

    test("accepts valid restrictions object with allowed_countries", () => {
      const validRestrictions = [
        { allowed_countries: ["US"] },
        { allowed_countries: ["DE", "FR"] },
        { allowed_countries: ["US", "CA", "GB"] },
        { allowed_countries: [] },
      ];

      for (const restrictions of validRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).not.toThrow();
      }
    });

    test("accepts valid restrictions object with allowed_asns", () => {
      const validRestrictions = [
        { allowed_asns: [13335] }, // Cloudflare
        { allowed_asns: [15169, 16509] }, // Google and Amazon
        { allowed_asns: [1, 65535, 4294967295] }, // Min, mid, max
        { allowed_asns: [] },
      ];

      for (const restrictions of validRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).not.toThrow();
      }
    });

    test("accepts valid restrictions object with all restriction types", () => {
      const validRestrictions = [
        {
          allowed_ips: ["192.168.1.1"],
          allowed_countries: ["US"],
          allowed_asns: [13335],
        },
        {
          allowed_ips: ["10.0.0.0/8", "172.16.0.0/12"],
          allowed_countries: ["DE", "FR"],
          allowed_asns: [15169, 16509],
        },
        {
          allowed_ips: ["2001:db8::/32"],
          allowed_countries: ["CA"],
          allowed_asns: [32934],
        },
      ];

      for (const restrictions of validRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).not.toThrow();
      }
    });

    test("rejects invalid allowed_countries array types", () => {
      const invalidRestrictions = [
        { allowed_countries: "US" },
        { allowed_countries: 123 },
        { allowed_countries: {} },
        { allowed_countries: true },
      ];

      for (const restrictions of invalidRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).toThrow();
        try {
          InputValidation.validateRestrictions(restrictions);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
        }
      }
    });

    test("rejects invalid allowed_asns array types", () => {
      const invalidRestrictions = [
        { allowed_asns: 13335 }, // Not an array
        { allowed_asns: "13335" }, // String instead of array
        { allowed_asns: {} },
        { allowed_asns: true },
      ];

      for (const restrictions of invalidRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).toThrow();
        try {
          InputValidation.validateRestrictions(restrictions);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
          expect(error.message).toBe("allowed_asns must be an array");
        }
      }
    });

    test("rejects invalid ASN values in allowed_asns", () => {
      const invalidRestrictions = [
        { allowed_asns: [0] }, // ASN 0 is reserved
        { allowed_asns: [-1] }, // Negative ASN
        { allowed_asns: [4294967296] }, // Above max (2^32)
        { allowed_asns: [1.5] }, // Non-integer
        { allowed_asns: ["13335" as any] }, // String instead of number
        { allowed_asns: [13335, 0] }, // Mix of valid and invalid
        { allowed_asns: [null as any] },
        { allowed_asns: [undefined as any] },
      ];

      for (const restrictions of invalidRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).toThrow();
        try {
          InputValidation.validateRestrictions(restrictions);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
        }
      }
    });

    test("rejects invalid country codes in allowed_countries", () => {
      const invalidRestrictions = [
        { allowed_countries: [""] },
        { allowed_countries: ["   "] },
        { allowed_countries: ["usa"] }, // lowercase
        { allowed_countries: ["USA"] }, // 3 letters
        { allowed_countries: ["US", "invalid"] },
        { allowed_countries: [123 as any] },
        { allowed_countries: [null as any] },
      ];

      for (const restrictions of invalidRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).toThrow();
        try {
          InputValidation.validateRestrictions(restrictions);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
        }
      }
    });

    test("rejects invalid restrictions object types", () => {
      const invalidInputs = [null, "string", 123, [], true, false];

      for (const input of invalidInputs) {
        try {
          InputValidation.validateRestrictions(input);
          fail(
            `Expected input ${JSON.stringify(input)} to throw, but it didn't`,
          );
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
        }
      }
    });

    test("rejects invalid allowed_ips array types", () => {
      const invalidRestrictions = [
        { allowed_ips: "192.168.1.1" },
        { allowed_ips: 123 },
        { allowed_ips: {} },
        { allowed_ips: true },
      ];

      for (const restrictions of invalidRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).toThrow();
        try {
          InputValidation.validateRestrictions(restrictions);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
          expect(error.message).toBe("allowed_ips must be an array");
        }
      }
    });

    test("rejects invalid IP addresses in allowed_ips", () => {
      const invalidRestrictions = [
        { allowed_ips: [""] },
        { allowed_ips: ["   "] },
        { allowed_ips: ["not-an-ip"] },
        { allowed_ips: ["192.168.1.1", "invalid-ip"] },
        { allowed_ips: [123 as any] },
        { allowed_ips: [null as any] },
      ];

      for (const restrictions of invalidRestrictions) {
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).toThrow();
        try {
          InputValidation.validateRestrictions(restrictions);
        } catch (error: any) {
          expect(error.code).toBe(HakanaiErrorCodes.INVALID_RESTRICTIONS);
        }
      }
    });

    test("accepts various valid IPv4 and IPv6 formats", () => {
      const validIps = [
        "192.168.1.1",
        "10.0.0.1",
        "127.0.0.1",
        "192.168.1.0/24",
        "10.0.0.0/8",
        "::1",
        "2001:db8::1",
        "2001:db8::/32",
      ];

      for (const ip of validIps) {
        const restrictions = { allowed_ips: [ip] };
        expect(() =>
          InputValidation.validateRestrictions(restrictions),
        ).not.toThrow();
      }
    });
  });
});
