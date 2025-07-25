import { displayErrorMessage } from "../../server/src/typescript/components/error-display";

// Mock DOM utilities
jest.mock("../../server/src/typescript/core/dom-utils", () => ({
  announceToScreenReader: jest.fn(),
}));

describe("Error Display Component", () => {
  let container: HTMLElement;

  beforeEach(() => {
    container = document.createElement("div");
    container.id = "result";
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
    test("should create error display with default container", () => {
      const testMessage = "Test error message";

      displayErrorMessage(testMessage);

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
      document.body.appendChild(customContainer);

      const testMessage = "Custom container error";

      displayErrorMessage(testMessage, { containerId: "custom-error" });

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

      displayErrorMessage("New error");

      expect(container.className).toBe("result error");
      expect(container.querySelector("p")).toBeNull();

      const title = container.querySelector("h3");
      const errorDiv = container.querySelector("div");
      expect(title).toBeTruthy();
      expect(errorDiv).toBeTruthy();
    });

    test("should handle missing container gracefully", () => {
      const consoleSpy = jest.spyOn(console, "error").mockImplementation();

      displayErrorMessage("Test message", { containerId: "non-existent" });

      expect(consoleSpy).toHaveBeenCalledWith(
        "Error container 'non-existent' not found",
      );

      consoleSpy.mockRestore();
    });

    test("should announce to screen reader", () => {
      const {
        announceToScreenReader,
      } = require("../../server/src/typescript/core/dom-utils");
      const testMessage = "Screen reader test";

      displayErrorMessage(testMessage);

      expect(announceToScreenReader).toHaveBeenCalledWith(
        "msg.errorTitle: " + testMessage,
      );
    });

    test("should handle empty message", () => {
      displayErrorMessage("");

      const errorDiv = container.querySelector("div");
      expect(errorDiv?.textContent).toBe("");
    });

    test("should handle special characters in message", () => {
      const specialMessage =
        "Error with <script>alert('xss')</script> & symbols";

      displayErrorMessage(specialMessage);

      const errorDiv = container.querySelector("div");
      expect(errorDiv?.textContent).toBe(specialMessage);
      // textContent should escape HTML automatically
      expect(errorDiv?.innerHTML).not.toContain("<script>");
    });

    test("should use i18n for error title", () => {
      displayErrorMessage("Test message");

      expect((window as any).i18n.t).toHaveBeenCalledWith("msg.errorTitle");
    });
  });
});
