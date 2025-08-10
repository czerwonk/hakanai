/**
 * Jest setup file for TypeScript DOM testing
 * Sets up Web Crypto API and browser globals
 */

// CRITICAL: Fix Date.now BEFORE any other imports to prevent jsdom issues
const OriginalDate = Date;
global.Date = OriginalDate;
global.Date.now =
  OriginalDate.now ||
  function () {
    return new OriginalDate().getTime();
  };

// Also monkey-patch the Date object to ensure it always has .now
if (!Date.now) {
  Date.now = function () {
    return new Date().getTime();
  };
}

// Setup Web Crypto API with real implementation
const { Crypto } = require("@peculiar/webcrypto");
const { TextEncoder, TextDecoder } = require("util");

// Setup ReadableStream polyfill for Node.js tests
if (!global.ReadableStream) {
  const {
    ReadableStream,
    WritableStream,
    TransformStream,
  } = require("web-streams-polyfill");
  global.ReadableStream = ReadableStream;
  global.WritableStream = WritableStream;
  global.TransformStream = TransformStream;

  if (typeof window !== "undefined") {
    window.ReadableStream = ReadableStream;
    window.WritableStream = WritableStream;
    window.TransformStream = TransformStream;
  }
}

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

// Additional Date.now setup for window context
if (typeof window !== "undefined") {
  window.Date = OriginalDate;
  window.Date.now =
    OriginalDate.now ||
    function () {
      return new OriginalDate().getTime();
    };
}

// Also set it directly on global for early access
global.now = global.Date.now;

// Setup mock location to avoid redefinition issues
global.locationMock = {
  origin: "https://example.com",
  protocol: "https:",
  href: "https://example.com/get",
  pathname: "/get",
};

// Setup i18n mock
const mockI18n = {
  t: (key) => key, // Simple key->key mapping for robustness
  setLanguage: () => {},
  getCurrentLanguage: () => "en",
};

global.i18n = mockI18n;
if (typeof window !== "undefined") {
  window.i18n = mockI18n;
}
