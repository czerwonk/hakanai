/**
 * QR Code generator using WebAssembly
 */

// Type definitions for the WASM module
interface QrGeneratorWasm {
  generate_svg(text: string, size: number): string;
}

interface WasmModule {
  default(): Promise<void>;
  QrGenerator: new () => QrGeneratorWasm;
}

export class QRCodeGenerator {
  private static generator: QrGeneratorWasm | null = null;
  private static loadPromise: Promise<void> | null = null;

  /**
   * Ensure WASM module is loaded (loads once, cached for reuse)
   */
  static async ensureWasmLoaded(): Promise<void> {
    if (this.loadPromise) return this.loadPromise;

    this.loadPromise = this.loadWasm();
    return this.loadPromise;
  }

  /**
   * Load the WASM QR code module
   */
  private static async loadWasm(): Promise<void> {
    try {
      // Dynamic import of the WASM module
      const module = (await import(
        "/hakanai_wasm.js"
      )) as unknown as WasmModule;

      // Initialize the WASM module
      await module.default();

      this.generator = new module.QrGenerator();

      console.debug("QR code WASM module loaded successfully");
    } catch (error) {
      console.warn("Failed to load QR code WASM module:", error);
      this.generator = null;
    }
  }

  /**
   * Generate QR code SVG for the given URL
   * @param url - URL to encode in QR code
   * @param size - Size of the QR code in pixels (default: 200)
   * @returns SVG string or null if generation failed
   */
  static generateQRCode(url: string, size: number = 200): string | null {
    if (!this.generator) return null;

    try {
      return this.generator.generate_svg(url, size);
    } catch (error) {
      console.warn("QR code generation failed:", error);
      return null;
    }
  }

  /**
   * Check if QR code generation is available
   */
  static isAvailable(): boolean {
    return this.generator !== null;
  }
}
