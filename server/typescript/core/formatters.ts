// SPDX-License-Identifier: Apache-2.0

/**
 * Format file size in human readable format
 * @param bytes - File size in bytes
 * @returns Formatted file size string (e.g., "1.5 MB")
 */
export function formatFileSize(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

/**
 * Format TTL seconds as human readable string
 * @param seconds - Duration in seconds
 * @returns Human readable duration string
 */
export function formatTTL(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const days = Math.floor(hours / 24);

  if (days > 0) {
    return `${days} day${days > 1 ? "s" : ""}`;
  } else if (hours > 0) {
    return `${hours} hour${hours > 1 ? "s" : ""}`;
  } else {
    const minutes = Math.floor(seconds / 60);
    return `${minutes} minute${minutes > 1 ? "s" : ""}`;
  }
}

/**
 * Sanitize filename by removing invalid characters and limiting length
 * @param fileName - Original filename to sanitize
 * @returns Sanitized filename or null if invalid
 */
export function sanitizeFileName(fileName: string): string | null {
  const sanitized = fileName
    .replace(/[<>:"/\\|?*\x00-\x1f]/g, "_")
    .replace(/^\.+/, "")
    .substring(0, 255)
    .trim();

  return sanitized.length > 0 ? sanitized : null;
}
