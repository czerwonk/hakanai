/**
 * Jest setup file for TypeScript DOM testing
 * Sets up Web Crypto API and browser globals
 */

// Setup Web Crypto API with real implementation
const { Crypto } = require("@peculiar/webcrypto");
const { TextEncoder, TextDecoder } = require("util");

// Create crypto instance
const cryptoInstance = new Crypto();

// Setup globals for browser environment
global.crypto = cryptoInstance;
global.TextEncoder = TextEncoder;
global.TextDecoder = TextDecoder;

// Also set on window for jsdom
if (typeof window !== "undefined") {
  window.crypto = cryptoInstance;
  window.TextEncoder = TextEncoder;
  window.TextDecoder = TextDecoder;
}

// Force setup of crypto.subtle on both global and window
// This works around jsdom potentially stripping the property
Object.defineProperty(global.crypto, "subtle", {
  value: cryptoInstance.subtle,
  writable: false,
  configurable: false,
});

if (typeof window !== "undefined") {
  Object.defineProperty(window.crypto, "subtle", {
    value: cryptoInstance.subtle,
    writable: false,
    configurable: false,
  });
}

// Ensure URL is available (jsdom should provide this)
if (!global.URL) {
  global.URL = require("url").URL;
}

// Setup fetch mock for controlled testing
global.fetch = jest.fn();

// Setup base64 operations
global.btoa =
  global.btoa || ((str) => Buffer.from(str, "binary").toString("base64"));
global.atob =
  global.atob || ((str) => Buffer.from(str, "base64").toString("binary"));

