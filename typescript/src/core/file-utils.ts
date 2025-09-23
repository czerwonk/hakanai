// SPDX-License-Identifier: Apache-2.0

/**
 * Common MIME type mappings for file extensions
 */
const MIME_MAP: Record<string, string> = {
  // Images
  jpg: "image/jpeg",
  jpeg: "image/jpeg",
  png: "image/png",
  gif: "image/gif",
  webp: "image/webp",
  heic: "image/heic",
  heif: "image/heif",
  svg: "image/svg+xml",
  bmp: "image/bmp",
  ico: "image/x-icon",

  // Documents
  pdf: "application/pdf",
  doc: "application/msword",
  docx: "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
  xls: "application/vnd.ms-excel",
  xlsx: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
  ppt: "application/vnd.ms-powerpoint",
  pptx: "application/vnd.openxmlformats-officedocument.presentationml.presentation",
  odt: "application/vnd.oasis.opendocument.text",

  // Videos
  mp4: "video/mp4",
  mov: "video/quicktime",
  avi: "video/x-msvideo",
  webm: "video/webm",
  mkv: "video/x-matroska",
  m4v: "video/x-m4v",

  // Audio
  mp3: "audio/mpeg",
  wav: "audio/wav",
  m4a: "audio/mp4",
  ogg: "audio/ogg",
  flac: "audio/flac",

  // Text/Code
  txt: "text/plain",
  json: "application/json",
  xml: "application/xml",
  html: "text/html",
  css: "text/css",
  js: "text/javascript",
  ts: "text/typescript",
  md: "text/markdown",
  csv: "text/csv",

  // Archives
  zip: "application/zip",
  tar: "application/x-tar",
  gz: "application/gzip",
  "7z": "application/x-7z-compressed",
  rar: "application/vnd.rar",
};

/**
 * Get MIME type from filename
 * @param filename The filename to extract extension from
 * @returns MIME type string
 */
export function getMimeType(filename: string): string {
  const ext = getExt(filename);
  return ext && MIME_MAP[ext] ? MIME_MAP[ext] : "application/octet-stream";
}

/**
 * Get file extension from filename
 * @param filename The filename to extract extension from
 * @returns extension in lowercase without the dot
 */
export function getExt(filename: string): string {
  return filename.split(".").pop()?.toLowerCase() || "";
}

export function isImageExt(ext: string): boolean {
  return ["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico", "heic", "heif"].includes(ext.toLowerCase());
}

/**
 * Get appropriate icon for a file based on its extension
 * @param filename The filename to get icon for
 * @returns Emoji icon representing the file type
 */
export function getFileIcon(filename: string): string {
  const ext = getExt(filename);

  // Image files
  if (isImageExt(ext)) {
    return "🖼️";
  }
  // Video files
  if (["mp4", "avi", "mov", "webm", "mkv", "m4v"].includes(ext)) {
    return "🎬";
  }
  // Audio files
  if (["mp3", "wav", "m4a", "ogg", "flac"].includes(ext)) {
    return "🎵";
  }
  // Documents
  if (["pdf"].includes(ext || "")) {
    return "📄";
  }
  if (["doc", "docx", "odt", "rtf"].includes(ext)) {
    return "📝";
  }
  if (["xls", "xlsx", "csv"].includes(ext)) {
    return "📊";
  }
  if (["ppt", "pptx"].includes(ext)) {
    return "📈";
  }
  // Archives
  if (["zip", "7z", "rar", "tar", "gz"].includes(ext)) {
    return "📦";
  }
  // Code/text files
  if (["txt", "md", "json", "xml", "js", "ts", "css", "html", "py", "rs", "go"].includes(ext)) {
    return "📃";
  }

  // Default file icon
  return "📎";
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

/**
 * Read a File object as an ArrayBuffer
 * @param file file to read
 * @returns
 */
export async function readFileAsArrayBuffer(file: File): Promise<ArrayBuffer> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (event) => {
      if (event.target?.result instanceof ArrayBuffer) {
        resolve(event.target.result);
      } else {
        reject(new Error("Failed to read file as ArrayBuffer"));
      }
    };
    reader.onerror = () => reject(new Error("File read error"));
    reader.readAsArrayBuffer(file);
  });
}
