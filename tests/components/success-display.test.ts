import { displaySuccessResult } from "../../server/src/typescript/components/success-display";

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

    // Mock window.i18n
    Object.defineProperty(window, "i18n", {
      value: {
        t: jest.fn((key: string) => {
          const translations: Record<string, string> = {
            "msg.successTitle": "Secret Created Successfully",
            "msg.shareInstructions":
              "Share this URL with the intended recipient.",
            "label.secretUrl": "Secret URL:",
            "label.decryptionKey": "Decryption Key:",
            "label.qrCode": "QR Code:",
            "button.copy": "Copy",
            "msg.createNote":
              "Note: Share this URL carefully. The secret will be deleted after the first access or when it expires.",
          };
          return translations[key] || key;
        }),
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

      // Check for success header
      const title = container.querySelector("h3");
      expect(title?.textContent).toBe("Secret Created Successfully");

      const instructions = container.querySelector(".share-instructions");
      expect(instructions?.textContent).toBe(
        "Share this URL with the intended recipient.",
      );

      // Check for URL input
      const urlInput = container.querySelector(
        'input[type="text"]',
      ) as HTMLInputElement;
      expect(urlInput?.value).toBe(testUrl);
      expect(urlInput?.readOnly).toBe(true);

      // Check for copy button
      const copyButton = container.querySelector(".copy-button");
      expect(copyButton?.textContent).toBe("Copy");

      // Check for QR code section
      const qrSection = container.querySelector(".qr-code-section");
      expect(qrSection).toBeTruthy();

      // Check for security note
      const note = container.querySelector(".secret-note");
      expect(note?.textContent).toContain("Note:");
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

      // Should have two copy buttons
      const copyButtons = container.querySelectorAll(".copy-button");
      expect(copyButtons).toHaveLength(2);

      // Should have labels for URL and key
      const labels = container.querySelectorAll("label");
      expect(labels[0]?.textContent).toBe("Secret URL:");
      expect(labels[1]?.textContent).toBe("Decryption Key:");
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

    test("should handle WASM loading failure gracefully", async () => {
      const {
        QRCodeGenerator,
      } = require("../../server/src/typescript/core/qr-generator");
      QRCodeGenerator.ensureWasmLoaded.mockRejectedValue(
        new Error("WASM load failed"),
      );

      const consoleSpy = jest.spyOn(console, "debug").mockImplementation();

      const testUrl = "https://example.com/s/123#abcdef";

      displaySuccessResult(testUrl, {
        container,
        separateKeyMode: false,
      });

      // Wait for async QR code generation
      await new Promise((resolve) => setTimeout(resolve, 0));

      // Should log debug message about QR code not being available
      expect(consoleSpy).toHaveBeenCalledWith(
        "QR code not available:",
        expect.any(Error),
      );

      // QR code section should not be present
      const qrSection = container.querySelector(".qr-code-section");
      expect(qrSection).toBeFalsy();

      consoleSpy.mockRestore();
    });

    test("should use fallback text when i18n is not available", async () => {
      delete (global as any).window.i18n;

      const testUrl = "https://example.com/s/123#abcdef";

      displaySuccessResult(testUrl, {
        container,
        separateKeyMode: false,
      });

      // Wait for async QR code generation
      await new Promise((resolve) => setTimeout(resolve, 0));

      // Should use fallback English text
      const title = container.querySelector("h3");
      expect(title?.textContent).toBe("Secret Created Successfully");

      const urlLabel = container.querySelector("label");
      expect(urlLabel?.textContent).toBe("Secret URL:");
    });
  });
});
