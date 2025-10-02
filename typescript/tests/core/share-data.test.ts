// SPDX-License-Identifier: Apache-2.0

/**
 * Tests for ShareData class validation and parsing
 */

import { ShareData } from "../../src/core/share-data";

describe("ShareData", () => {
  describe("constructor and validation", () => {
    test("creates valid ShareData with required data only", () => {
      const shareData = new ShareData("SGVsbG8gV29ybGQ=");
      expect(shareData.data).toBe("SGVsbG8gV29ybGQ=");
      expect(shareData.filename).toBeUndefined();
      expect(shareData.token).toBeUndefined();
      expect(shareData.ttl).toBeUndefined();
    });

    test("creates valid ShareData with all fields", () => {
      const shareData = new ShareData("SGVsbG8gV29ybGQ=", "test.txt", "token123", 3600, undefined);
      expect(shareData.data).toBe("SGVsbG8gV29ybGQ=");
      expect(shareData.filename).toBe("test.txt");
      expect(shareData.token).toBe("token123");
      expect(shareData.ttl).toBe(3600);
    });

    test("throws error for missing data", () => {
      expect(() => new ShareData("")).toThrow('Missing or invalid "data" field');
      expect(() => new ShareData(null as any)).toThrow('Missing or invalid "data" field');
      expect(() => new ShareData(123 as any)).toThrow('Missing or invalid "data" field');
    });

    test("throws error for invalid filename type", () => {
      expect(() => new ShareData("data", 123 as any)).toThrow('Invalid "filename" field - must be string');
    });

    test("throws error for invalid token type", () => {
      expect(() => new ShareData("data", undefined, 123 as any)).toThrow('Invalid "token" field - must be string');
    });

    test("throws error for invalid ttl", () => {
      expect(() => new ShareData("data", undefined, undefined, "invalid" as any)).toThrow(
        'Invalid "ttl" field - must be positive number',
      );
      expect(() => new ShareData("data", undefined, undefined, -1)).toThrow(
        'Invalid "ttl" field - must be positive number',
      );
      expect(() => new ShareData("data", undefined, undefined, 0)).toThrow(
        'Invalid "ttl" field - must be positive number',
      );
    });
  });

  describe("fromJSON", () => {
    test("parses valid JSON with all fields", () => {
      const json = JSON.stringify({
        data: "SGVsbG8gV29ybGQ=",
        filename: "test.txt",
        token: "token123",
        ttl: 3600,
        restrictions: {
          passphrase: "test",
        },
      });

      const shareData = ShareData.fromJSON(json);
      expect(shareData.data).toBe("SGVsbG8gV29ybGQ=");
      expect(shareData.filename).toBe("test.txt");
      expect(shareData.token).toBe("token123");
      expect(shareData.ttl).toBe(3600);
      expect(shareData.restrictions?.passphrase).toBe("test");
    });

    test("parses valid JSON with minimal data", () => {
      const json = JSON.stringify({ data: "SGVsbG8=" });

      const shareData = ShareData.fromJSON(json);
      expect(shareData.data).toBe("SGVsbG8=");
      expect(shareData.filename).toBeUndefined();
      expect(shareData.token).toBeUndefined();
      expect(shareData.ttl).toBeUndefined();
    });

    test("throws error for empty string", () => {
      expect(() => ShareData.fromJSON("")).toThrow("JSON string is empty");
      expect(() => ShareData.fromJSON("   ")).toThrow("JSON string is empty");
    });

    test("throws error for invalid JSON", () => {
      expect(() => ShareData.fromJSON("invalid json")).toThrow("Invalid JSON format");
      expect(() => ShareData.fromJSON("{incomplete")).toThrow("Invalid JSON format");
    });

    test("throws validation error for invalid data", () => {
      const invalidJson = JSON.stringify({ filename: "test.txt" }); // missing data
      expect(() => ShareData.fromJSON(invalidJson)).toThrow('Missing or invalid "data" field');
    });
  });

  describe("getContentSize", () => {
    test("calculates content size correctly", () => {
      // Base64 formula: Math.ceil((base64Length * 3) / 4)
      const testCases = [
        { data: "SGVsbG8=", expectedSize: 6 }, // 8 chars -> 6 bytes
        { data: "SGVsbG8gV29ybGQ=", expectedSize: 12 }, // 16 chars -> 12 bytes
        { data: "dGVzdA==", expectedSize: 6 }, // 8 chars -> 6 bytes
        { data: "dA==", expectedSize: 3 }, // 4 chars -> 3 bytes (avoiding empty data which fails validation)
      ];

      testCases.forEach(({ data, expectedSize }) => {
        const shareData = new ShareData(data);
        expect(shareData.getContentSize()).toBe(expectedSize);
      });
    });
  });

  describe("readonly properties", () => {
    test("properties are accessible", () => {
      const shareData = new ShareData("test", "file.txt", "token", 3600);

      // Verify all properties are accessible
      expect(shareData.data).toBe("test");
      expect(shareData.filename).toBe("file.txt");
      expect(shareData.token).toBe("token");
      expect(shareData.ttl).toBe(3600);

      // Note: readonly is a TypeScript compile-time feature,
      // it doesn't prevent runtime modification in JavaScript
    });
  });
});
