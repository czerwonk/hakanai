// SPDX-License-Identifier: Apache-2.0

/**
 * Browser compatibility tests for HakanaiClient
 * Tests availability of required browser APIs
 */

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
});

