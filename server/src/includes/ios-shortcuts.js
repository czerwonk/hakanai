/**
 * iOS Shortcuts API for Hakanai
 *
 * This module provides a simplified API for iOS Shortcuts integration,
 * exposing functions that can be called directly from iOS Shortcuts actions.
 */
import { HakanaiClient } from "./hakanai-client.js?v=1753089873";
/**
 * Send a file as a secret via Hakanai
 *
 * @param fileData - Base64-encoded file data
 * @param filename - Name of the file (optional)
 * @param serverUrl - Hakanai server URL
 * @param authToken - Authentication token (optional)
 * @returns Promise resolving to response with URL or error
 */
async function sendSecret(fileData, filename, serverUrl, authToken) {
    var _a, _b;
    try {
        const client = new HakanaiClient(serverUrl);
        const payload = client.createPayload(filename);
        (_a = payload.setFromBase64) === null || _a === void 0 ? void 0 : _a.call(payload, fileData);
        const url = await client.sendPayload(payload, 86400, authToken); // 24 hours default
        return {
            success: true,
            url: url,
            id: (_b = url.split('/').pop()) === null || _b === void 0 ? void 0 : _b.split('#')[0], // Extract ID from URL
        };
    }
    catch (error) {
        return {
            success: false,
            error: error.message,
            errorCode: error.code || "UNKNOWN_ERROR",
        };
    }
}
/**
 * Send text as a secret via Hakanai
 *
 * @param text - Plain text to send
 * @param serverUrl - Hakanai server URL
 * @param authToken - Authentication token (optional)
 * @returns Promise resolving to response with URL or error
 */
async function sendText(text, serverUrl, authToken) {
    var _a, _b;
    try {
        const bytes = new TextEncoder().encode(text);
        const client = new HakanaiClient(serverUrl);
        const payload = client.createPayload();
        (_a = payload.setFromBytes) === null || _a === void 0 ? void 0 : _a.call(payload, bytes);
        const url = await client.sendPayload(payload, 86400, authToken);
        return {
            success: true,
            url: url,
            id: (_b = url.split('/').pop()) === null || _b === void 0 ? void 0 : _b.split('#')[0], // Extract ID from URL
        };
    }
    catch (error) {
        return {
            success: false,
            error: error.message,
            errorCode: error.code || "UNKNOWN_ERROR",
        };
    }
}
globalThis.HakanaiShortcuts = {
    sendSecret,
    sendText,
};
// Also set on window for browser compatibility
if (typeof window !== "undefined") {
    window.HakanaiShortcuts = {
        sendSecret,
        sendText,
    };
}
// Signal that the API is ready
if (typeof window !== "undefined") {
    window.hakanaiReady = true;
}
else if (typeof globalThis !== "undefined") {
    globalThis.hakanaiReady = true;
}
// Export for module systems
export { sendSecret, sendText };
