// SPDX-License-Identifier: Apache-2.0

/**
 * Error handling and validation tests for HakanaiClient
 */

import { HakanaiClient, HakanaiErrorCodes, Base64UrlSafe, ContentAnalysis } from "../../src/hakanai-client";

// Helper function to ensure we get proper Uint8Array in tests
function encodeText(text: string): Uint8Array {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(text);
  return new Uint8Array(encoded);
}

// Mock Response and Headers for XHR-based client
if (typeof global.Response === "undefined") {
  (global as any).Response = class MockResponse {
    private _text: string;
    public status: number;
    public statusText: string;
    public headers: any;
    public ok: boolean;

    constructor(body: string, init?: { status?: number; statusText?: string; headers?: any }) {
      this._text = body;
      this.status = init?.status || 200;
      this.statusText = init?.statusText || "OK";
      this.headers = init?.headers || new Map();
      this.ok = this.status >= 200 && this.status < 300;
    }

    async json() {
      return JSON.parse(this._text);
    }

    async text() {
      return this._text;
    }
  };
}

if (typeof global.Headers === "undefined") {
  (global as any).Headers = class MockHeaders extends Map {
    constructor() {
      super();
    }
  };
}

// Mock XMLHttpRequest for XHR-based client
const createMockXHR = (
  config: {
    status?: number;
    statusText?: string;
    responseText?: string;
    shouldError?: boolean;
  } = {},
) => {
  const mockXHRInstance = {
    open: jest.fn(),
    send: jest.fn(),
    setRequestHeader: jest.fn(),
    upload: {},
    status: config.status || 200,
    statusText: config.statusText || "OK",
    responseText: config.responseText || '{"id": "test-uuid"}',
    onload: null as any,
    onerror: null as any,
  };

  // Configure send behavior
  mockXHRInstance.send.mockImplementation(() => {
    // Simulate async behavior
    setTimeout(() => {
      if (config.shouldError) {
        mockXHRInstance.onerror?.();
      } else {
        mockXHRInstance.onload?.();
      }
    }, 0);
  });

  (global as any).XMLHttpRequest = jest.fn(() => mockXHRInstance);
  return mockXHRInstance;
};

describe("Error Handling", () => {
  let client: HakanaiClient;
  let mockXHRInstance: any;

  beforeEach(() => {
    mockXHRInstance = createMockXHR();
    client = new HakanaiClient("http://localhost:8080");
  });

  test("sendPayload validates input", async () => {
    await expect(client.sendPayload({} as any)).rejects.toThrow("Payload data cannot be empty");

    const emptyPayload = client.createPayload();
    // Don't call setFromBytes, so data remains empty
    await expect(client.sendPayload(emptyPayload)).rejects.toThrow("Payload data cannot be empty");

    const testBytes = encodeText("test");
    const validPayload = client.createPayload();
    validPayload.setFromBytes(testBytes.buffer as ArrayBuffer);

    await expect(client.sendPayload(validPayload, 0)).rejects.toThrow("TTL must be a positive number");

    await expect(client.sendPayload(validPayload, 3600, 123 as any)).rejects.toThrow("Auth token must be a string");

    // Test restrictions validation
    await expect(client.sendPayload(validPayload, 3600, undefined, undefined, 123 as any)).rejects.toThrow(
      "Restrictions must be an object",
    );

    await expect(
      client.sendPayload(validPayload, 3600, undefined, undefined, {
        allowed_ips: "not-array",
      } as any),
    ).rejects.toThrow("allowed_ips must be an array");

    await expect(
      client.sendPayload(validPayload, 3600, undefined, undefined, {
        allowed_ips: ["invalid-ip"],
      }),
    ).rejects.toThrow("Invalid IP address or CIDR notation");
  });

  test("sendPayload throws HakanaiError (network error)", async () => {
    mockXHRInstance = createMockXHR({ shouldError: true });
    client = new HakanaiClient("http://localhost:8080");

    const testBytes = encodeText("test secret");
    const payload = client.createPayload();
    payload.setFromBytes(testBytes.buffer as ArrayBuffer);

    await expect(client.sendPayload(payload, 3600)).rejects.toThrow();
  });

  test("sendPayload throws HakanaiError (403)", async () => {
    mockXHRInstance = createMockXHR({
      status: 403,
      statusText: "Forbidden",
    });
    client = new HakanaiClient("http://localhost:8080");

    const testBytes = encodeText("test secret");
    const payload = client.createPayload();
    payload.setFromBytes(testBytes.buffer as ArrayBuffer);

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

  test("sendPayload throws HakanaiError (413)", async () => {
    mockXHRInstance = createMockXHR({
      status: 413,
      statusText: "Payload Too Large",
    });
    client = new HakanaiClient("http://localhost:8080");

    const testBytes = encodeText("test secret");
    const payload = client.createPayload();
    payload.setFromBytes(testBytes.buffer as ArrayBuffer);

    try {
      await client.sendPayload(payload, 3600);
      fail("Expected error to be thrown");
    } catch (error: any) {
      expect(error.name).toBe("HakanaiError");
      expect(error.code).toBe(HakanaiErrorCodes.PAYLOAD_TOO_LARGE);
      expect(error.statusCode).toBe(413);
    }
  });

  test("sendPayload throws HakanaiError (501)", async () => {
    mockXHRInstance = createMockXHR({
      status: 501,
      statusText: "Not Implemented",
    });
    client = new HakanaiClient("http://localhost:8080");

    const testBytes = encodeText("test secret");
    const payload = client.createPayload();
    payload.setFromBytes(testBytes.buffer as ArrayBuffer);

    try {
      await client.sendPayload(payload, 3600);
      fail("Expected error to be thrown");
    } catch (error: any) {
      expect(error.name).toBe("HakanaiError");
      expect(error.code).toBe(HakanaiErrorCodes.NOT_SUPPORTED);
      expect(error.statusCode).toBe(501);
      expect(error.message).toBe("This feature or operation is not supported by the server");
    }
  });

  test("receivePayload validates URL", async () => {
    await expect(client.receivePayload("")).rejects.toThrow("URL cannot be empty");

    await expect(client.receivePayload("not-a-url")).rejects.toThrow("Invalid URL format");

    await expect(client.receivePayload("http://localhost/no-secret-id")).rejects.toThrow(
      "URL must contain secret ID in format /s/{id}",
    );

    await expect(client.receivePayload("http://localhost/s/550e8400-e29b-41d4-a716-446655440000")).rejects.toThrow(
      "URL must contain decryption key and hash in fragment",
    );
  });

  test("receivePayload handles server errors", async () => {
    // receivePayload still uses fetch for GET requests, so we mock fetch for this test
    global.fetch = jest.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: "Not Found",
    }) as any;

    const client = new HakanaiClient("http://localhost:8080");

    // Use a proper UUID and 32-byte base64 key for the test
    const validUuid = "550e8400-e29b-41d4-a716-446655440000";
    const validKey = Base64UrlSafe.encode(new Uint8Array(32)); // 32 zero bytes
    const validHash = "AAAAAAAAAAAAAAAAAAAAAA"; // 22-char base64url hash
    await expect(
      client.receivePayload("http://localhost:8080/s/" + validUuid + "#" + validKey + ":" + validHash),
    ).rejects.toThrow("Secret not found or has expired");
  });
});

describe("Error Code Constants", () => {
  test("Error codes are readonly constants", () => {
    const codes = Object.keys(HakanaiErrorCodes);
    expect(codes.length).toBe(33);

    expect(codes).toContain("AUTHENTICATION_REQUIRED");
    expect(codes).toContain("INVALID_TOKEN");
    expect(codes).toContain("SEND_FAILED");
    expect(codes).toContain("SECRET_NOT_FOUND");
    expect(codes).toContain("SECRET_ALREADY_ACCESSED");
    expect(codes).toContain("RETRIEVE_FAILED");
    expect(codes).toContain("MISSING_DECRYPTION_KEY");
    expect(codes).toContain("PAYLOAD_TOO_LARGE");
    expect(codes).toContain("NOT_SUPPORTED");
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
    expect(codes).toContain("INVALID_HASH");
    expect(codes).toContain("MISSING_HASH");
    expect(codes).toContain("HASH_MISMATCH");
    expect(codes).toContain("ACCESS_DENIED");
    expect(codes).toContain("PASSPHRASE_REQUIRED");
  });

  test("Error codes can be used for comparison", () => {
    // Test that we can compare against the constants
    function checkErrorCode(code: string): boolean {
      return code === HakanaiErrorCodes.AUTHENTICATION_REQUIRED;
    }

    expect(checkErrorCode(HakanaiErrorCodes.AUTHENTICATION_REQUIRED)).toBe(true);
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
    const { InputValidation } = require("../../src/hakanai-client") as any;
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
