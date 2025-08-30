// SPDX-License-Identifier: Apache-2.0

import { displayErrorMessage } from "../../src/components/error-display";

// Mock DOM utilities
jest.mock("../../src/core/dom-utils", () => ({
  announceToScreenReader: jest.fn(),
}));

describe("Error Display Component", () => {
  let container: HTMLElement;

  beforeEach(() => {
    container = document.createElement("div");
    container.id = "result";

    // Mock DOM methods that aren't available in JSDOM
    container.scrollIntoView = jest.fn();
    container.focus = jest.fn();

    document.body.appendChild(container);

    // Mock window.i18n with key->key mapping
    Object.defineProperty(window, "i18n", {
      value: {
        t: jest.fn((key: string) => key), // Return the key itself
      },
      writable: true,
    });
  });

  afterEach(() => {
    document.body.removeChild(container);
    jest.clearAllMocks();
  });

  describe("displayErrorMessage", () => {
    test("should create error display with container element", () => {
      const testMessage = "Test error message";

      displayErrorMessage(testMessage, container);

      expect(container.className).toBe("result error");
      expect(container.innerHTML).not.toBe("");

      // Check for structural elements
      const title = container.querySelector("h3");
      expect(title).toBeTruthy();
      expect(title?.textContent).toBe("msg.errorTitle");

      const errorDiv = container.querySelector("div");
      expect(errorDiv).toBeTruthy();
      expect(errorDiv?.textContent).toBe(testMessage);
    });

    test("should create error display with custom container", () => {
      const customContainer = document.createElement("div");
      customContainer.id = "custom-error";

      // Mock DOM methods for custom container
      customContainer.scrollIntoView = jest.fn();
      customContainer.focus = jest.fn();

      document.body.appendChild(customContainer);

      const testMessage = "Custom container error";

      displayErrorMessage(testMessage, customContainer);

      expect(customContainer.className).toBe("result error");
      expect(customContainer.innerHTML).not.toBe("");

      const title = customContainer.querySelector("h3");
      expect(title?.textContent).toBe("msg.errorTitle");

      const errorDiv = customContainer.querySelector("div");
      expect(errorDiv?.textContent).toBe(testMessage);

      document.body.removeChild(customContainer);
    });

    test("should clear previous content", () => {
      container.innerHTML = "<p>Previous content</p>";
      container.className = "previous-class";

      displayErrorMessage("New error", container);

      expect(container.className).toBe("result error");
      expect(container.querySelector("p")).toBeNull();

      const title = container.querySelector("h3");
      const errorDiv = container.querySelector("div");
      expect(title).toBeTruthy();
      expect(errorDiv).toBeTruthy();
    });

    test("should handle null container gracefully", () => {
      const consoleSpy = jest.spyOn(console, "error").mockImplementation();
      const nullContainer = null as any;

      expect(() =>
        displayErrorMessage("Test message", nullContainer),
      ).toThrow();

      consoleSpy.mockRestore();
    });

    test("should announce to screen reader", () => {
      const { announceToScreenReader } = require("../../src/core/dom-utils");
      const testMessage = "Screen reader test";

      displayErrorMessage(testMessage, container);

      expect(announceToScreenReader).toHaveBeenCalledWith(
        "msg.errorTitle: " + testMessage,
      );
    });

    test("should handle empty message", () => {
      displayErrorMessage("", container);

      const errorDiv = container.querySelector("div");
      expect(errorDiv?.textContent).toBe("");
    });

    test("should handle special characters in message", () => {
      const specialMessage =
        "Error with <script>alert('xss')</script> & symbols";

      displayErrorMessage(specialMessage, container);

      const errorDiv = container.querySelector("div");
      expect(errorDiv?.textContent).toBe(specialMessage);
      // textContent should escape HTML automatically
      expect(errorDiv?.innerHTML).not.toContain("<script>");
    });

    test("should use i18n for error title", () => {
      displayErrorMessage("Test message", container);

      expect((window as any).i18n.t).toHaveBeenCalledWith("msg.errorTitle");
    });

    test("should make container focusable with tabindex", () => {
      displayErrorMessage("Test message", container);

      expect(container.getAttribute("tabindex")).toBe("-1");
    });

    test("should call scrollIntoView and focus on container", () => {
      // Mock scrollIntoView and focus methods
      const scrollIntoViewMock = jest.fn();
      const focusMock = jest.fn();
      container.scrollIntoView = scrollIntoViewMock;
      container.focus = focusMock;

      displayErrorMessage("Test message", container);

      expect(scrollIntoViewMock).toHaveBeenCalledWith({
        behavior: "smooth",
        block: "center",
      });
      expect(focusMock).toHaveBeenCalled();
    });

    test("should set focus behavior after DOM manipulation", () => {
      const focusMock = jest.fn();
      const scrollIntoViewMock = jest.fn();
      container.focus = focusMock;
      container.scrollIntoView = scrollIntoViewMock;

      displayErrorMessage("Test message", container);

      // Verify container has correct properties before focus
      expect(container.className).toBe("result error");
      expect(container.getAttribute("tabindex")).toBe("-1");

      // Verify focus methods were called
      expect(scrollIntoViewMock).toHaveBeenCalled();
      expect(focusMock).toHaveBeenCalled();
    });
  });
});
