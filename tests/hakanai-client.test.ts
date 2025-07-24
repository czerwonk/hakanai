/**
 * Integration tests for Hakanai TypeScript client
 * Tests real crypto operations with minimal mocking
 */

import {
  HakanaiClient,
  HakanaiErrorCodes,
  Base64UrlSafe,
  ContentAnalysis,
} from "../server/src/typescript/hakanai-client";

// Helper function to ensure we get proper Uint8Array in tests
function encodeText(text: string): Uint8Array {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(text);
  return new Uint8Array(encoded);
}

// Helper function to generate valid UUID v4
function generateUUID(): string {
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function(c) {
    const r = Math.random() * 16 | 0;
    const v = c == 'x' ? r : (r & 0x3 | 0x8);
    return v.toString(16);
  });
}

// Mock server responses only
const createMockFetch = () => {
  const mockSecrets = new Map<string, string>();

  return jest.fn((url: string, options?: any) => {
    const urlObj = new URL(url);

    // POST /api/v1/secret - create secret
    if (urlObj.pathname === "/api/v1/secret" && options?.method === "POST") {
      const secretId = generateUUID();
      const body = JSON.parse(options.body);
      mockSecrets.set(secretId, body.data);

      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ id: secretId }),
      });
    }

    // GET /api/v1/secret/{id} - retrieve secret
    const getMatch = urlObj.pathname.match(/^\/api\/v1\/secret\/(.+)$/);
    if (getMatch && (!options?.method || options.method === "GET")) {
      const secretId = getMatch[1];
      const encryptedData = mockSecrets.get(secretId);

      if (!encryptedData) {
        return Promise.resolve({
          ok: false,
          status: 404,
          statusText: "Not Found",
        });
      }

      return Promise.resolve({
        ok: true,
        text: () => Promise.resolve(encryptedData),
      });
    }

    return Promise.resolve({
      ok: false,
      status: 404,
      statusText: "Not Found",
    });
  });
};

describe("Base64UrlSafe", () => {
  test("encode and decode round trip with text", () => {
    const original = "Hello, World! 🌍";
    const encoder = new TextEncoder();
    const bytes = encoder.encode(original);

    // Convert to Uint8Array if needed (Node.js TextEncoder returns different type)
    const uint8Array =
      bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);

    const encoded = Base64UrlSafe.encode(uint8Array);
    const decoded = Base64UrlSafe.decode(encoded);

    const decoder = new TextDecoder();
    const result = decoder.decode(decoded);

    expect(result).toBe(original);
  });

  test("encode produces URL-safe characters", () => {
    const testBytes = new Uint8Array([255, 254, 253, 252, 251, 250]);
    const encoded = Base64UrlSafe.encode(testBytes);

    // Should not contain +, /, or = characters
    expect(encoded).not.toMatch(/[+/=]/);
    // Should only contain URL-safe characters
    expect(encoded).toMatch(/^[A-Za-z0-9_-]*$/);
  });

  test("decode handles padding correctly", () => {
    const testBytes = new Uint8Array([1, 2, 3, 4, 5]);
    const encoded = Base64UrlSafe.encode(testBytes);
    const decoded = Base64UrlSafe.decode(encoded);

    expect(Array.from(decoded)).toEqual([1, 2, 3, 4, 5]);
  });

  test("encodeText and decodeText convenience methods", () => {
    const original = "Test string with special chars: åëïöü";
    const encoded = Base64UrlSafe.encodeText(original);
    const decoded = Base64UrlSafe.decodeText(encoded);

    expect(decoded).toBe(original);
  });
});

describe("ContentAnalysis", () => {
  test("isBinary detects null bytes correctly", () => {
    // Text content without null bytes
    const textBytes = encodeText("Hello, world!");
    expect(ContentAnalysis.isBinary(textBytes)).toBe(false);

    // Binary content with null bytes
    const binaryBytes = new Uint8Array([0x00, 0x01, 0x02, 0xff]);
    expect(ContentAnalysis.isBinary(binaryBytes)).toBe(true);
  });

  test("isBinary returns false for empty arrays", () => {
    const emptyBytes = new Uint8Array(0);
    expect(ContentAnalysis.isBinary(emptyBytes)).toBe(false);
  });

  test("isBinary detects null bytes anywhere in the array", () => {
    // Null byte at the beginning
    const startNull = new Uint8Array([0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    expect(ContentAnalysis.isBinary(startNull)).toBe(true);

    // Null byte in the middle
    const middleNull = new Uint8Array([0x48, 0x65, 0x00, 0x6c, 0x6c, 0x6f]);
    expect(ContentAnalysis.isBinary(middleNull)).toBe(true);

    // Null byte at the end
    const endNull = new Uint8Array([0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x00]);
    expect(ContentAnalysis.isBinary(endNull)).toBe(true);
  });

  test("isBinary handles UTF-8 text correctly", () => {
    const utf8Text = encodeText("Hello, 世界! 🌍");
    expect(ContentAnalysis.isBinary(utf8Text)).toBe(false);

    const specialChars = encodeText("åëïöü äöü ñ");
    expect(ContentAnalysis.isBinary(specialChars)).toBe(false);
  });

  test("isBinary handles typical binary file headers", () => {
    // PNG file signature
    const pngHeader = new Uint8Array([
      0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
    ]);
    expect(ContentAnalysis.isBinary(pngHeader)).toBe(false); // PNG header doesn't contain null bytes

    // Add null byte to simulate binary content
    const binaryWithNull = new Uint8Array([
      0x89, 0x50, 0x4e, 0x47, 0x00, 0x0a, 0x1a, 0x0a,
    ]);
    expect(ContentAnalysis.isBinary(binaryWithNull)).toBe(true);
  });

  test("isBinary validates input type", () => {
    expect(() => ContentAnalysis.isBinary("not a uint8array" as any)).toThrow(
      "Input must be a Uint8Array",
    );
    expect(() => ContentAnalysis.isBinary(null as any)).toThrow(
      "Input must be a Uint8Array",
    );
    expect(() => ContentAnalysis.isBinary(undefined as any)).toThrow(
      "Input must be a Uint8Array",
    );
    expect(() => ContentAnalysis.isBinary([1, 2, 3] as any)).toThrow(
      "Input must be a Uint8Array",
    );
  });

  test("isBinary handles large arrays efficiently", () => {
    // Create a large text array without null bytes
    const largeText = encodeText("x".repeat(10000));
    expect(ContentAnalysis.isBinary(largeText)).toBe(false);

    // Create a large array with a null byte somewhere in the middle
    const largeWithNull = new Uint8Array(10000);
    largeWithNull.fill(65); // Fill with 'A' (ASCII 65)
    largeWithNull[5000] = 0; // Add null byte in the middle
    expect(ContentAnalysis.isBinary(largeWithNull)).toBe(true);
  });
});

describe("HakanaiClient Integration", () => {
  let client: HakanaiClient;
  let originalFetch: any;

  beforeEach(() => {
    originalFetch = global.fetch;
    global.fetch = createMockFetch() as any;
    client = new HakanaiClient("http://localhost:8080");
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  test("complete roundtrip: send and receive text secret", async () => {
    const originalText =
      "This is a secret message that should roundtrip correctly! 🔐";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload();
    originalPayload.setFromBytes!(textBytes);

    // Send the secret
    const secretUrl = await client.sendPayload(originalPayload, 3600);

    expect(secretUrl).toMatch(
      /^http:\/\/localhost:8080\/s\/[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}#[A-Za-z0-9_-]+$/i,
    );

    // Receive the secret
    const retrievedPayload = await client.receivePayload(secretUrl);

    expect(retrievedPayload.decode!()).toBe(originalText);
    expect(retrievedPayload.filename).toBeUndefined();
  });

  test("complete roundtrip: send and receive file secret", async () => {
    const originalText = "Binary file content or any text treated as file";
    const filename = "test-document.txt";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload(filename);
    originalPayload.setFromBytes!(textBytes);

    // Send the secret
    const secretUrl = await client.sendPayload(originalPayload, 1800);

    expect(secretUrl).toMatch(
      /^http:\/\/localhost:8080\/s\/[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}#[A-Za-z0-9_-]+$/i,
    );

    // Receive the secret
    const retrievedPayload = await client.receivePayload(secretUrl);

    expect(retrievedPayload.decode!()).toBe(originalText);
    expect(retrievedPayload.filename).toBe(filename);
  });

  test("roundtrip with special characters and unicode", async () => {
    const originalText = "Special chars: åëïöü 中文 🚀 \n\t\r\"'\\\x00\xFF";
    const filename = "unicode-file-名前.txt";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload(filename);
    originalPayload.setFromBytes!(textBytes);

    const secretUrl = await client.sendPayload(originalPayload);
    const retrievedPayload = await client.receivePayload(secretUrl);

    expect(retrievedPayload.decode!()).toBe(originalText);
    expect(retrievedPayload.filename).toBe(filename);
  });

  test("roundtrip with empty filename (null handling)", async () => {
    const originalText = "Secret without filename";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload();
    originalPayload.setFromBytes!(textBytes);

    const secretUrl = await client.sendPayload(originalPayload);
    const retrievedPayload = await client.receivePayload(secretUrl);

    expect(retrievedPayload.decode!()).toBe(originalText);
    expect(retrievedPayload.filename).toBeUndefined();
  });

  test("large payload roundtrip", async () => {
    // Create a larger payload to test chunked base64 processing
    const largeData = "x".repeat(10000) + " end marker";
    const filename = "large-file.txt";
    const textBytes = encodeText(largeData);

    const originalPayload = client.createPayload(filename);
    originalPayload.setFromBytes!(textBytes);

    const secretUrl = await client.sendPayload(originalPayload);
    const retrievedPayload = await client.receivePayload(secretUrl);

    const decodedData = retrievedPayload.decode!();
    expect(decodedData).toBe(largeData);
    expect(decodedData).toHaveLength(10011);
    expect(decodedData.endsWith(" end marker")).toBe(true);
    expect(retrievedPayload.filename).toBe(filename);
  });

  test("payload data is base64-encoded in internal format", async () => {
    const originalText = "test message";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload();
    originalPayload.setFromBytes!(textBytes);

    // Mock fetch to capture what gets sent
    const sentData: any[] = [];
    global.fetch = jest.fn((url: string, options?: any) => {
      if (options?.method === "POST") {
        const body = JSON.parse(options.body);
        sentData.push(body);
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ id: "test-123" }),
        });
      }
      return Promise.resolve({ ok: false, status: 404 });
    }) as any;

    await client.sendPayload(originalPayload);

    expect(sentData).toHaveLength(1);

    // The encrypted payload should be decryptable
    expect(sentData[0].data).toBeDefined();
    expect(typeof sentData[0].data).toBe("string");
    expect(sentData[0].expires_in).toBe(3600);
  });

  test("URL format matches expected pattern", async () => {
    const textBytes = encodeText("test");

    const payload = client.createPayload();
    payload.setFromBytes!(textBytes);

    const url = await client.sendPayload(payload);

    const urlObj = new URL(url);
    expect(urlObj.protocol).toBe("http:");
    expect(urlObj.hostname).toBe("localhost");
    expect(urlObj.port).toBe("8080");
    expect(urlObj.pathname).toMatch(/^\/s\/[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i);
    expect(urlObj.hash).toMatch(/^#[A-Za-z0-9_-]+$/);

    // Key should be exactly 32 bytes when decoded
    const keyBase64 = urlObj.hash.slice(1);
    const keyBytes = Base64UrlSafe.decode(keyBase64);
    expect(keyBytes).toHaveLength(32);
  });

  test("PayloadData decode() method works correctly", async () => {
    const originalText = "Test message with unicode: 🔐 åëïöü";
    const filename = "test.txt";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload(filename);
    originalPayload.setFromBytes!(textBytes);

    const secretUrl = await client.sendPayload(originalPayload);
    const retrievedPayload = await client.receivePayload(secretUrl);

    // Test the decode() method
    expect(retrievedPayload.decode).toBeDefined();
    const decodedData = retrievedPayload.decode!();
    expect(decodedData).toBe(originalText);
  });

  test("PayloadData decodeBytes() method works correctly", async () => {
    const originalText = "Binary data test";
    const filename = "binary.dat";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload(filename);
    originalPayload.setFromBytes!(textBytes);

    const secretUrl = await client.sendPayload(originalPayload);
    const retrievedPayload = await client.receivePayload(secretUrl);

    // Test the decodeBytes() method
    expect(retrievedPayload.decodeBytes).toBeDefined();
    const decodedBytes = retrievedPayload.decodeBytes!();
    expect(decodedBytes).toBeInstanceOf(Uint8Array);

    // Convert back to string to verify
    const decoder = new TextDecoder();
    const decodedString = decoder.decode(decodedBytes);
    expect(decodedString).toBe(originalText);
  });
});

describe("Error Handling", () => {
  let client: HakanaiClient;

  beforeEach(() => {
    global.fetch = createMockFetch() as any;
    client = new HakanaiClient("http://localhost:8080");
  });

  test("sendPayload validates input", async () => {
    await expect(client.sendPayload({} as any)).rejects.toThrow(
      "Payload data cannot be empty",
    );

    const emptyPayload = client.createPayload();
    // Don't call setFromBytes, so data remains empty
    await expect(client.sendPayload(emptyPayload)).rejects.toThrow(
      "Payload data cannot be empty",
    );

    const testBytes = encodeText("test");
    const validPayload = client.createPayload();
    validPayload.setFromBytes!(testBytes);

    await expect(client.sendPayload(validPayload, 0)).rejects.toThrow(
      "TTL must be a positive number",
    );

    await expect(
      client.sendPayload(validPayload, 3600, 123 as any),
    ).rejects.toThrow("Auth token must be a string");
  });

  test("sendPayload handles 401 authentication required error", async () => {
    global.fetch = jest.fn().mockResolvedValue({
      ok: false,
      status: 401,
      statusText: "Unauthorized",
    }) as any;

    const testBytes = encodeText("test secret");
    const payload = client.createPayload();
    payload.setFromBytes!(testBytes);

    // Use a properly formatted base64url token (43 chars) that will pass client validation but fail server auth
    await expect(
      client.sendPayload(payload, 3600, "oCTJV5YSQEllqpBQ5_4ttyeTJsQxNtgsz3xSGjqP9xw"),
    ).rejects.toThrow(
      "Authentication required: Please provide a valid authentication token",
    );
  });

  test("sendPayload handles 403 invalid token error", async () => {
    global.fetch = jest.fn().mockResolvedValue({
      ok: false,
      status: 403,
      statusText: "Forbidden",
    }) as any;

    const testBytes = encodeText("test secret");
    const payload = client.createPayload();
    payload.setFromBytes!(testBytes);

    // Use a properly formatted base64url token (43 chars) that will pass client validation but fail server auth
    await expect(
      client.sendPayload(payload, 3600, "opBEGjLy_mkCsTbMog4nxnvstB39kNx8K7450KHHH4E"),
    ).rejects.toThrow(
      "Invalid authentication token: Please check your token and try again",
    );
  });

  test("sendPayload throws HakanaiError with correct error codes", async () => {
    // Test 401 error
    global.fetch = jest.fn().mockResolvedValue({
      ok: false,
      status: 401,
      statusText: "Unauthorized",
    }) as any;

    const testBytes = encodeText("test secret");
    const payload = client.createPayload();
    payload.setFromBytes!(testBytes);

    try {
      // Use a properly formatted but invalid token (43 chars base64url)
      await client.sendPayload(payload, 3600, "HUqlqUd68TmqGkNj5o7pMqRcJe2YIQqoOlMfSSYF5r8");
      fail("Expected error to be thrown");
    } catch (error: any) {
      expect(error.name).toBe("HakanaiError");
      expect(error.code).toBe(HakanaiErrorCodes.AUTHENTICATION_REQUIRED);
      expect(error.statusCode).toBe(401);
    }

    // Test 403 error
    global.fetch = jest.fn().mockResolvedValue({
      ok: false,
      status: 403,
      statusText: "Forbidden",
    }) as any;

    try {
      // Use a properly formatted but invalid token (43 chars base64url)
      await client.sendPayload(payload, 3600, "opBEGjLy_mkCsTbMog4nxnvstB39kNx8K7450KHHH4E");
      fail("Expected error to be thrown");
    } catch (error: any) {
      expect(error.name).toBe("HakanaiError");
      expect(error.code).toBe(HakanaiErrorCodes.INVALID_TOKEN);
      expect(error.statusCode).toBe(403);
    }
  });

  test("receivePayload validates URL", async () => {
    await expect(client.receivePayload("")).rejects.toThrow(
      "URL cannot be empty",
    );

    await expect(client.receivePayload("not-a-url")).rejects.toThrow(
      "Invalid URL format",
    );

    await expect(
      client.receivePayload("http://localhost/no-secret-id"),
    ).rejects.toThrow("URL must contain secret ID in format /s/{id}");

    await expect(
      client.receivePayload("http://localhost/s/550e8400-e29b-41d4-a716-446655440000"),
    ).rejects.toThrow("URL must contain decryption key in fragment");
  });

  test("receivePayload handles server errors", async () => {
    global.fetch = jest.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: "Not Found",
    }) as any;

    const client = new HakanaiClient("http://localhost:8080");

    // Use a proper UUID and 32-byte base64 key for the test
    const validUuid = "550e8400-e29b-41d4-a716-446655440000";
    const validKey = Base64UrlSafe.encode(new Uint8Array(32)); // 32 zero bytes
    await expect(
      client.receivePayload("http://localhost:8080/s/" + validUuid + "#" + validKey),
    ).rejects.toThrow("Secret not found or has expired");
  });
});

describe("Error Code Constants", () => {
  test("HakanaiErrorCodes exports all expected constants", () => {
    expect(HakanaiErrorCodes.AUTHENTICATION_REQUIRED).toBe(
      "AUTHENTICATION_REQUIRED",
    );
    expect(HakanaiErrorCodes.INVALID_TOKEN).toBe("INVALID_TOKEN");
    expect(HakanaiErrorCodes.SEND_FAILED).toBe("SEND_FAILED");
    expect(HakanaiErrorCodes.SECRET_NOT_FOUND).toBe("SECRET_NOT_FOUND");
    expect(HakanaiErrorCodes.SECRET_ALREADY_ACCESSED).toBe(
      "SECRET_ALREADY_ACCESSED",
    );
    expect(HakanaiErrorCodes.RETRIEVE_FAILED).toBe("RETRIEVE_FAILED");
    expect(HakanaiErrorCodes.MISSING_DECRYPTION_KEY).toBe(
      "MISSING_DECRYPTION_KEY",
    );
  });

  test("Error codes are readonly constants", () => {
    // This should be a compile-time check, but we can verify the values exist
    const codes = Object.keys(HakanaiErrorCodes);
    expect(codes.length).toBe(25);

    // Verify all expected codes are present (original codes)
    expect(codes).toContain("AUTHENTICATION_REQUIRED");
    expect(codes).toContain("INVALID_TOKEN");
    expect(codes).toContain("SEND_FAILED");
    expect(codes).toContain("SECRET_NOT_FOUND");
    expect(codes).toContain("SECRET_ALREADY_ACCESSED");
    expect(codes).toContain("RETRIEVE_FAILED");
    expect(codes).toContain("MISSING_DECRYPTION_KEY");

    // Verify new validation error codes (all implemented codes)
    expect(codes).toContain("EXPECTED_UINT8_ARRAY");
    expect(codes).toContain("EXPECTED_STRING");
    expect(codes).toContain("INVALID_INPUT_FORMAT");
    expect(codes).toContain("MISSING_KEY");
    expect(codes).toContain("INVALID_KEY");
    expect(codes).toContain("CRYPTO_API_UNAVAILABLE");
    expect(codes).toContain("INVALID_TTL");
    expect(codes).toContain("MISSING_AUTH_TOKEN");
    expect(codes).toContain("INVALID_AUTH_TOKEN");
    expect(codes).toContain("BASE64_ERROR");
    expect(codes).toContain("INVALID_ENCRYPTED_DATA");
    expect(codes).toContain("DECRYPTION_FAILED");
    expect(codes).toContain("INVALID_URL_FORMAT");
    expect(codes).toContain("MISSING_SECRET_ID");
    expect(codes).toContain("INVALID_SECRET_ID");
    expect(codes).toContain("INVALID_PAYLOAD");
    expect(codes).toContain("INVALID_SERVER_RESPONSE");
    expect(codes).toContain("CRYPTO_CONTEXT_DISPOSED");
  });

  test("Error codes can be used for comparison", () => {
    // Test that we can compare against the constants
    function checkErrorCode(code: string): boolean {
      return code === HakanaiErrorCodes.AUTHENTICATION_REQUIRED;
    }

    expect(checkErrorCode(HakanaiErrorCodes.AUTHENTICATION_REQUIRED)).toBe(
      true,
    );
    expect(checkErrorCode(HakanaiErrorCodes.INVALID_TOKEN)).toBe(false);
    expect(checkErrorCode("SOME_OTHER_CODE")).toBe(false);
  });

  test("Validation errors throw HakanaiError with correct codes", () => {
    // Test Base64UrlSafe.encode validation
    try {
      Base64UrlSafe.encode("not a uint8array" as any);
      fail("Expected HakanaiError to be thrown");
    } catch (error: any) {
      expect(error.name).toBe("HakanaiError");
      expect(error.code).toBe(HakanaiErrorCodes.EXPECTED_UINT8_ARRAY);
      expect(error.message).toBe("Input must be a Uint8Array");
    }

    // Test ContentAnalysis.isBinary validation
    try {
      ContentAnalysis.isBinary("not a uint8array" as any);
      fail("Expected HakanaiError to be thrown");
    } catch (error: any) {
      expect(error.name).toBe("HakanaiError");
      expect(error.code).toBe(HakanaiErrorCodes.EXPECTED_UINT8_ARRAY);
      expect(error.message).toBe("Input must be a Uint8Array");
    }
  });

  test("MISSING vs INVALID error differentiation", async () => {
    const { InputValidation } = require("../server/src/typescript/hakanai-client") as any;
    const client = new HakanaiClient("http://localhost:8080");

    // Test auth token: empty should be valid (no error thrown)
    expect(() => {
      InputValidation.validateAuthToken("");
    }).not.toThrow();

    // Test auth token: invalid format should throw error
    try {
      InputValidation.validateAuthToken("invalid-format");
      fail("Expected HakanaiError to be thrown");
    } catch (error: any) {
      expect(error.code).toBe(HakanaiErrorCodes.INVALID_AUTH_TOKEN);
      expect(error.message).toBe("Auth token must be a 43-character base64url string (server-generated format)");
    }

    // Test secret ID: missing vs invalid
    try {
      InputValidation.validateSecretId("");
      fail("Expected HakanaiError to be thrown");
    } catch (error: any) {
      expect(error.code).toBe(HakanaiErrorCodes.MISSING_SECRET_ID);
      expect(error.message).toBe("Secret ID cannot be empty");
    }

    try {
      InputValidation.validateSecretId("not-a-uuid");
      fail("Expected HakanaiError to be thrown");
    } catch (error: any) {
      expect(error.code).toBe(HakanaiErrorCodes.INVALID_SECRET_ID);
      expect(error.message).toBe("Secret ID must be a valid UUID");
    }

    // Test secret key: missing vs invalid
    try {
      InputValidation.validateSecretKey("");
      fail("Expected HakanaiError to be thrown");
    } catch (error: any) {
      expect(error.code).toBe(HakanaiErrorCodes.MISSING_KEY);
      expect(error.message).toBe("Secret key cannot be empty");
    }

    try {
      InputValidation.validateSecretKey("invalid-key-format");
      fail("Expected HakanaiError to be thrown");
    } catch (error: any) {
      expect(error.code).toBe(HakanaiErrorCodes.INVALID_KEY);
      expect(error.message).toBe("Secret key must be a 43-character base64url string (32 bytes)");
    }

    // Test URL parsing via receivePayload: missing secret ID vs invalid format
    try {
      await client.receivePayload("http://localhost/s/");
      fail("Expected HakanaiError to be thrown");
    } catch (error: any) {
      expect(error.code).toBe(HakanaiErrorCodes.MISSING_SECRET_ID);
      expect(error.message).toBe("URL must contain secret ID in format /s/{id}");
    }
  });
});

describe("Browser Compatibility", () => {
  test("Web Crypto API is available", () => {
    expect(global.crypto).toBeDefined();
    expect(global.crypto.subtle).toBeDefined();
    expect(global.crypto.getRandomValues).toBeDefined();

    // Also verify that the functions work
    expect(typeof global.crypto.getRandomValues).toBe("function");
    expect(typeof global.crypto.subtle.encrypt).toBe("function");
    expect(typeof global.crypto.subtle.decrypt).toBe("function");
  });

  test("DOM APIs are available", () => {
    expect(global.URL).toBeDefined();
    expect(global.TextEncoder).toBeDefined();
    expect(global.TextDecoder).toBeDefined();
  });

  test("Base64 operations work with real browser APIs", () => {
    // Test that btoa/atob work with the DOM environment
    const original = "Hello, World!";
    const encoded = btoa(original);
    const decoded = atob(encoded);

    expect(decoded).toBe(original);
  });
});
