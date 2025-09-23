/**
 * Simple TAR archive builder for the browser
 *
 * Creates uncompressed TAR archives without any external dependencies.
 * Implements only the subset needed for Hakanai's multi-file bundling.
 *
 * Security features:
 * - Filename sanitization (no path traversal)
 * - Length validation (100 byte TAR limit)
 * - Safe permissions (0644)
 * - No symlinks or special files
 */

import { sanitizeFileName } from "./file-utils";

const BLOCK_SIZE = 512;

export class TarBuilder {
  private chunks: Uint8Array[] = [];
  private encoder = new TextEncoder();

  /**
   * Add a file to the TAR archive
   * @param filename - Name of the file (will be sanitized)
   * @param content - File content
   * @throws Error if filename is too long or invalid
   */
  addFile(filename: string, content: ArrayBuffer): void {
    const basename = filename.split(/[/\\]/).pop() || filename;

    const sanitized = sanitizeFileName(basename);
    if (!sanitized) {
      throw new Error(`Invalid filename: "${filename}" cannot be sanitized`);
    }

    const nameBytes = this.encoder.encode(sanitized);
    if (nameBytes.length > 100) {
      throw new Error(`Filename too long: ${sanitized} (${nameBytes.length} bytes, max 100)`);
    }

    const header = this.createHeader(sanitized, content.byteLength);
    this.chunks.push(header);

    this.chunks.push(new Uint8Array(content));

    // Add padding to align to 512-byte boundary
    const padding = (BLOCK_SIZE - (content.byteLength % BLOCK_SIZE)) % BLOCK_SIZE;
    if (padding > 0) {
      this.chunks.push(new Uint8Array(padding));
    }
  }

  /**
   * Finalize the TAR archive and return the complete data
   * @returns Complete TAR archive as bytes
   */
  finalize(): Uint8Array {
    // TAR files end with two 512-byte blocks of zeros
    this.chunks.push(new Uint8Array(1024));

    // Calculate total size
    const totalSize = this.chunks.reduce((sum, chunk) => sum + chunk.length, 0);

    // Concatenate all chunks
    const result = new Uint8Array(totalSize);
    let offset = 0;
    for (const chunk of this.chunks) {
      result.set(chunk, offset);
      offset += chunk.length;
    }

    return result;
  }

  /**
   * Get the current size of the archive (before finalization)
   * Useful for checking size limits before adding more files
   */
  getCurrentSize(): number {
    return this.chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  }

  /**
   * Create a TAR header for a file
   * Using POSIX ustar format for maximum compatibility
   */
  private createHeader(filename: string, fileSize: number): Uint8Array {
    const header = new Uint8Array(512); // TAR headers are always 512 bytes

    // Fill with zeros (already done by Uint8Array constructor)

    // Filename (0-99)
    const nameBytes = this.encoder.encode(filename);
    header.set(nameBytes, 0);

    // File mode "0644" (100-107) - read/write for owner, read for others
    header.set(this.encoder.encode("0644\0"), 100);

    // Owner ID "0000000" (108-115)
    header.set(this.encoder.encode("0000000"), 108);

    // Group ID "0000000" (116-123)
    header.set(this.encoder.encode("0000000"), 116);

    // File size in octal, null-terminated (124-135)
    const sizeOctal = fileSize.toString(8).padStart(11, "0");
    header.set(this.encoder.encode(sizeOctal), 124);

    // Modification time (current time in octal) (136-147)
    const mtime = Math.floor(Date.now() / 1000)
      .toString(8)
      .padStart(11, "0");
    header.set(this.encoder.encode(mtime), 136);

    // Checksum placeholder - 8 spaces (148-155)
    header.set(this.encoder.encode("        "), 148);

    // File type '0' = normal file (156)
    header.set([48], 156); // ASCII '0'

    // Link name (157-256) - empty for normal files

    // POSIX ustar indicator (257-264)
    header.set(this.encoder.encode("ustar\0"), 257);

    // Version "00" (265-266)
    header.set(this.encoder.encode("00"), 265);

    // Owner name (267-296) - using generic "user"
    header.set(this.encoder.encode("user"), 267);

    // Group name (297-328) - using generic "group"
    header.set(this.encoder.encode("group"), 297);

    // Device numbers (329-344) - not used for regular files

    // Calculate and set checksum
    this.setChecksum(header);

    return header;
  }

  /**
   * Calculate and set the checksum for a TAR header
   * The checksum is the sum of all header bytes, treating the
   * checksum field itself as spaces
   */
  private setChecksum(header: Uint8Array): void {
    // Calculate checksum (sum of all bytes)
    let checksum = 0;
    for (let i = 0; i < 512; i++) {
      // Treat checksum field (148-155) as spaces (ASCII 32)
      if (i >= 148 && i < 156) {
        checksum += 32;
      } else {
        checksum += header[i];
      }
    }

    // Format checksum as 6 octal digits + null + space
    const checksumStr = checksum.toString(8).padStart(6, "0") + "\0 ";
    header.set(this.encoder.encode(checksumStr), 148);
  }
}
