import { TarBuilder } from "../../src/core/tar-builder";

describe("TarBuilder", () => {
  it("should create a valid TAR archive with a single file", () => {
    const builder = new TarBuilder();
    const content = new TextEncoder().encode("Hello, World!").buffer;

    builder.addFile("test.txt", content);
    const archive = builder.finalize();

    // TAR header is 512 bytes, content is 13 bytes, padding to 512, plus 1024 end blocks
    const expectedSize = 512 + 512 + 1024; // header + content block + end marker
    expect(archive.length).toBe(expectedSize);

    // Check that filename appears in header
    const headerStr = new TextDecoder().decode(archive.slice(0, 100));
    expect(headerStr).toContain("test.txt");

    // Check that content appears after header
    const contentSection = archive.slice(512, 512 + 13);
    expect(new TextDecoder().decode(contentSection)).toBe("Hello, World!");
  });

  it("should handle multiple files", () => {
    const builder = new TarBuilder();

    builder.addFile("file1.txt", new TextEncoder().encode("First file").buffer);
    builder.addFile("file2.txt", new TextEncoder().encode("Second file").buffer);

    const archive = builder.finalize();

    // Should contain both filenames
    const archiveStr = new TextDecoder().decode(archive);
    expect(archiveStr).toContain("file1.txt");
    expect(archiveStr).toContain("file2.txt");
  });

  it("should throw error for filenames that are too long", () => {
    const builder = new TarBuilder();
    const longName = "a".repeat(101); // 101 characters exceeds 100 byte limit

    expect(() => {
      builder.addFile(longName, new Uint8Array(10).buffer);
    }).toThrow(/Filename too long/);
  });

  it("should handle non-ASCII filenames correctly", () => {
    const builder = new TarBuilder();
    const filename = "文件.txt"; // Chinese characters
    const content = new Uint8Array(10).buffer;

    // This filename is 10 bytes in UTF-8, well within limit
    builder.addFile(filename, content);
    const archive = builder.finalize();

    // Should contain the UTF-8 encoded filename
    const headerBytes = archive.slice(0, 100);
    const headerStr = new TextDecoder().decode(headerBytes);
    expect(headerStr).toContain("文件.txt");
  });

  it("should reject filenames that exceed 100 bytes when encoded", () => {
    const builder = new TarBuilder();
    // 34 Chinese characters × 3 bytes each = 102 bytes in UTF-8
    const longUnicodeFilename = "文".repeat(34);

    expect(() => {
      builder.addFile(longUnicodeFilename, new Uint8Array(10).buffer);
    }).toThrow(/Filename too long/);
  });

  it("should sanitize path traversal attempts", () => {
    const builder = new TarBuilder();
    const content = new TextEncoder().encode("test").buffer;

    // Try various path traversal patterns
    builder.addFile("../../../etc/passwd", content);
    builder.addFile("..\\..\\windows\\system32\\config", content);
    builder.addFile("./hidden/.secret", content);

    const archive = builder.finalize();
    const archiveStr = new TextDecoder().decode(archive);

    // Should not contain directory traversal patterns
    expect(archiveStr).not.toContain("../");
    expect(archiveStr).not.toContain("..\\");
    expect(archiveStr).not.toContain("/etc/");

    // With new logic: extract basename, then sanitize with file-utils
    // "../../../etc/passwd" → "passwd"
    expect(archiveStr).toContain("passwd");
    // "..\\..\\windows\\system32\\config" → "config"
    expect(archiveStr).toContain("config");
    // "./hidden/.secret" → ".secret" → "secret" (leading dots removed by sanitizer)
    expect(archiveStr).toContain("secret");
  });

  it("should remove leading dots from filenames", () => {
    const builder = new TarBuilder();
    const content = new TextEncoder().encode("hidden").buffer;

    builder.addFile(".hidden_file", content);
    builder.addFile("..double_dot", content);

    const archive = builder.finalize();
    const archiveStr = new TextDecoder().decode(archive);

    // Leading dots should be removed
    // ".hidden_file" → "hidden_file"
    expect(archiveStr).toContain("hidden_file");
    // "..double_dot" → "double_dot" (leading dots removed, then ".." → "." but no ".." left)
    expect(archiveStr).toContain("double_dot");
    expect(archiveStr).not.toContain("\0.hidden_file");
    expect(archiveStr).not.toContain("\0..double_dot");
  });

  it("should reject dot-only filenames", () => {
    const builder = new TarBuilder();
    const content = new TextEncoder().encode("test").buffer;

    // Single dot returns null from sanitizer
    expect(() => {
      builder.addFile(".", content);
    }).toThrow(/Invalid filename.*cannot be sanitized/);

    // ".." returns null from sanitizer
    expect(() => {
      builder.addFile("..", content);
    }).toThrow(/Invalid filename.*cannot be sanitized/);

    // "..." returns null from sanitizer
    expect(() => {
      builder.addFile("...", content);
    }).toThrow(/Invalid filename.*cannot be sanitized/);

    // When in a path, basenames are extracted first
    // "/path/to/.hidden" → ".hidden" → "hidden" (works)
    const builder2 = new TarBuilder();
    builder2.addFile("/path/to/.hidden", content);
    const archive = builder2.finalize();
    expect(new TextDecoder().decode(archive)).toContain("hidden");
  });

  it("should handle binary content correctly", () => {
    const builder = new TarBuilder();

    // Create binary content with all byte values
    const binaryContent = new Uint8Array(256);
    for (let i = 0; i < 256; i++) {
      binaryContent[i] = i;
    }

    builder.addFile("binary.dat", binaryContent.buffer);
    const archive = builder.finalize();

    // Extract content from archive (after 512-byte header)
    const extractedContent = archive.slice(512, 512 + 256);

    // Should match original binary content
    expect(extractedContent).toEqual(binaryContent);
  });

  it("should properly pad files to 512-byte boundaries", () => {
    const builder = new TarBuilder();

    // File with size that doesn't align to 512 bytes
    const content = new Uint8Array(100).buffer; // 100 bytes needs 412 bytes padding
    builder.addFile("padded.txt", content);

    const archive = builder.finalize();

    // Header (512) + content (100) + padding (412) + end marker (1024) = 2048
    expect(archive.length).toBe(2048);

    // Check that padding area is zeros
    const paddingStart = 512 + 100;
    const paddingEnd = 512 + 512;
    const padding = archive.slice(paddingStart, paddingEnd);

    for (let i = 0; i < padding.length; i++) {
      expect(padding[i]).toBe(0);
    }
  });

  it("should track current size before finalization", () => {
    const builder = new TarBuilder();

    expect(builder.getCurrentSize()).toBe(0);

    // Add a small file (10 bytes content)
    builder.addFile("small.txt", new Uint8Array(10).buffer);
    // Header (512) + content padded to 512 = 1024
    expect(builder.getCurrentSize()).toBe(1024);

    // Add another file
    builder.addFile("another.txt", new Uint8Array(20).buffer);
    // Previous (1024) + header (512) + content padded to 512 = 2048
    expect(builder.getCurrentSize()).toBe(2048);
  });

  it("should set correct file mode and timestamps", () => {
    const builder = new TarBuilder();
    const content = new TextEncoder().encode("test").buffer;

    const beforeTime = Math.floor(Date.now() / 1000);
    builder.addFile("test.txt", content);
    const afterTime = Math.floor(Date.now() / 1000);

    const archive = builder.finalize();

    // Check file mode at position 100-107 (should be "0644")
    const fileMode = new TextDecoder().decode(archive.slice(100, 104));
    expect(fileMode).toBe("0644");

    // Check timestamp at position 136-147 (octal)
    const timestampOctal = new TextDecoder().decode(archive.slice(136, 147)).trim();
    const timestamp = parseInt(timestampOctal, 8);

    // Timestamp should be between before and after
    expect(timestamp).toBeGreaterThanOrEqual(beforeTime);
    expect(timestamp).toBeLessThanOrEqual(afterTime);
  });

  it("should calculate correct checksum", () => {
    const builder = new TarBuilder();
    builder.addFile("test.txt", new TextEncoder().encode("test").buffer);

    const archive = builder.finalize();

    // Extract checksum from header (position 148-155)
    const checksumField = archive.slice(148, 156);
    const storedChecksum = parseInt(new TextDecoder().decode(checksumField).trim(), 8);

    // Recalculate checksum
    let calculatedChecksum = 0;
    for (let i = 0; i < 512; i++) {
      if (i >= 148 && i < 156) {
        calculatedChecksum += 32; // Space character
      } else {
        calculatedChecksum += archive[i];
      }
    }

    expect(storedChecksum).toBe(calculatedChecksum);
  });

  it("should create TAR with proper end marker", () => {
    const builder = new TarBuilder();
    builder.addFile("test.txt", new TextEncoder().encode("test").buffer);

    const archive = builder.finalize();

    // TAR files should end with two 512-byte blocks of zeros
    const endMarker = archive.slice(archive.length - 1024);

    for (let i = 0; i < 1024; i++) {
      expect(endMarker[i]).toBe(0);
    }
  });
});
