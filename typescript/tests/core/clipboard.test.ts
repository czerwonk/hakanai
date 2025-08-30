// SPDX-License-Identifier: Apache-2.0

import {
  copyToClipboard,
  copyToClipboardByElementId,
} from "../../src/core/clipboard";

describe("Clipboard Operations", () => {
  let mockButton: HTMLButtonElement;
  let mockWriteText: jest.Mock;

  beforeEach(() => {
    // Create mock button
    mockButton = document.createElement("button");
    mockButton.textContent = "Copy";

    // Mock clipboard API
    mockWriteText = jest.fn();
    Object.defineProperty(navigator, "clipboard", {
      value: {
        writeText: mockWriteText,
      },
      writable: true,
    });

    // Mock window.i18n with key->key mapping
    Object.defineProperty(window, "i18n", {
      value: {
        t: jest.fn((key: string) => key), // Just return the key
      },
      writable: true,
    });

    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
    jest.clearAllMocks();
  });

  describe("copyToClipboard", () => {
    test("should copy text and show success feedback", async () => {
      mockWriteText.mockResolvedValue(undefined);

      copyToClipboard("test content", mockButton);

      await Promise.resolve(); // Wait for promise to resolve

      expect(mockWriteText).toHaveBeenCalledWith("test content");
      expect(mockButton.textContent).toBe("button.copied");
      expect(mockButton.classList.contains("copied")).toBe(true);
    });

    test("should restore original text after timeout", async () => {
      mockWriteText.mockResolvedValue(undefined);
      mockButton.textContent = "Original Text";

      copyToClipboard("test content", mockButton);

      await Promise.resolve();

      // Fast-forward time
      jest.advanceTimersByTime(2000);

      expect(mockButton.textContent).toBe("Original Text");
      expect(mockButton.classList.contains("copied")).toBe(false);
    });
  });

  describe("copyToClipboardByElementId", () => {
    test("should copy element value when element exists", () => {
      // Create mock input element
      const input = document.createElement("input");
      input.id = "test-input";
      input.value = "test value";
      document.body.appendChild(input);

      mockWriteText.mockResolvedValue(undefined);

      copyToClipboardByElementId("test-input", mockButton);

      expect(mockWriteText).toHaveBeenCalledWith("test value");

      // Cleanup
      document.body.removeChild(input);
    });

    test("should show error when element not found", () => {
      copyToClipboardByElementId("non-existent", mockButton);

      expect(mockButton.textContent).toBe("msg.copyFailed");
      expect(mockButton.classList.contains("copy-failed")).toBe(true);
    });
  });
});
