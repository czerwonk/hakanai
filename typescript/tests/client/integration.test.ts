// SPDX-License-Identifier: Apache-2.0

/**
 * Integration tests for HakanaiClient
 * Tests real crypto operations with minimal mocking
 */

import { HakanaiClient, HakanaiErrorCodes, Base64UrlSafe } from "../../src/hakanai-client";

// Helper function to ensure we get proper Uint8Array in tests
function encodeText(text: string): Uint8Array {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(text);
  return new Uint8Array(encoded);
}

// Helper function to generate valid UUID v4
function generateUUID(): string {
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, function (c) {
    const r = (Math.random() * 16) | 0;
    const v = c == "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
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

// Mock server responses for both XHR and fetch
const createMockServices = () => {
  const mockSecrets = new Map<string, string>();

  // Create XHR mock for POST requests
  const mockXHRInstance = {
    open: jest.fn(),
    send: jest.fn((body: string) => {
      setTimeout(() => {
        try {
          const parsedBody = JSON.parse(body);
          const secretId = generateUUID();
          mockSecrets.set(secretId, parsedBody.data);
          mockXHRInstance.responseText = JSON.stringify({ id: secretId });
          mockXHRInstance.onload?.();
        } catch (error) {
          mockXHRInstance.onerror?.();
        }
      }, 0);
    }),
    setRequestHeader: jest.fn(),
    upload: {},
    status: 200,
    statusText: "OK",
    responseText: "",
    onload: null as any,
    onerror: null as any,
  };

  (global as any).XMLHttpRequest = jest.fn(() => mockXHRInstance);

  // Create fetch mock for GET requests
  const mockFetch = jest.fn(async (url: string, options?: any) => {
    const urlObj = new URL(url);

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
        headers: {
          get: (name: string) => (name === "content-length" ? encryptedData.length.toString() : null),
        },
        body: new ReadableStream({
          start(controller) {
            const bytes = new TextEncoder().encode(encryptedData);
            controller.enqueue(bytes);
            controller.close();
          },
        }),
        text: () => Promise.resolve(encryptedData),
      });
    }

    return Promise.resolve({
      ok: false,
      status: 404,
      statusText: "Not Found",
    });
  });

  return { mockFetch };
};

describe("HakanaiClient Integration", () => {
  let client: HakanaiClient;
  let originalFetch: any;

  beforeEach(() => {
    originalFetch = global.fetch;
    const { mockFetch } = createMockServices();
    global.fetch = mockFetch as any;
    client = new HakanaiClient("http://localhost:8080");
  });

  afterEach(() => {
    global.fetch = originalFetch;
    jest.restoreAllMocks();
  });

  test("complete roundtrip: send and receive text secret", async () => {
    const originalText = "This is a secret message that should roundtrip correctly! 🔐";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload();
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

    // Send the secret
    const secretUrl = await client.sendPayload(originalPayload, 3600);

    expect(secretUrl).toMatch(/^http:\/\/localhost:8080\/s\/[0-9a-f-]+#[A-Za-z0-9_-]+:[A-Za-z0-9_-]{22}$/i);

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
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

    // Send the secret
    const secretUrl = await client.sendPayload(originalPayload, 1800);

    expect(secretUrl).toMatch(/^http:\/\/localhost:8080\/s\/[0-9a-f-]+#[A-Za-z0-9_-]+:[A-Za-z0-9_-]{22}$/i);

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
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

    const secretUrl = await client.sendPayload(originalPayload);
    const retrievedPayload = await client.receivePayload(secretUrl);

    expect(retrievedPayload.decode!()).toBe(originalText);
    expect(retrievedPayload.filename).toBe(filename);
  });

  test("roundtrip with empty filename (null handling)", async () => {
    const originalText = "Secret without filename";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload();
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

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
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

    const secretUrl = await client.sendPayload(originalPayload);
    const retrievedPayload = await client.receivePayload(secretUrl);

    const decodedData = retrievedPayload.decode!();
    expect(decodedData).toBe(largeData);
    expect(decodedData).toHaveLength(10011);
    expect(decodedData.endsWith(" end marker")).toBe(true);
    expect(retrievedPayload.filename).toBe(filename);
  });

  test("receive one-time token", async () => {
    const originalText = "Binary file content or any text treated as file";
    const filename = "test-document.txt";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload(filename);
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

    // Send the secret
    const secretUrl = await client.sendPayload(originalPayload, 1800);

    expect(secretUrl).toMatch(/^http:\/\/localhost:8080\/s\/[0-9a-f-]+#[A-Za-z0-9_-]+:[A-Za-z0-9_-]{22}$/i);

    // Receive the secret
    const retrievedPayload = await client.receivePayload(secretUrl);

    expect(retrievedPayload.decode!()).toBe(originalText);
    expect(retrievedPayload.filename).toBe(filename);
  });

  test("payload data is base64-encoded in internal format", async () => {
    const originalText = "test message";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload();
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

    // Mock XHR to capture what gets sent
    const sentData: any[] = [];
    const captureXHRInstance = {
      open: jest.fn(),
      send: jest.fn((body: string) => {
        try {
          const parsedBody = JSON.parse(body);
          sentData.push(parsedBody);
          captureXHRInstance.responseText = JSON.stringify({ id: "test-123" });
          setTimeout(() => captureXHRInstance.onload?.(), 0);
        } catch (error) {
          setTimeout(() => captureXHRInstance.onerror?.(), 0);
        }
      }),
      setRequestHeader: jest.fn(),
      upload: {},
      status: 200,
      statusText: "OK",
      responseText: '{"id": "test-123"}',
      onload: null as any,
      onerror: null as any,
    };
    (global as any).XMLHttpRequest = jest.fn(() => captureXHRInstance);

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
    payload.setFromBytes(textBytes.buffer as ArrayBuffer);

    const url = await client.sendPayload(payload);

    const urlObj = new URL(url);
    expect(urlObj.protocol).toBe("http:");
    expect(urlObj.hostname).toBe("localhost");
    expect(urlObj.port).toBe("8080");
    expect(urlObj.pathname).toMatch(/^\/s\/[0-9a-f-]+$/i);
    expect(urlObj.hash).toMatch(/^#[A-Za-z0-9_-]+:[A-Za-z0-9_-]{22}$/);

    // Parse key and hash from fragment
    const fragmentParts = urlObj.hash.slice(1).split(":");
    expect(fragmentParts).toHaveLength(2);

    const keyBase64 = fragmentParts[0];
    const hash = fragmentParts[1];

    // Key should be exactly 32 bytes when decoded
    const keyBytes = Base64UrlSafe.decode(keyBase64);
    expect(keyBytes).toHaveLength(32);

    // Hash should be exactly 22 base64url characters
    expect(hash).toMatch(/^[A-Za-z0-9_-]{22}$/);
    expect(hash).toHaveLength(22);
  });

  test("PayloadData decode() method works correctly", async () => {
    const originalText = "Test message with unicode: 🔐 åëïöü";
    const filename = "test.txt";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload(filename);
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

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
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

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

  test("hash mismatch validation fails", async () => {
    const originalText = "This is a test secret for hash validation";
    const textBytes = encodeText(originalText);

    const originalPayload = client.createPayload();
    originalPayload.setFromBytes(textBytes.buffer as ArrayBuffer);

    // Send the secret to get a valid URL
    const secretUrl = await client.sendPayload(originalPayload, 3600);

    // Parse the URL and tamper with the hash
    const urlObj = new URL(secretUrl);
    const fragmentParts = urlObj.hash.slice(1).split(":");
    const key = fragmentParts[0];
    const originalHash = fragmentParts[1];

    // Create a tampered hash (flip the last character)
    const tamperedHash = originalHash.slice(0, -1) + (originalHash.slice(-1) === "a" ? "b" : "a");
    const tamperedUrl = `${urlObj.origin}${urlObj.pathname}#${key}:${tamperedHash}`;

    // Attempt to retrieve with tampered hash should fail
    await expect(client.receivePayload(tamperedUrl)).rejects.toThrow();

    try {
      await client.receivePayload(tamperedUrl);
    } catch (error: any) {
      expect(error.code).toBe(HakanaiErrorCodes.HASH_MISMATCH);
      expect(error.message).toContain("Hash verification failed");
    }
  });
});
