// SPDX-License-Identifier: Apache-2.0

import { sanitizeFileName } from "../../src/core/file-utils";

describe("sanitizeFileName", () => {
  test("removes dangerous characters", () => {
    expect(sanitizeFileName('test<>:"/\\|?*file.txt')).toBe("test_________file.txt");
  });

  test("removes leading dots", () => {
    expect(sanitizeFileName(".hidden")).toBe("hidden");
    expect(sanitizeFileName("...test")).toBe("test");
  });

  test("preserves normal filenames", () => {
    expect(sanitizeFileName("normal.txt")).toBe("normal.txt");
    expect(sanitizeFileName("document.pdf")).toBe("document.pdf");
    expect(sanitizeFileName("image_file.jpg")).toBe("image_file.jpg");
  });

  test("returns null for invalid filenames", () => {
    expect(sanitizeFileName("")).toBe(null);
    expect(sanitizeFileName("...")).toBe(null);
    expect(sanitizeFileName("   ")).toBe(null);
  });

  test("limits filename length", () => {
    const longName = "a".repeat(300);
    const result = sanitizeFileName(longName);
    expect(result).not.toBeNull();
    expect(result!.length).toBe(255);
  });

  test("handles control characters", () => {
    expect(sanitizeFileName("test\x00\x01\x1ffile.txt")).toBe("test___file.txt");
  });
});
