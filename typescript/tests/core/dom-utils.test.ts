// SPDX-License-Identifier: Apache-2.0

import {
  createButton,
  createButtonContainer,
  announceToScreenReader,
  secureInputClear,
  generateRandomId,
} from "../../src/core/dom-utils";

describe("DOM Utilities", () => {
  beforeEach(() => {
    // Clear any existing timers
    jest.clearAllTimers();
  });

  describe("createButton", () => {
    test("should create button with correct properties", () => {
      const clickHandler = jest.fn();
      const button = createButton("test-class", "Test Button", "Test aria label", clickHandler);

      expect(button.tagName).toBe("BUTTON");
      expect(button.className).toBe("test-class");
      expect(button.textContent).toBe("Test Button");
      expect(button.getAttribute("aria-label")).toBe("Test aria label");

      button.click();
      expect(clickHandler).toHaveBeenCalled();
    });

    test("should create button without click handler", () => {
      const button = createButton("test-class", "Test Button", "Test aria label");

      expect(button.tagName).toBe("BUTTON");
      expect(button.className).toBe("test-class");
      expect(button.textContent).toBe("Test Button");
      expect(button.getAttribute("aria-label")).toBe("Test aria label");
    });
  });

  describe("createButtonContainer", () => {
    test("should create div with buttons-container class", () => {
      const container = createButtonContainer();

      expect(container.tagName).toBe("DIV");
      expect(container.className).toBe("buttons-container");
    });
  });

  describe("announceToScreenReader", () => {
    test("should create and remove announcement element", () => {
      jest.useFakeTimers();

      announceToScreenReader("Test announcement");

      // Check that element was created
      const announcement = document.querySelector('[aria-live="polite"]');
      expect(announcement).toBeTruthy();
      expect(announcement?.textContent).toBe("Test announcement");

      // Fast-forward time to trigger cleanup
      jest.advanceTimersByTime(1000);

      // Check that element was removed
      const removedAnnouncement = document.querySelector('[aria-live="polite"]');
      expect(removedAnnouncement).toBeFalsy();

      jest.useRealTimers();
    });
  });

  describe("secureInputClear", () => {
    test("should clear input value", () => {
      const input = document.createElement("input");
      input.value = "sensitive data";
      document.body.appendChild(input);

      secureInputClear(input);

      expect(input.value).toBe("");
    });

    test("should handle input not in DOM", () => {
      const input = document.createElement("input");
      input.value = "sensitive data";

      expect(() => secureInputClear(input)).not.toThrow();
      expect(input.value).toBe("");
    });
  });

  describe("generateRandomId", () => {
    test("should generate unique IDs", () => {
      const id1 = generateRandomId();
      const id2 = generateRandomId();

      expect(id1).not.toBe(id2);
      expect(typeof id1).toBe("string");
      expect(typeof id2).toBe("string");
      expect(id1.startsWith("url-")).toBe(true);
      expect(id2.startsWith("url-")).toBe(true);
    });

    test("should use crypto.randomUUID when available", () => {
      const mockRandomUUID = jest.fn(() => "mock-uuid-123");

      Object.defineProperty(global, "crypto", {
        value: {
          randomUUID: mockRandomUUID,
        },
        writable: true,
      });

      const id = generateRandomId();

      expect(id).toBe("url-mock-uuid-123");
      expect(mockRandomUUID).toHaveBeenCalled();
    });

    test("should fallback to Date.now and Math.random when crypto unavailable", () => {
      const originalCrypto = (global as any).crypto;

      // Remove crypto from global scope
      (global as any).crypto = undefined;

      const id = generateRandomId();

      expect(id.startsWith("url-")).toBe(true);
      expect(id.includes("-")).toBe(true);

      // Restore crypto
      (global as any).crypto = originalCrypto;
    });
  });
});
