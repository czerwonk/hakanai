/**
 * Tests for ContentAnalysis binary detection functionality
 */

import { ContentAnalysis } from "../../server/src/typescript/hakanai-client";

// Helper function to ensure we get proper Uint8Array in tests
function encodeText(text: string): Uint8Array {
  const encoder = new TextEncoder();
  const encoded = encoder.encode(text);
  return new Uint8Array(encoded);
}

describe("ContentAnalysis", () => {
  test("isBinary detects null bytes correctly", () => {
    // Text content without null bytes
    const textBytes = encodeText("Hello, world!");
    expect(ContentAnalysis.isBinary(textBytes)).toBe(false);

    // Binary content with null bytes
    const binaryBytes = new Uint8Array([0x00, 0x01, 0x02, 0xff]);
    expect(ContentAnalysis.isBinary(binaryBytes)).toBe(true);
  });

  test("isBinary returns false for empty arrays", () => {
    const emptyBytes = new Uint8Array(0);
    expect(ContentAnalysis.isBinary(emptyBytes)).toBe(false);
  });

  test("isBinary detects null bytes anywhere in the array", () => {
    // Null byte at the beginning
    const startNull = new Uint8Array([0x00, 0x48, 0x65, 0x6c, 0x6c, 0x6f]);
    expect(ContentAnalysis.isBinary(startNull)).toBe(true);

    // Null byte in the middle
    const middleNull = new Uint8Array([0x48, 0x65, 0x00, 0x6c, 0x6c, 0x6f]);
    expect(ContentAnalysis.isBinary(middleNull)).toBe(true);

    // Null byte at the end
    const endNull = new Uint8Array([0x48, 0x65, 0x6c, 0x6c, 0x6f, 0x00]);
    expect(ContentAnalysis.isBinary(endNull)).toBe(true);
  });

  test("isBinary handles UTF-8 text correctly", () => {
    const utf8Text = encodeText("Hello, 世界! 🌍");
    expect(ContentAnalysis.isBinary(utf8Text)).toBe(false);

    const specialChars = encodeText("åëïöü äöü ñ");
    expect(ContentAnalysis.isBinary(specialChars)).toBe(false);
  });

  test("isBinary handles typical binary file headers", () => {
    // PNG file signature
    const pngHeader = new Uint8Array([
      0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a,
    ]);
    expect(ContentAnalysis.isBinary(pngHeader)).toBe(false); // PNG header doesn't contain null bytes

    // Add null byte to simulate binary content
    const binaryWithNull = new Uint8Array([
      0x89, 0x50, 0x4e, 0x47, 0x00, 0x0a, 0x1a, 0x0a,
    ]);
    expect(ContentAnalysis.isBinary(binaryWithNull)).toBe(true);
  });

  test("isBinary validates input type", () => {
    expect(() => ContentAnalysis.isBinary("not a uint8array" as any)).toThrow(
      "Input must be a Uint8Array",
    );
    expect(() => ContentAnalysis.isBinary(null as any)).toThrow(
      "Input must be a Uint8Array",
    );
    expect(() => ContentAnalysis.isBinary(undefined as any)).toThrow(
      "Input must be a Uint8Array",
    );
    expect(() => ContentAnalysis.isBinary([1, 2, 3] as any)).toThrow(
      "Input must be a Uint8Array",
    );
  });

  test("isBinary handles large arrays efficiently", () => {
    // Create a large text array without null bytes
    const largeText = encodeText("x".repeat(10000));
    expect(ContentAnalysis.isBinary(largeText)).toBe(false);

    // Create a large array with a null byte somewhere in the middle
    const largeWithNull = new Uint8Array(10000);
    largeWithNull.fill(65); // Fill with 'A' (ASCII 65)
    largeWithNull[5000] = 0; // Add null byte in the middle
    expect(ContentAnalysis.isBinary(largeWithNull)).toBe(true);
  });
});
