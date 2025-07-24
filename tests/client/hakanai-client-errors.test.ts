/**
 * Error handling and validation tests for HakanaiClient
 */

import {
  HakanaiClient,
  HakanaiErrorCodes,
  Base64UrlSafe,
  ContentAnalysis,
} from "../../server/src/typescript/hakanai-client";

// Helper function to ensure we get proper Uint8Array in tests
function encodeText(text: string): Uint8Array {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(text);
  return new Uint8Array(encoded);
}

// Mock server responses only
const createMockFetch = () => {
  return jest.fn((url: string, options?: any) => {
    return Promise.resolve({
      ok: false,
      status: 404,
      statusText: "Not Found",
    });
  });
};

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
      client.sendPayload(
        payload,
        3600,
        "oCTJV5YSQEllqpBQ5_4ttyeTJsQxNtgsz3xSGjqP9xw",
      ),
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
      client.sendPayload(
        payload,
        3600,
        "opBEGjLy_mkCsTbMog4nxnvstB39kNx8K7450KHHH4E",
      ),
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
      await client.sendPayload(
        payload,
        3600,
        "HUqlqUd68TmqGkNj5o7pMqRcJe2YIQqoOlMfSSYF5r8",
      );
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
      await client.sendPayload(
        payload,
        3600,
        "opBEGjLy_mkCsTbMog4nxnvstB39kNx8K7450KHHH4E",
      );
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
      client.receivePayload(
        "http://localhost/s/550e8400-e29b-41d4-a716-446655440000",
      ),
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
      client.receivePayload(
        "http://localhost:8080/s/" + validUuid + "#" + validKey,
      ),
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
    expect(codes.length).toBe(26);

    expect(codes).toContain("AUTHENTICATION_REQUIRED");
    expect(codes).toContain("INVALID_TOKEN");
    expect(codes).toContain("SEND_FAILED");
    expect(codes).toContain("SECRET_NOT_FOUND");
    expect(codes).toContain("SECRET_ALREADY_ACCESSED");
    expect(codes).toContain("RETRIEVE_FAILED");
    expect(codes).toContain("MISSING_DECRYPTION_KEY");
    expect(codes).toContain("PAYLOAD_TOO_LARGE");
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
    const { InputValidation } =
      require("../../server/src/typescript/hakanai-client") as any;
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
      expect(error.message).toBe(
        "Auth token must be a 43-character base64url string (server-generated format)",
      );
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
      expect(error.message).toBe(
        "Secret key must be a 43-character base64url string (32 bytes)",
      );
    }

    // Test URL parsing via receivePayload: missing secret ID vs invalid format
    try {
      await client.receivePayload("http://localhost/s/");
      fail("Expected HakanaiError to be thrown");
    } catch (error: any) {
      expect(error.code).toBe(HakanaiErrorCodes.MISSING_SECRET_ID);
      expect(error.message).toBe(
        "URL must contain secret ID in format /s/{id}",
      );
    }
  });
});

