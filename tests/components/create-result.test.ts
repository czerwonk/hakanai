import { displaySuccessResult } from "../../server/src/typescript/components/create-result";

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

      // Should have labels with correct i18n keys
      const labels = container.querySelectorAll("label");
      expect(labels[0]?.textContent).toBe("label.url");
      expect(labels[1]?.textContent).toBe("label.key");

      // Verify i18n.t was called with key-specific labels
      const mockI18n = window.i18n as { t: jest.Mock };
      expect(mockI18n.t).toHaveBeenCalledWith("label.url");
      expect(mockI18n.t).toHaveBeenCalledWith("label.key");
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

    test("should use correct i18n keys for all text elements", async () => {
      const testUrl = "https://example.com/s/123#abcdef";
      const mockI18n = window.i18n as { t: jest.Mock };

      displaySuccessResult(testUrl, {
        container,
        separateKeyMode: false,
      });

      // Wait for async QR code generation
      await new Promise((resolve) => setTimeout(resolve, 0));

      // Verify i18n.t was called with core keys (QR code might not be generated)
      expect(mockI18n.t).toHaveBeenCalledWith("msg.successTitle");
      expect(mockI18n.t).toHaveBeenCalledWith("msg.shareInstructions");
      expect(mockI18n.t).toHaveBeenCalledWith("label.url");
      expect(mockI18n.t).toHaveBeenCalledWith("button.copy");
      expect(mockI18n.t).toHaveBeenCalledWith("msg.createNote");

      // Verify elements contain the i18n keys (since our mock returns keys)
      const title = container.querySelector("h3");
      expect(title?.textContent).toBe("msg.successTitle");

      const instructions = container.querySelector(".share-instructions");
      expect(instructions?.textContent).toBe("msg.shareInstructions");

      const urlLabel = container.querySelector("label");
      expect(urlLabel?.textContent).toBe("label.url");

      const copyButton = container.querySelector(".copy-button");
      expect(copyButton?.textContent).toBe("button.copy");

      const note = container.querySelector(".secret-note");
      expect(note?.textContent).toBe("msg.createNote");
    });

    test("should use correct i18n key for QR code label when QR generation succeeds", async () => {
      const testUrl = "https://example.com/s/123#abcdef";
      const mockI18n = window.i18n as { t: jest.Mock };

      // Ensure QR code generation succeeds
      const {
        QRCodeGenerator,
      } = require("../../server/src/typescript/core/qr-generator");
      QRCodeGenerator.generateQRCode.mockReturnValue("<svg>qr code</svg>");

      displaySuccessResult(testUrl, {
        container,
        separateKeyMode: false,
      });

      // Wait for async QR code generation
      await new Promise((resolve) => setTimeout(resolve, 0));

      // QR code section should be present
      const qrSection = container.querySelector(".qr-code-section");
      expect(qrSection).toBeTruthy();

      // Verify QR code label i18n key was used
      expect(mockI18n.t).toHaveBeenCalledWith("label.qrCode");

      const qrLabel = qrSection?.querySelector("label");
      expect(qrLabel?.textContent).toBe("label.qrCode");
    });
  });
});
