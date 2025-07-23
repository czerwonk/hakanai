/**
 * Tests for ShareData validation error translations
 */

import {
  ShareData,
  ShareDataError,
  ShareDataValidationError,
} from "../../server/src/typescript/core/types";

// Mock i18n for testing
const mockI18n = {
  t: (key: string) => {
    const translations: { [key: string]: string } = {
      "validation.MISSING_DATA": "Missing or invalid data field",
      "validation.INVALID_FILENAME": "Invalid filename field - must be text",
      "validation.INVALID_TOKEN": "Invalid token field - must be text",
      "validation.INVALID_TTL":
        "Invalid expiration time - must be a positive number",
      "validation.EMPTY_JSON": "Clipboard content is empty",
      "validation.INVALID_JSON_FORMAT":
        "Invalid clipboard format - not valid JSON",
    };
    return translations[key] || key;
  },
};

// Mock window.i18n globally
declare global {
  interface Window {
    i18n: typeof mockI18n;
  }
}

// Set up mock before tests
beforeAll(() => {
  Object.defineProperty(window, "i18n", {
    value: mockI18n,
    writable: true,
  });
});

describe("ShareData Validation Error Translations", () => {
  test("throws ShareDataError with correct code for missing data", () => {
    try {
      new ShareData("");
      fail("Expected ShareDataError to be thrown");
    } catch (error) {
      expect(error).toBeInstanceOf(ShareDataError);
      expect((error as ShareDataError).code).toBe(
        ShareDataValidationError.MISSING_DATA,
      );
      expect((error as ShareDataError).message).toBe(
        'Missing or invalid "data" field',
      );
    }
  });

  test("throws ShareDataError with correct code for invalid filename", () => {
    try {
      new ShareData("test-data", 123 as any); // Invalid filename type
      fail("Expected ShareDataError to be thrown");
    } catch (error) {
      expect(error).toBeInstanceOf(ShareDataError);
      expect((error as ShareDataError).code).toBe(
        ShareDataValidationError.INVALID_FILENAME,
      );
      expect((error as ShareDataError).message).toBe(
        'Invalid "filename" field - must be string',
      );
    }
  });

  test("throws ShareDataError with correct code for invalid token", () => {
    try {
      new ShareData("test-data", undefined, 123 as any); // Invalid token type
      fail("Expected ShareDataError to be thrown");
    } catch (error) {
      expect(error).toBeInstanceOf(ShareDataError);
      expect((error as ShareDataError).code).toBe(
        ShareDataValidationError.INVALID_TOKEN,
      );
      expect((error as ShareDataError).message).toBe(
        'Invalid "token" field - must be string',
      );
    }
  });

  test("throws ShareDataError with correct code for invalid TTL", () => {
    try {
      new ShareData("test-data", undefined, undefined, -1); // Invalid TTL
      fail("Expected ShareDataError to be thrown");
    } catch (error) {
      expect(error).toBeInstanceOf(ShareDataError);
      expect((error as ShareDataError).code).toBe(
        ShareDataValidationError.INVALID_TTL,
      );
      expect((error as ShareDataError).message).toBe(
        'Invalid "ttl" field - must be positive number',
      );
    }
  });

  test("throws ShareDataError with correct code for empty JSON", () => {
    try {
      ShareData.fromJSON("");
      fail("Expected ShareDataError to be thrown");
    } catch (error) {
      expect(error).toBeInstanceOf(ShareDataError);
      expect((error as ShareDataError).code).toBe(
        ShareDataValidationError.EMPTY_JSON,
      );
      expect((error as ShareDataError).message).toBe("JSON string is empty");
    }
  });

  test("throws ShareDataError with correct code for invalid JSON format", () => {
    try {
      ShareData.fromJSON("{ invalid json }");
      fail("Expected ShareDataError to be thrown");
    } catch (error) {
      expect(error).toBeInstanceOf(ShareDataError);
      expect((error as ShareDataError).code).toBe(
        ShareDataValidationError.INVALID_JSON_FORMAT,
      );
      expect((error as ShareDataError).message).toBe("Invalid JSON format");
    }
  });

  test("validation error can be translated using error codes", () => {
    try {
      new ShareData("");
      fail("Expected ShareDataError to be thrown");
    } catch (error) {
      expect(error).toBeInstanceOf(ShareDataError);
      const shareError = error as ShareDataError;

      // Test that we can translate the error using the code
      const translationKey = `validation.${shareError.code}`;
      const translatedMessage = window.i18n.t(translationKey);

      expect(translatedMessage).toBe("Missing or invalid data field");
      expect(translationKey).toBe("validation.MISSING_DATA");
    }
  });

  test("creates ShareData successfully with valid data", () => {
    const shareData = new ShareData("test-data", "test.txt", "token123", 3600);

    expect(shareData.data).toBe("test-data");
    expect(shareData.filename).toBe("test.txt");
    expect(shareData.token).toBe("token123");
    expect(shareData.ttl).toBe(3600);
  });

  test("creates ShareData successfully from valid JSON", () => {
    const json =
      '{"data": "test-data", "filename": "test.txt", "token": "token123", "ttl": 3600}';
    const shareData = ShareData.fromJSON(json);

    expect(shareData.data).toBe("test-data");
    expect(shareData.filename).toBe("test.txt");
    expect(shareData.token).toBe("token123");
    expect(shareData.ttl).toBe(3600);
  });
});

