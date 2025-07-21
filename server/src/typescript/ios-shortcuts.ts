/**
 * iOS Shortcuts API for Hakanai
 *
 * This module provides a simplified API for iOS Shortcuts integration,
 * exposing functions that can be called directly from iOS Shortcuts actions.
 */

import { HakanaiClient } from "./hakanai-client.js";

/**
 * Response format for iOS Shortcuts
 */
interface ShortcutResponse {
  success: boolean;
  url?: string;
  error?: string;
  errorCode?: string;
}

/**
 * Send a file as a secret via Hakanai
 *
 * @param fileData - Base64-encoded file data
 * @param filename - Name of the file (optional)
 * @param serverUrl - Hakanai server URL
 * @param authToken - Authentication token (optional)
 * @returns Promise resolving to response with URL or error
 */
async function sendSecret(
  fileData: string,
  filename: string | undefined,
  serverUrl: string,
  authToken?: string,
): Promise<ShortcutResponse> {
  try {
    const client = new HakanaiClient(serverUrl);
    const payload = client.createPayload(filename);
    payload.setFromBase64?.(fileData);

    const url = await client.sendPayload(payload, 86400, authToken); // 24 hours default

    return {
      success: true,
      url: url,
    };
  } catch (error: any) {
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
async function sendText(
  text: string,
  serverUrl: string,
  authToken?: string,
): Promise<ShortcutResponse> {
  try {
    const bytes = new TextEncoder().encode(text);
    const client = new HakanaiClient(serverUrl);
    const payload = client.createPayload();
    payload.setFromBytes?.(bytes);

    const url = await client.sendPayload(payload, 86400, authToken);

    return {
      success: true,
      url: url,
    };
  } catch (error: any) {
    return {
      success: false,
      error: error.message,
      errorCode: error.code || "UNKNOWN_ERROR",
    };
  }
}

// Export for iOS Shortcuts (global scope)
declare const globalThis: any;
globalThis.HakanaiShortcuts = {
  sendSecret,
  sendText,
};

// Also set on window for browser compatibility
if (typeof window !== "undefined") {
  (window as any).HakanaiShortcuts = {
    sendSecret,
    sendText,
  };
}

// Signal that the API is ready
if (typeof window !== "undefined") {
  (window as any).hakanaiReady = true;
} else if (typeof globalThis !== "undefined") {
  globalThis.hakanaiReady = true;
}

// Export for module systems
export { sendSecret, sendText, type ShortcutResponse };
