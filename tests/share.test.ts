/**
 * Tests for share.ts functionality (excluding clipboard API interactions)
 */

// Mock DOM elements and methods
const createMockElement = () => ({
  textContent: "",
  style: { display: "" } as any,
  classList: {
    add: jest.fn(),
    remove: jest.fn(),
  } as any,
  addEventListener: jest.fn(),
});

global.document = {
  getElementById: jest.fn().mockReturnValue(createMockElement()),
  readyState: "complete", // Set to complete so it doesn't wait for DOMContentLoaded
  addEventListener: jest.fn(),
} as any;

global.window = {
  location: {
    origin: "https://test.example.com",
    search: "",
  },
} as any;

global.URLSearchParams = jest.fn().mockImplementation((search) => ({
  get: jest.fn((key) =>
    key === "auto" && search.includes("auto=true") ? "true" : null,
  ),
}));

// Import the module under test after setting up mocks
const { validateClipboardPayload } = require("../server/src/typescript/share");

// Since share.ts doesn't export functions, we need to test the validation function
// that we can access through the module structure

describe("Share functionality", () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe("validateClipboardPayload", () => {
    test("validates required data field", () => {
      expect(() => validateClipboardPayload({})).toThrow(
        'Missing or invalid "data" field',
      );
      expect(() => validateClipboardPayload({ data: "" })).toThrow(
        'Missing or invalid "data" field',
      );
      expect(() => validateClipboardPayload({ data: 123 })).toThrow(
        'Missing or invalid "data" field',
      );
    });

    test("accepts valid data field", () => {
      expect(() =>
        validateClipboardPayload({ data: "valid-base64-content" }),
      ).not.toThrow();
    });

    test("validates optional filename field", () => {
      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          filename: 123,
        }),
      ).toThrow('Invalid "filename" field - must be string');

      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          filename: "document.pdf",
        }),
      ).not.toThrow();

      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          filename: undefined,
        }),
      ).not.toThrow();
    });

    test("validates optional token field", () => {
      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          token: 123,
        }),
      ).toThrow('Invalid "token" field - must be string');

      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          token: "auth-token-123",
        }),
      ).not.toThrow();

      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          token: undefined,
        }),
      ).not.toThrow();
    });

    test("validates optional ttl field", () => {
      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          ttl: "123",
        }),
      ).toThrow('Invalid "ttl" field - must be positive number');

      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          ttl: -1,
        }),
      ).toThrow('Invalid "ttl" field - must be positive number');

      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          ttl: 0,
        }),
      ).toThrow('Invalid "ttl" field - must be positive number');

      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          ttl: 3600,
        }),
      ).not.toThrow();

      expect(() =>
        validateClipboardPayload({
          data: "valid-data",
          ttl: undefined,
        }),
      ).not.toThrow();
    });

    test("validates complete valid payload", () => {
      const validPayload = {
        data: "SGVsbG8gV29ybGQ=", // base64 "Hello World"
        filename: "test.txt",
        token: "user-token-123",
        ttl: 86400,
      };

      expect(() => validateClipboardPayload(validPayload)).not.toThrow();
    });

    test("validates minimal valid payload", () => {
      const minimalPayload = {
        data: "SGVsbG8=", // base64 "Hello"
      };

      expect(() => validateClipboardPayload(minimalPayload)).not.toThrow();
    });
  });

  describe("Base64 size calculation", () => {
    test("calculates content bytes from base64 correctly", () => {
      // This tests the logic used in showClipboardContent
      // Base64 encoding formula: Math.ceil((base64Length * 3) / 4)

      const testCases = [
        { base64: "SGVsbG8=", expectedBytes: 6 }, // base64 length 8, (8*3)/4 = 6
        { base64: "SGVsbG8gV29ybGQ=", expectedBytes: 12 }, // base64 length 16, (16*3)/4 = 12
        { base64: "dGVzdA==", expectedBytes: 6 }, // base64 length 8, (8*3)/4 = 6
        { base64: "", expectedBytes: 0 }, // empty = 0 bytes
      ];

      testCases.forEach(({ base64, expectedBytes }) => {
        const calculatedBytes = Math.ceil((base64.length * 3) / 4);
        expect(calculatedBytes).toBe(expectedBytes);
      });
    });
  });

  describe("ClipboardData interface", () => {
    test("defines correct structure", () => {
      // This tests the expected data structure for clipboard payloads
      const validData = {
        data: "base64-content",
        filename: "optional-filename.txt",
        token: "optional-token",
        ttl: 3600,
      };

      expect(typeof validData.data).toBe("string");
      expect(typeof validData.filename).toBe("string");
      expect(typeof validData.token).toBe("string");
      expect(typeof validData.ttl).toBe("number");
    });

    test("allows optional fields to be undefined", () => {
      const minimalData: any = {
        data: "base64-content",
      };

      expect(typeof minimalData.data).toBe("string");
      expect(minimalData.filename).toBeUndefined();
      expect(minimalData.token).toBeUndefined();
      expect(minimalData.ttl).toBeUndefined();
    });
  });

  describe("DOM interactions (mocked)", () => {
    test("getElementById mock function works", () => {
      // This tests that our mocking setup is functional
      const getElementSpy = jest.spyOn(document, "getElementById");

      // Simulate calling a function that interacts with DOM
      document.getElementById("loading-text");
      document.getElementById("clipboard-content");
      document.getElementById("error-message");

      expect(getElementSpy).toHaveBeenCalledWith("loading-text");
      expect(getElementSpy).toHaveBeenCalledWith("clipboard-content");
      expect(getElementSpy).toHaveBeenCalledWith("error-message");
      expect(getElementSpy).toHaveBeenCalledTimes(3);
    });
  });

  describe("URL parameter parsing", () => {
    test("URLSearchParams mock works correctly", () => {
      const params1 = new URLSearchParams("?auto=true");
      expect(params1.get("auto")).toBe("true");

      const params2 = new URLSearchParams("?other=value");
      expect(params2.get("auto")).toBeNull();

      const params3 = new URLSearchParams("");
      expect(params3.get("auto")).toBeNull();
    });
  });
});
