import { displaySuccessResult } from "../../server/src/typescript/components/success-display";
import { I18n } from "../../server/src/typescript/core/i18n";

// Mock the QR code generator
jest.mock("../../server/src/typescript/core/qr-generator", () => ({
  QRCodeGenerator: {
    ensureWasmLoaded: jest.fn().mockResolvedValue(undefined),
    generateQRCode: jest.fn().mockReturnValue("<svg>mock qr code</svg>"),
  },
}));

describe("Success Display Component", () => {
  let container: HTMLElement;

  beforeEach(() => {
    container = document.createElement("div");
    document.body.appendChild(container);

    // Mock localStorage for i18n
    Object.defineProperty(window, "localStorage", {
      value: {
        getItem: jest.fn(),
        setItem: jest.fn(),
      },
      writable: true,
    });

    // Mock navigator.language
    Object.defineProperty(navigator, "language", {
      value: "en-US",
      writable: true,
    });

    // Initialize real i18n instead of mocking
    const i18n = new I18n();

    Object.defineProperty(window, "i18n", {
      value: i18n,
      writable: true,
    });
  });

  afterEach(() => {
    document.body.removeChild(container);
    jest.clearAllMocks();
  });

  describe("displaySuccessResult", () => {
    test("should create complete success display for combined URL", async () => {
      const testUrl = "https://example.com/s/123#abcdef";

      displaySuccessResult(testUrl, {
        container,
        separateKeyMode: false,
      });

      // Wait for async QR code generation
      await new Promise((resolve) => setTimeout(resolve, 0));

      expect(container.className).toBe("result success");

      // Check for structural elements (not text content)
      const title = container.querySelector("h3");
      expect(title).toBeTruthy();

      const instructions = container.querySelector(".share-instructions");
      expect(instructions).toBeTruthy();

      // Check for URL input with correct value
      const urlInput = container.querySelector(
        'input[type="text"]',
      ) as HTMLInputElement;
      expect(urlInput?.value).toBe(testUrl);
      expect(urlInput?.readOnly).toBe(true);

      // Check for copy button exists
      const copyButton = container.querySelector(".copy-button");
      expect(copyButton).toBeTruthy();

      // Check for QR code section
      const qrSection = container.querySelector(".qr-code-section");
      expect(qrSection).toBeTruthy();

      // Check for security note exists
      const note = container.querySelector(".secret-note");
      expect(note).toBeTruthy();
    });

    test("should create separate URL and key display when in separate key mode", async () => {
      const testUrl = "https://example.com/s/123#abcdef";

      displaySuccessResult(testUrl, {
        container,
        separateKeyMode: true,
      });

      // Wait for async QR code generation
      await new Promise((resolve) => setTimeout(resolve, 0));

      // Should have two input fields - one for URL, one for key
      const inputs = container.querySelectorAll('input[type="text"]');
      expect(inputs).toHaveLength(2);

      const urlInput = inputs[0] as HTMLInputElement;
      const keyInput = inputs[1] as HTMLInputElement;

      expect(urlInput.value).toBe("https://example.com/s/123");
      expect(keyInput.value).toBe("abcdef");

      // Verify that key input has the "-key" suffix in its ID
      expect(keyInput.id).toMatch(/-key$/);
      expect(urlInput.id).not.toMatch(/-key$/);

      // Should have two copy buttons
      const copyButtons = container.querySelectorAll(".copy-button");
      expect(copyButtons).toHaveLength(2);

      // Should have proper number of labels (including QR label)
      const labels = container.querySelectorAll("label");
      expect(labels.length).toBeGreaterThanOrEqual(2);
    });

    test("should handle QR code generation failure gracefully", async () => {
      const {
        QRCodeGenerator,
      } = require("../../server/src/typescript/core/qr-generator");
      QRCodeGenerator.generateQRCode.mockReturnValue(null);

      const testUrl = "https://example.com/s/123#abcdef";

      displaySuccessResult(testUrl, {
        container,
        separateKeyMode: false,
      });

      // Wait for async QR code generation
      await new Promise((resolve) => setTimeout(resolve, 0));

      // QR code section should not be present
      const qrSection = container.querySelector(".qr-code-section");
      expect(qrSection).toBeFalsy();

      // But other elements should still be present
      const title = container.querySelector("h3");
      expect(title).toBeTruthy();
    });

    test("should create proper element structure with i18n", async () => {
      const testUrl = "https://example.com/s/123#abcdef";

      displaySuccessResult(testUrl, {
        container,
        separateKeyMode: false,
      });

      // Wait for async QR code generation
      await new Promise((resolve) => setTimeout(resolve, 0));

      // Should have proper structural elements
      const title = container.querySelector("h3");
      expect(title).toBeTruthy();
      expect(title?.textContent).toBeTruthy(); // Has some content

      const urlLabel = container.querySelector("label");
      expect(urlLabel).toBeTruthy();
      expect(urlLabel?.getAttribute("for")).toBeTruthy(); // Has for attribute

      // Label should be associated with input
      const labelFor = urlLabel?.getAttribute("for");
      const associatedInput = labelFor
        ? document.getElementById(labelFor)
        : null;
      expect(associatedInput).toBeTruthy();
    });
  });
});
