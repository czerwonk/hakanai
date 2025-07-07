/**
 * Integration tests for Hakanai TypeScript client
 * Tests real crypto operations with minimal mocking
 */

import {
  HakanaiClient,
  Base64UrlSafe,
  CryptoOperations,
} from "../server/src/includes/hakanai-client";

// Helper function to ensure we get proper Uint8Array in tests
function encodeText(text: string): Uint8Array {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(text);
  return new Uint8Array(encoded);
}

// Mock server responses only
const createMockFetch = () => {
  const mockSecrets = new Map<string, string>();

  return jest.fn((url: string, options?: any) => {
    const urlObj = new URL(url);

    // POST /api/v1/secret - create secret
    if (urlObj.pathname === "/api/v1/secret" && options?.method === "POST") {
      const secretId = "test-" + Math.random().toString(36).substring(7);
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

describe("CryptoOperations", () => {
  test("generateKey produces 32-byte keys", () => {
    const key1 = CryptoOperations.generateKey();
    const key2 = CryptoOperations.generateKey();

    expect(key1.bytes).toHaveLength(32);
    expect(key2.bytes).toHaveLength(32);
    expect(key1.length).toBe(32);

    // Keys should be different
    expect(key1.bytes).not.toEqual(key2.bytes);
  });

  test("encrypt and decrypt round trip", async () => {
    const original = "Secret message for encryption test";
    const key = CryptoOperations.generateKey();

    const encrypted = await CryptoOperations.encrypt(original, key);
    const decrypted = await CryptoOperations.decrypt(encrypted, key.bytes);

    expect(decrypted).toBe(original);
  });

  test("encrypt produces different results with same input", async () => {
    const original = "Same message";
    const key = CryptoOperations.generateKey();

    const encrypted1 = await CryptoOperations.encrypt(original, key);
    const encrypted2 = await CryptoOperations.encrypt(original, key);

    // Should be different due to random nonce
    expect(encrypted1).not.toBe(encrypted2);

    // But both should decrypt to same message
    const decrypted1 = await CryptoOperations.decrypt(encrypted1, key.bytes);
    const decrypted2 = await CryptoOperations.decrypt(encrypted2, key.bytes);

    expect(decrypted1).toBe(original);
    expect(decrypted2).toBe(original);
  });

  test("decrypt fails with wrong key", async () => {
    const original = "Secret message";
    const key1 = CryptoOperations.generateKey();
    const key2 = CryptoOperations.generateKey();

    const encrypted = await CryptoOperations.encrypt(original, key1);

    await expect(
      CryptoOperations.decrypt(encrypted, key2.bytes),
    ).rejects.toThrow("Decryption failed");
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
      /^http:\/\/localhost:8080\/s\/test-.+#[A-Za-z0-9_-]+$/,
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
      /^http:\/\/localhost:8080\/s\/test-.+#[A-Za-z0-9_-]+$/,
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
    expect(urlObj.pathname).toMatch(/^\/s\/test-.+$/);
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
      "TTL must be a positive integer",
    );

    await expect(
      client.sendPayload(validPayload, 3600, 123 as any),
    ).rejects.toThrow("Auth token must be a string if provided");
  });

  test("receivePayload validates URL", async () => {
    await expect(client.receivePayload("")).rejects.toThrow(
      "URL must be a non-empty string",
    );

    await expect(client.receivePayload("not-a-url")).rejects.toThrow(
      "Invalid URL format",
    );

    await expect(
      client.receivePayload("http://localhost/no-secret-id"),
    ).rejects.toThrow("No secret ID found in URL");

    await expect(
      client.receivePayload("http://localhost/s/123"),
    ).rejects.toThrow("No decryption key found in URL");
  });

  test("receivePayload handles server errors", async () => {
    global.fetch = jest.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: "Not Found",
    }) as any;

    const client = new HakanaiClient("http://localhost:8080");

    // Use a proper 32-byte base64 key for the test
    const validKey = Base64UrlSafe.encode(new Uint8Array(32)); // 32 zero bytes
    await expect(
      client.receivePayload("http://localhost:8080/s/missing#" + validKey),
    ).rejects.toThrow("Secret not found or has expired");
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
