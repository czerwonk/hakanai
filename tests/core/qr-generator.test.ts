import { QRCodeGenerator } from "../../server/src/typescript/core/qr-generator";

describe("QRCodeGenerator", () => {
  beforeEach(() => {
    // Reset the static properties
    (QRCodeGenerator as any).wasmModule = null;
    (QRCodeGenerator as any).loadPromise = null;
  });

  afterEach(() => {
    jest.clearAllMocks();
  });

  describe("ensureWasmLoaded", () => {
    test("should load WASM module once and cache it", async () => {
      const loadWasmSpy = jest.spyOn(QRCodeGenerator as any, "loadWasm");
      loadWasmSpy.mockResolvedValue(undefined);

      // First call
      await QRCodeGenerator.ensureWasmLoaded();
      expect(loadWasmSpy).toHaveBeenCalledTimes(1);

      // Second call should use cached promise
      await QRCodeGenerator.ensureWasmLoaded();
      expect(loadWasmSpy).toHaveBeenCalledTimes(1);

      loadWasmSpy.mockRestore();
    });

    test("should handle load failure", async () => {
      const loadWasmSpy = jest.spyOn(QRCodeGenerator as any, "loadWasm");
      loadWasmSpy.mockRejectedValue(new Error("WASM load failed"));

      await expect(QRCodeGenerator.ensureWasmLoaded()).rejects.toThrow(
        "WASM load failed",
      );

      loadWasmSpy.mockRestore();
    });
  });

  describe("generateQRCode", () => {
    test("should return null when WASM module not loaded", () => {
      const result = QRCodeGenerator.generateQRCode("https://example.com");
      expect(result).toBeNull();
    });

    test("should generate QR code when WASM module is loaded", () => {
      // Mock loaded WASM module
      (QRCodeGenerator as any).wasmModule = {
        generate_qr_svg: jest.fn().mockReturnValue("<svg>mock qr code</svg>"),
      };

      const result = QRCodeGenerator.generateQRCode("https://example.com");

      expect(result).toBe("<svg>mock qr code</svg>");
      expect(
        (QRCodeGenerator as any).wasmModule.generate_qr_svg,
      ).toHaveBeenCalledWith("https://example.com");
    });

    test("should handle generation errors gracefully", () => {
      const consoleSpy = jest.spyOn(console, "warn").mockImplementation();

      // Mock WASM module that throws
      (QRCodeGenerator as any).wasmModule = {
        generate_qr_svg: jest.fn().mockImplementation(() => {
          throw new Error("QR generation failed");
        }),
      };

      const result = QRCodeGenerator.generateQRCode("https://example.com");

      expect(result).toBeNull();
      expect(consoleSpy).toHaveBeenCalledWith(
        "QR code generation failed:",
        expect.any(Error),
      );

      consoleSpy.mockRestore();
    });
  });

  describe("loadWasm", () => {
    test("should set up mock WASM module for development", async () => {
      const consoleSpy = jest.spyOn(console, "debug").mockImplementation();

      await (QRCodeGenerator as any).loadWasm();

      expect((QRCodeGenerator as any).wasmModule).toBeDefined();
      expect(
        (QRCodeGenerator as any).wasmModule.generate_qr_svg,
      ).toBeInstanceOf(Function);
      expect(consoleSpy).toHaveBeenCalledWith(
        "QR code WASM module would be loaded here",
      );

      consoleSpy.mockRestore();
    });

    test("should generate placeholder SVG", async () => {
      await (QRCodeGenerator as any).loadWasm();

      const svg = (QRCodeGenerator as any).wasmModule.generate_qr_svg("test");

      expect(svg).toContain("<svg");
      expect(svg).toContain("QR CODE");
      expect(svg).toContain("PLACEHOLDER");
    });

    test("should handle load failures", async () => {
      const consoleSpy = jest.spyOn(console, "warn").mockImplementation();

      // Mock console.debug to throw an error to simulate load failure
      const debugSpy = jest.spyOn(console, "debug").mockImplementation(() => {
        throw new Error("Simulated load failure");
      });

      await expect((QRCodeGenerator as any).loadWasm()).rejects.toThrow(
        "Simulated load failure",
      );

      expect(consoleSpy).toHaveBeenCalledWith(
        "Failed to load QR code WASM module:",
        expect.any(Error),
      );

      consoleSpy.mockRestore();
      debugSpy.mockRestore();
    });
  });
});
