/**
 * Hakanai JavaScript Client (TypeScript Implementation)
 *
 * This client implements the same cryptographic protocol as the Rust CLI client,
 * allowing you to send and receive encrypted secrets via the Hakanai API.
 */
/**
 * Type-safe Base64 URL-Safe encoding utility
 * Uses modern browser APIs for better performance and reliability
 */
class Base64UrlSafe {
    /**
     * Encode Uint8Array to URL-safe base64 string
     * Uses chunked processing to handle large arrays safely
     */
    static encode(data) {
        if (!(data instanceof Uint8Array)) {
            throw new Error("Input must be a Uint8Array");
        }
        // Handle empty arrays
        if (data.length === 0) {
            return "";
        }
        // Convert Uint8Array to binary string safely, handling large arrays
        let binaryString = "";
        const chunkSize = 8192; // Process in chunks to avoid call stack limits
        for (let i = 0; i < data.length; i += chunkSize) {
            const chunk = data.subarray(i, i + chunkSize);
            binaryString += String.fromCharCode(...chunk);
        }
        // Encode to standard base64 then convert to URL-safe
        return btoa(binaryString)
            .replace(/\+/g, "-")
            .replace(/\//g, "_")
            .replace(/=/g, "");
    }
    /**
     * Decode URL-safe base64 string to Uint8Array
     * More robust error handling and validation
     */
    static decode(encoded) {
        if (typeof encoded !== "string") {
            throw new Error("Input must be a string");
        }
        if (encoded.length === 0) {
            return new Uint8Array(0);
        }
        if (!/^[A-Za-z0-9_-]*$/.test(encoded)) {
            throw new Error("Invalid base64url characters");
        }
        // Add proper padding
        const paddingLength = (4 - (encoded.length % 4)) % 4;
        const padded = encoded + "=".repeat(paddingLength);
        // Convert back to standard base64
        const standard = padded.replace(/-/g, "+").replace(/_/g, "/");
        try {
            const binaryString = atob(standard);
            const bytes = new Uint8Array(binaryString.length);
            // Convert binary string to Uint8Array more efficiently
            for (let i = 0; i < binaryString.length; i++) {
                bytes[i] = binaryString.charCodeAt(i);
            }
            return bytes;
        }
        catch (error) {
            throw new Error("Failed to decode base64url string: invalid encoding");
        }
    }
    /**
     * Utility method for encoding text strings directly
     * More efficient than manual TextEncoder + encode
     */
    static encodeText(text) {
        if (typeof text !== "string") {
            throw new Error("Input must be a string");
        }
        const encoder = new TextEncoder();
        const data = encoder.encode(text);
        return Base64UrlSafe.encode(data);
    }
    /**
     * Utility method for decoding to text strings directly
     * More efficient than decode + manual TextDecoder
     */
    static decodeText(encoded) {
        const data = Base64UrlSafe.decode(encoded);
        const decoder = new TextDecoder();
        return decoder.decode(data);
    }
}
/**
 * Browser compatibility checker
 */
class BrowserCompatibility {
    /**
     * Get detailed compatibility information
     */
    static getCompatibilityInfo() {
        const missingFeatures = [];
        // Check for Web Crypto API
        if (!window.crypto || !window.crypto.subtle) {
            missingFeatures.push("Web Crypto API (crypto.subtle)");
        }
        // Check for TextEncoder/TextDecoder
        if (typeof TextEncoder === "undefined") {
            missingFeatures.push("TextEncoder");
        }
        if (typeof TextDecoder === "undefined") {
            missingFeatures.push("TextDecoder");
        }
        // Check for crypto.getRandomValues
        if (!window.crypto || typeof window.crypto.getRandomValues !== "function") {
            missingFeatures.push("crypto.getRandomValues");
        }
        // Check for fetch API
        if (typeof fetch === "undefined") {
            missingFeatures.push("Fetch API");
        }
        // Check for Uint8Array
        if (typeof Uint8Array === "undefined") {
            missingFeatures.push("Uint8Array");
        }
        // Check for atob/btoa (base64 functions)
        if (typeof atob === "undefined" || typeof btoa === "undefined") {
            missingFeatures.push("Base64 functions (atob/btoa)");
        }
        return {
            isCompatible: missingFeatures.length === 0,
            missingFeatures: Object.freeze(missingFeatures),
            supportedBrowsers: "Chrome 60+, Firefox 57+, Safari 11+, Edge 79+",
        };
    }
    /**
     * Check if browser is compatible (simple boolean)
     */
    static isCompatible() {
        return BrowserCompatibility.getCompatibilityInfo().isCompatible;
    }
}
/**
 * Type-safe crypto operations
 */
class CryptoOperations {
    /**
     * Generate a random 256-bit AES key
     */
    static generateKey() {
        const bytes = new Uint8Array(32);
        crypto.getRandomValues(bytes);
        return Object.freeze({ bytes, length: 32 });
    }
    /**
     * Import a raw key for use with WebCrypto API
     */
    static async importKey(rawKey) {
        if (!(rawKey instanceof Uint8Array)) {
            throw new Error("Key must be a Uint8Array");
        }
        if (rawKey.length !== 32) {
            throw new Error("Key must be exactly 32 bytes for AES-256");
        }
        return crypto.subtle.importKey("raw", rawKey, { name: "AES-GCM", length: 256 }, false, ["encrypt", "decrypt"]);
    }
    /**
     * Encrypt a message with AES-GCM
     */
    static async encrypt(plaintext, key) {
        if (typeof plaintext !== "string") {
            throw new Error("Plaintext must be a string");
        }
        const encoder = new TextEncoder();
        const plaintextBytes = encoder.encode(plaintext);
        // Generate random nonce
        const nonce = new Uint8Array(12);
        crypto.getRandomValues(nonce);
        const cryptoKey = await CryptoOperations.importKey(key.bytes);
        const ciphertext = await crypto.subtle.encrypt({ name: "AES-GCM", iv: nonce }, cryptoKey, plaintextBytes);
        // Combine nonce and ciphertext
        const combined = new Uint8Array(nonce.length + ciphertext.byteLength);
        combined.set(nonce);
        combined.set(new Uint8Array(ciphertext), nonce.length);
        // Encode to standard base64 using chunked approach
        let binaryString = "";
        const chunkSize = 8192;
        for (let i = 0; i < combined.length; i += chunkSize) {
            const chunk = combined.subarray(i, i + chunkSize);
            binaryString += String.fromCharCode(...chunk);
        }
        return btoa(binaryString);
    }
    /**
     * Decrypt an encrypted message
     */
    static async decrypt(encryptedData, key) {
        if (typeof encryptedData !== "string") {
            throw new Error("Encrypted data must be a string");
        }
        if (!(key instanceof Uint8Array) || key.length !== 32) {
            throw new Error("Key must be a 32-byte Uint8Array");
        }
        // Decode from standard base64 more efficiently
        const binaryString = atob(encryptedData);
        const combined = new Uint8Array(binaryString.length);
        for (let i = 0; i < binaryString.length; i++) {
            combined[i] = binaryString.charCodeAt(i);
        }
        if (combined.length < 12) {
            throw new Error("Invalid encrypted data: too short");
        }
        // Extract nonce and ciphertext
        const nonce = combined.slice(0, 12);
        const ciphertext = combined.slice(12);
        const cryptoKey = await CryptoOperations.importKey(key);
        try {
            const plaintextBytes = await crypto.subtle.decrypt({ name: "AES-GCM", iv: nonce }, cryptoKey, ciphertext);
            const decoder = new TextDecoder();
            return decoder.decode(plaintextBytes);
        }
        catch (error) {
            throw new Error("Decryption failed: invalid key or corrupted data");
        }
    }
}
/**
 * Main Hakanai client class
 */
class HakanaiClient {
    constructor(baseUrl) {
        if (typeof baseUrl !== "string" || !baseUrl.trim()) {
            throw new Error("Base URL must be a non-empty string");
        }
        this.baseUrl = baseUrl.replace(/\/$/, ""); // Remove trailing slash
        // Check browser compatibility
        const compatibilityInfo = BrowserCompatibility.getCompatibilityInfo();
        if (!compatibilityInfo.isCompatible) {
            throw new Error(`Your browser does not support the following required features: ${compatibilityInfo.missingFeatures.join(", ")}. ` +
                `Please use a modern browser (${compatibilityInfo.supportedBrowsers}) that supports these Web APIs.`);
        }
    }
    /**
     * Static method to check browser compatibility without creating an instance
     */
    static isCompatible() {
        return BrowserCompatibility.isCompatible();
    }
    /**
     * Static method to get compatibility information
     */
    static getCompatibilityInfo() {
        return BrowserCompatibility.getCompatibilityInfo();
    }
    /**
     * Send a payload to the server
     */
    async sendPayload(payload, ttl = 3600, authToken) {
        if (!payload || typeof payload !== "object") {
            throw new Error("Payload must be an object");
        }
        if (!payload.data ||
            typeof payload.data !== "string" ||
            payload.data.trim().length === 0) {
            throw new Error("Payload data cannot be empty");
        }
        if (typeof ttl !== "number" || ttl <= 0 || !Number.isInteger(ttl)) {
            throw new Error("TTL must be a positive integer");
        }
        if (authToken !== undefined && typeof authToken !== "string") {
            throw new Error("Auth token must be a string if provided");
        }
        const key = CryptoOperations.generateKey();
        const payloadJson = JSON.stringify(payload);
        const encryptedData = await CryptoOperations.encrypt(payloadJson, key);
        const headers = {
            "Content-Type": "application/json",
        };
        if (authToken && authToken.length > 0) {
            headers["Authorization"] = `Bearer ${authToken}`;
        }
        const requestBody = {
            data: encryptedData,
            expires_in: ttl,
        };
        const response = await fetch(`${this.baseUrl}/api/v1/secret`, {
            method: "POST",
            headers: headers,
            body: JSON.stringify(requestBody),
        });
        if (!response.ok) {
            throw new Error(`Failed to send secret: ${response.status} ${response.statusText}`);
        }
        const result = await response.json();
        if (!result.id || typeof result.id !== "string") {
            throw new Error("Invalid response: missing secret ID");
        }
        const secretId = result.id;
        const keyBase64 = Base64UrlSafe.encode(key.bytes);
        return `${this.baseUrl}/s/${secretId}#${keyBase64}`;
    }
    /**
     * Receive a payload from the server
     */
    async receivePayload(url) {
        if (typeof url !== "string" || !url.trim()) {
            throw new Error("URL must be a non-empty string");
        }
        // Parse the URL
        let urlObj;
        try {
            urlObj = new URL(url);
        }
        catch (error) {
            throw new Error("Invalid URL format");
        }
        const secretId = urlObj.pathname.split("/").pop();
        if (!secretId) {
            throw new Error("No secret ID found in URL");
        }
        const keyBase64 = urlObj.hash.slice(1); // Remove the #
        if (!keyBase64) {
            throw new Error("No decryption key found in URL");
        }
        let key;
        try {
            key = Base64UrlSafe.decode(keyBase64);
        }
        catch (error) {
            throw new Error("Invalid decryption key in URL");
        }
        if (key.length !== 32) {
            throw new Error("Invalid key length");
        }
        const response = await fetch(`${this.baseUrl}/api/v1/secret/${secretId}`);
        if (!response.ok) {
            if (response.status === 404) {
                throw new Error("Secret not found or has expired");
            }
            throw new Error(`Failed to retrieve secret: ${response.status} ${response.statusText}`);
        }
        const encryptedData = await response.text();
        if (!encryptedData) {
            throw new Error("Empty response from server");
        }
        const decryptedJson = await CryptoOperations.decrypt(encryptedData, key);
        let payload;
        try {
            payload = JSON.parse(decryptedJson);
        }
        catch (error) {
            throw new Error("Failed to parse decrypted payload");
        }
        // Validate payload structure
        if (!payload ||
            typeof payload !== "object" ||
            typeof payload.data !== "string") {
            throw new Error("Invalid payload structure");
        }
        return payload;
    }
    /**
     * Legacy methods for backward compatibility
     */
    /**
     * @deprecated Use CryptoOperations.generateKey() instead
     */
    async generateKey() {
        return CryptoOperations.generateKey().bytes;
    }
    /**
     * @deprecated Use CryptoOperations.importKey() instead
     */
    async importKey(rawKey) {
        return CryptoOperations.importKey(rawKey);
    }
    /**
     * @deprecated Use CryptoOperations.encrypt() instead
     */
    async encrypt(plaintext, key) {
        return CryptoOperations.encrypt(plaintext, { bytes: key, length: 32 });
    }
    /**
     * @deprecated Use CryptoOperations.decrypt() instead
     */
    async decrypt(encryptedData, key) {
        return CryptoOperations.decrypt(encryptedData, key);
    }
}
// Export for use in browsers (global namespace)
if (typeof window !== "undefined") {
    window.HakanaiClient = HakanaiClient;
    window.Base64UrlSafe = Base64UrlSafe;
    window.CryptoOperations = CryptoOperations;
}
if (typeof module !== "undefined" && module.exports) {
    module.exports = { HakanaiClient, Base64UrlSafe, CryptoOperations };
}
export { HakanaiClient, Base64UrlSafe, CryptoOperations, };
