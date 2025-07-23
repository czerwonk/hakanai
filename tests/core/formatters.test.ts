import {
  formatFileSize,
  formatTTL,
  sanitizeFileName,
} from "../../server/src/typescript/core/formatters";

describe("formatFileSize", () => {
  test("returns correct formatted sizes", () => {
    expect(formatFileSize(0)).toBe("0 Bytes");
    expect(formatFileSize(1024)).toBe("1 KB");
    expect(formatFileSize(1048576)).toBe("1 MB");
    expect(formatFileSize(1073741824)).toBe("1 GB");
    expect(formatFileSize(512)).toBe("512 Bytes");
  });

  test("handles decimal values correctly", () => {
    expect(formatFileSize(1536)).toBe("1.5 KB");
    expect(formatFileSize(1572864)).toBe("1.5 MB");
    expect(formatFileSize(2048)).toBe("2 KB");
  });

  test("handles large files", () => {
    expect(formatFileSize(5368709120)).toBe("5 GB");
    expect(formatFileSize(10737418240)).toBe("10 GB");
  });
});

describe("formatTTL", () => {
  test("formats seconds correctly", () => {
    expect(formatTTL(30)).toBe("0 minute");
    expect(formatTTL(60)).toBe("1 minute");
    expect(formatTTL(120)).toBe("2 minutes");
    expect(formatTTL(300)).toBe("5 minutes");
  });

  test("formats hours correctly", () => {
    expect(formatTTL(3600)).toBe("1 hour");
    expect(formatTTL(7200)).toBe("2 hours");
    expect(formatTTL(10800)).toBe("3 hours");
  });

  test("formats days correctly", () => {
    expect(formatTTL(86400)).toBe("1 day");
    expect(formatTTL(172800)).toBe("2 days");
    expect(formatTTL(259200)).toBe("3 days");
    expect(formatTTL(604800)).toBe("7 days");
  });

  test("prefers larger units", () => {
    expect(formatTTL(90000)).toBe("1 day"); // 25 hours -> 1 day
    expect(formatTTL(180000)).toBe("2 days"); // 50 hours -> 2 days
  });

  test("handles edge cases", () => {
    expect(formatTTL(0)).toBe("0 minute");
    expect(formatTTL(59)).toBe("0 minute");
    expect(formatTTL(3599)).toBe("59 minutes");
    expect(formatTTL(86399)).toBe("23 hours");
  });
});

describe("sanitizeFileName", () => {
  test("removes dangerous characters", () => {
    expect(sanitizeFileName('test<>:"/\\|?*file.txt')).toBe(
      "test_________file.txt",
    );
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
    expect(sanitizeFileName("test\x00\x01\x1ffile.txt")).toBe(
      "test___file.txt",
    );
  });
});
